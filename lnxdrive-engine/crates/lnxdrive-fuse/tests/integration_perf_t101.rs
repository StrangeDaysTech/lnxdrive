//! T101 — performance validation (Charter-01, Fase 2).
//!
//! Mounts a *real* FUSE filesystem backed by an in-memory database populated
//! with `N_ENTRIES` items directly under the mount root, then measures the
//! latency a user actually observes for `getattr` and `readdir` by issuing
//! real syscalls against the mountpoint.
//!
//! T101 targets (from `specs/002-files-on-demand/tasks.md`):
//!   * `getattr`  completes in < 1ms
//!   * `readdir`  completes in < 10ms for 1000 entries
//!   * idle memory < 50MB with 10k tracked files (covered separately by
//!     `lnxdrive-testing/scripts/perf-idle-memory-t101.sh`, not a Rust test)
//!
//! Marked `#[ignore]` because it mounts FUSE, which requires `/dev/fuse` and
//! `fusermount3` — CI runners have no FUSE device, so this is a *local* gate.
//! Run it with:
//!
//! ```sh
//! cargo test -p lnxdrive-fuse --test integration_perf_t101 -- --ignored --nocapture
//! ```
//!
//! A `multi_thread` runtime is mandatory: the FUSE `init()` callback runs on
//! fuser's own OS thread and `block_on`s database work served by the
//! `WriteSerializer` task. On a `current_thread` runtime that work could not
//! make progress while the test thread is blocked in `read_dir`/`stat`
//! syscalls, deadlocking the mount.

