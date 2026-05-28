//! Integration test for RISK-003 / SIM-L2-002: FUSE write during hydration
//! must be blocked to prevent file corruption.
//!
//! The mitigation has two layers, verified here:
//!
//! 1. `InodeEntry::lock_state_guard()` — per-inode `parking_lot::Mutex` that
//!    serializes a FUSE `write()`'s `is_hydrating` check + cache write with
//!    `HydrationManager::hydrate()`'s active-map registration. Prevents a
//!    hydration from starting between a write's check and its cache write.
//!
//! 2. `HydrationManager::is_hydrating(ino)` — live `DashMap` lookup that
//!    `FuseHandler::write()` consults to decide between `EBUSY` (active
//!    hydration) and proceeding with the cache write.
//!
//! The exact `libc::EBUSY` return is verified by code review of
//! `filesystem.rs::write()`; we cannot drive `LnxDriveFs::write()` directly
//! from a unit test because `fuser::ReplyWrite` has no public constructor —
//! exercising the full callback would require standing up a real FUSE mount.

use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use lnxdrive_cache::pool::DatabasePool;
use lnxdrive_core::domain::{ItemState, RemoteId, UniqueId};
use lnxdrive_fuse::write_serializer::WriteSerializer;
use lnxdrive_fuse::{
    inode::InodeTable,
    inode_entry::{InodeEntry, InodeNumber},
    ContentCache, HydrationManager,
};
use lnxdrive_graph::{client::GraphClient, provider::GraphCloudProvider};
use tempfile::tempdir;
use tokio::runtime::Handle;

fn make_test_entry(ino: u64, name: &str) -> InodeEntry {
    InodeEntry::new(
        InodeNumber::new(ino),
        UniqueId::new(),
        Some(RemoteId::new(format!("remote_{}", ino)).unwrap()),
        InodeNumber::new(1),
        name.to_string(),
        fuser::FileType::RegularFile,
        10 * 1024 * 1024, // 10 MB placeholder (SIM-L2-002 uses 100 MB; 10 MB is sufficient)
        0o644,
        SystemTime::now(),
        SystemTime::now(),
        SystemTime::now(),
        1,
        ItemState::Online,
    )
}

/// Verifies that `InodeEntry::lock_state_guard()` provides genuine mutual
/// exclusion across threads. This is the primitive RISK-003 relies on.
#[test]
fn state_guard_provides_mutual_exclusion() {
    let entry = Arc::new(make_test_entry(42, "test.bin"));
    let entry_clone = Arc::clone(&entry);

    // Hold the guard on the main thread, then spawn a thread that tries to
    // acquire it. The thread should block until we release.
    let guard = entry.lock_state_guard();

    let (tx, rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let _g = entry_clone.lock_state_guard();
        tx.send(Instant::now()).unwrap();
    });

    // The child thread should NOT have acquired the lock yet.
    thread::sleep(Duration::from_millis(50));
    assert!(
        rx.try_recv().is_err(),
        "child thread acquired the lock while main thread held it"
    );

    let release_time = Instant::now();
    drop(guard);

    // After release, child thread must acquire promptly.
    let acquired_time = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("child thread never acquired lock after release");
    handle.join().unwrap();

    assert!(
        acquired_time >= release_time,
        "child thread reported acquisition before release"
    );
}

/// Verifies the RISK-003 contract end-to-end at the HydrationManager level:
/// after `test_register_active(ino)` returns, `is_hydrating(ino)` returns
/// `true` — meaning a concurrent FUSE write that re-checks `is_hydrating`
/// under the inode lock will observe the hydration and return `EBUSY`.
#[tokio::test]
async fn hydration_registration_makes_is_hydrating_true() {
    let temp = tempdir().unwrap();
    let cache = Arc::new(ContentCache::new(temp.path().to_path_buf()).unwrap());
    let pool = DatabasePool::in_memory().await.unwrap();
    let (serializer, write_handle) = WriteSerializer::new(pool.clone());
    tokio::spawn(async move { serializer.run().await });

    let inode_table = Arc::new(InodeTable::new());
    let entry = make_test_entry(99, "concurrent.bin");
    let item_id = *entry.item_id();
    let remote_id = entry.remote_id().unwrap().clone();
    let total_size = entry.size();
    inode_table.insert(entry);

    // GraphCloudProvider with a dummy token — the test exercises only the
    // lock + active-map paths, never reaching real Graph API calls.
    let client = GraphClient::new("test_dummy_token");
    let provider = Arc::new(GraphCloudProvider::new(client));

    let hm = HydrationManager::new(
        4,
        cache,
        write_handle,
        provider,
        Handle::current(),
        Arc::clone(&inode_table),
    );

    assert!(
        !hm.is_hydrating(99),
        "fresh manager must report not hydrating"
    );

    hm.test_register_active(99, item_id, remote_id, total_size);

    assert!(
        hm.is_hydrating(99),
        "after registration, is_hydrating must report true — this is the \
         signal a concurrent FuseHandler::write() observes to return EBUSY"
    );

    hm.test_unregister_active(99);
    assert!(!hm.is_hydrating(99));
}

/// Verifies that `HydrationManager::test_register_active` acquires the per-inode
/// state_guard, blocking concurrent FUSE-write-style critical sections — the
/// RISK-003 atomicity property.
///
/// `clippy::await_holding_lock` is suppressed because the test is *intentionally*
/// holding the lock across an await to demonstrate that the lock blocks
/// concurrent registration; the main `tokio::time::sleep` is only used to give
/// the spawned task a window to attempt acquisition.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn hydration_registration_serializes_with_inode_lock() {
    let temp = tempdir().unwrap();
    let cache = Arc::new(ContentCache::new(temp.path().to_path_buf()).unwrap());
    let pool = DatabasePool::in_memory().await.unwrap();
    let (serializer, write_handle) = WriteSerializer::new(pool.clone());
    tokio::spawn(async move { serializer.run().await });

    let inode_table = Arc::new(InodeTable::new());
    let entry = make_test_entry(123, "race.bin");
    let item_id = *entry.item_id();
    let remote_id = entry.remote_id().unwrap().clone();
    let total_size = entry.size();
    inode_table.insert(entry);

    let client = GraphClient::new("test_dummy_token");
    let provider = Arc::new(GraphCloudProvider::new(client));

    let hm = Arc::new(HydrationManager::new(
        4,
        cache,
        write_handle,
        provider,
        Handle::current(),
        Arc::clone(&inode_table),
    ));

    // Simulate a FUSE write holding the inode lock.
    let entry_arc = inode_table.get(123).expect("entry inserted above");
    let write_guard = entry_arc.lock_state_guard();

    // Concurrent hydration registration must block on the same lock.
    let hm_clone = Arc::clone(&hm);
    let registration = tokio::task::spawn_blocking(move || {
        let before = Instant::now();
        hm_clone.test_register_active(123, item_id, remote_id, total_size);
        before.elapsed()
    });

    // Give the spawned task time to attempt acquisition.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        !hm.is_hydrating(123),
        "hydration must NOT be registered while inode lock is held by simulated FUSE write"
    );

    // Release the lock; registration should complete promptly.
    drop(write_guard);

    let elapsed = registration.await.expect("registration task panicked");
    assert!(
        elapsed >= Duration::from_millis(50),
        "registration completed too fast — lock was not actually contended (elapsed={:?})",
        elapsed
    );
    assert!(
        hm.is_hydrating(123),
        "after lock release, hydration registration must complete and be visible"
    );
}