use std::{
    fs,
    path::Path,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use fuser::MountOption;
use lnxdrive_cache::{DatabasePool, SqliteStateRepository};
use lnxdrive_core::{
    config::FuseConfig,
    domain::{
        newtypes::{Email, RemoteId, RemotePath, SyncPath},
        Account, SyncItem,
    },
    ports::IStateRepository,
};
use lnxdrive_fuse::{ContentCache, LnxDriveFs};
use tempfile::tempdir;
use tokio::runtime::Handle;

/// Directory entries used for the readdir/getattr measurements. T101 specifies
/// the 1000-entry readdir target explicitly; override with `LNXDRIVE_PERF_N`
/// when iterating locally.
const N_ENTRIES_DEFAULT: usize = 1000;

fn n_entries() -> usize {
    std::env::var("LNXDRIVE_PERF_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(N_ENTRIES_DEFAULT)
}

/// Populates `repo` with one account and `n` regular files, all direct children
/// of `mount_root` so they hang off the FUSE root inode (ino = 1). An account
/// must exist first: `save_item` associates each item with the default account.
async fn populate_db(repo: &SqliteStateRepository, mount_root: &Path, n: usize) {
    let email = Email::new("perf@example.com".to_string()).unwrap();
    let sync_root = SyncPath::new(mount_root.to_path_buf()).unwrap();
    let account = Account::new(email, "Perf Test", "drive_perf", sync_root);
    repo.save_account(&account).await.unwrap();

    for i in 0..n {
        let local_path = SyncPath::new(mount_root.join(format!("file_{i:05}.txt"))).unwrap();
        let remote_path = RemotePath::new(format!("/file_{i:05}.txt")).unwrap();
        let remote_id = RemoteId::new(format!("remote_file_{i:05}")).unwrap();
        let mut item =
            SyncItem::new_file(local_path, remote_path, 4096, Some("text/plain".to_string()))
                .unwrap();
        item.set_remote_id(remote_id);
        repo.save_item(&item).await.unwrap();
        // Persist a unique inode (ino 1 is the FUSE root), mirroring the state a
        // real sync leaves in the DB. `update_inode` is used rather than
        // `set_inode` + `save_item` because the INSERT path does not persist the
        // inode column; items left without one would be assigned via the inode
        // counter during init() instead.
        repo.update_inode(item.id(), (i as u64) + 2).await.unwrap();
    }
}

/// Blocks until the FUSE mount has finished `init()` and exposes at least
/// `expected` entries, or panics after a timeout. Also serves as a warm-up.
fn wait_for_mount(mount: &Path, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(rd) = fs::read_dir(mount) {
            if rd.filter_map(Result::ok).count() >= expected {
                return;
            }
        }
        assert!(
            Instant::now() < deadline,
            "FUSE mount did not expose {expected} entries within 10s"
        );
        thread::sleep(Duration::from_millis(50));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires /dev/fuse + fusermount3; run with --ignored"]
async fn t101_getattr_readdir_latency() {
    let n_entries = n_entries();
    let mount_dir = tempdir().unwrap();
    let cache_dir = tempdir().unwrap();
    let db_dir = tempdir().unwrap();
    let mount_root = mount_dir.path().to_path_buf();

    // Populate a file-backed DB with N entries, then mount FUSE on top of it.
    // A file-backed pool (max_connections = 5, like the daemon) is required:
    // `init()` issues a query plus one inode-counter increment per item, and
    // an in-memory pool (max_connections = 1) deadlocks under that reentrant
    // `block_on` load.
    let pool = DatabasePool::new(&db_dir.path().join("perf.db")).await.unwrap();
    let repo = SqliteStateRepository::new(pool.pool().clone());
    populate_db(&repo, &mount_root, n_entries).await;

    // Build and mount the filesystem directly (instead of the production
    // `lnxdrive_fuse::mount()` wrapper) so we can use minimal mount options.
    // The wrapper sets `AutoUnmount`, which libfuse couples with `allow_other`
    // and therefore needs `user_allow_other` in /etc/fuse.conf — a system
    // change we must not require from a test. Dropping the `BackgroundSession`
    // still unmounts cleanly here.
    let config = FuseConfig {
        mount_point: mount_root.to_string_lossy().into_owned(),
        cache_dir: cache_dir.path().to_string_lossy().into_owned(),
        ..Default::default()
    };
    let cache = Arc::new(ContentCache::new(cache_dir.path().to_path_buf()).unwrap());
    let fs = LnxDriveFs::new(Handle::current(), pool, config, cache, None);
    let session = fuser::spawn_mount2(
        fs,
        &mount_root,
        &[MountOption::FSName("lnxdrive-perf".to_string()), MountOption::RO],
    )
    .expect("FUSE mount failed (is /dev/fuse available and fusermount3 installed?)");

    // Ensure init() has loaded every entry before we start timing.
    wait_for_mount(&mount_root, n_entries);

    // --- readdir: full listing of the root directory (n_entries children) ---
    // Report the best of several runs as the steady-state latency.
    let mut best_readdir = Duration::MAX;
    for _ in 0..10 {
        let start = Instant::now();
        let count = fs::read_dir(&mount_root).unwrap().filter_map(Result::ok).count();
        let elapsed = start.elapsed();
        assert_eq!(count, n_entries, "readdir returned {count} entries, expected {n_entries}");
        best_readdir = best_readdir.min(elapsed);
    }

    // --- getattr: cold stat of each distinct file ---
    // Each cold stat pays a FUSE lookup *plus* a getattr, and the files are all
    // distinct so the kernel attr cache (TTL = 1s) is never reused. The mean
    // per-file time is therefore a conservative UPPER BOUND on getattr alone.
    let paths: Vec<_> = fs::read_dir(&mount_root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    assert_eq!(paths.len(), n_entries);

    let start = Instant::now();
    for p in &paths {
        let _ = fs::metadata(p).unwrap();
    }
    let total_getattr = start.elapsed();
    let mean_getattr = total_getattr / paths.len() as u32;

    println!("T101 readdir ({n_entries} entries): best = {best_readdir:?}");
    println!(
        "T101 getattr (lookup+getattr upper bound, mean over {} cold stats): {mean_getattr:?} \
         (total {total_getattr:?})",
        paths.len()
    );

    // Unmount before the tempdirs are torn down.
    drop(session);

    assert!(
        best_readdir < Duration::from_millis(10),
        "readdir for {n_entries} entries took {best_readdir:?}, T101 target < 10ms"
    );
    assert!(
        mean_getattr < Duration::from_millis(1),
        "mean getattr (incl. lookup) was {mean_getattr:?}, T101 target < 1ms"
    );
}

/// Resident set size of the current process, in bytes, read from
/// `/proc/self/statm` (field 2 = resident pages × page size).
fn read_rss_bytes() -> u64 {
    let statm = fs::read_to_string("/proc/self/statm").unwrap();
    let resident_pages: u64 = statm.split_whitespace().nth(1).unwrap().parse().unwrap();
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
    resident_pages * page_size
}

/// T101 — idle memory must stay under 50 MB with 10k tracked files.
///
/// Mounts the filesystem backed by 10 000 entries and measures the process RSS
/// while the mount sits idle. This is a conservative upper bound on the daemon's
/// footprint: the whole test process (Tokio runtime, test harness, the FUSE
/// state) is measured, so the real daemon's tracked-file overhead is no larger.
/// `#[ignore]` for the same reason as the latency test (needs `/dev/fuse`).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires /dev/fuse + fusermount3; run with --ignored"]
async fn t101_idle_memory_10k() {
    const N: usize = 10_000;
    let mount_dir = tempdir().unwrap();
    let cache_dir = tempdir().unwrap();
    let db_dir = tempdir().unwrap();
    let mount_root = mount_dir.path().to_path_buf();

    let pool = DatabasePool::new(&db_dir.path().join("perf.db")).await.unwrap();
    let repo = SqliteStateRepository::new(pool.pool().clone());
    populate_db(&repo, &mount_root, N).await;

    let config = FuseConfig {
        mount_point: mount_root.to_string_lossy().into_owned(),
        cache_dir: cache_dir.path().to_string_lossy().into_owned(),
        ..Default::default()
    };
    let cache = Arc::new(ContentCache::new(cache_dir.path().to_path_buf()).unwrap());
    let fs = LnxDriveFs::new(Handle::current(), pool, config, cache, None);
    let session = fuser::spawn_mount2(
        fs,
        &mount_root,
        &[MountOption::FSName("lnxdrive-perf".to_string()), MountOption::RO],
    )
    .expect("FUSE mount failed (is /dev/fuse available and fusermount3 installed?)");

    wait_for_mount(&mount_root, N);

    // Let the mount settle, then sample RSS while idle.
    thread::sleep(Duration::from_millis(500));
    let rss_bytes = read_rss_bytes();
    let rss_mb = rss_bytes as f64 / (1024.0 * 1024.0);

    println!("T101 idle memory with {N} tracked files: {rss_mb:.1} MB RSS");

    drop(session);

    assert!(
        rss_bytes < 50 * 1024 * 1024,
        "idle RSS with {N} tracked files was {rss_mb:.1} MB, T101 target < 50 MB"
    );
}
