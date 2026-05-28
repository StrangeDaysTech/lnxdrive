---
id: AILOG-2026-05-28-001
title: Mitigate RISK-003 — block FUSE writes during hydration with per-inode lock + EBUSY
status: accepted
created: 2026-05-28
agent: claude-opus-4-7-v1.0
confidence: high
review_required: true
risk_level: high
tags: [data-integrity, fuse, hydration, race-condition, charter-01, risk-003, sim-l2-002]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-05-29-002
  - AILOG-2026-02-05-009-implement-stage4-hydration
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security]
iso_42001_clause: [8]
---

# AILOG: Mitigate RISK-003 — FUSE write during hydration

## Summary

Closes GitHub issue [#7](https://github.com/StrangeDaysTech/lnxdrive/issues/7) /
RISK-003 (RACE-001, CRITICAL, P0) — concurrent writes against a file that is
being downloaded from OneDrive could corrupt the local cache by interleaving
application bytes with download chunks. The mitigation has two enforcement
points:

1. `InodeEntry::lock_state_guard()` — new `parking_lot::Mutex<()>` on the
   in-memory inode record. `FuseHandler::write()` acquires it across the
   `HydrationManager::is_hydrating(ino)` check and the cache write;
   `HydrationManager::hydrate()` acquires it briefly to insert into the
   active map atomically before any `.await`. The lock guarantees that any
   FUSE write seeing `is_hydrating == false` will complete before a
   subsequent hydration can register the same inode.

2. `FuseHandler::write()` now returns `libc::EBUSY` (was `libc::EIO`) when
   the hydration check fires, matching the SIM-L2-002 acceptance contract
   and POSIX "resource temporarily occupied" semantics.

The original Charter entry pointed `RISK-003` at
`lnxdrive-engine/crates/lnxdrive-fuse/src/write_serializer.rs` based on the
risk-analysis document. The audit performed today established that
`write_serializer.rs` was already implemented (it serializes DB writes via
`tokio::sync::mpsc`) and the actual data-integrity gap was in the FUSE
write path, not the DB write path. The Charter's `## Files to modify` table
is updated atomically in this PR to reflect the real surface.

## Context

`RISK-003-data-integrity.md` documents the race timeline in detail: a
hydration task downloads chunks `[0..1MB]`, `[1..2MB]`, `[2..3MB]` while an
application performs `write()` at offset `1.5MB`; the application's bytes
land in the cache file, then chunk `[2..3MB]` overwrites them, silently
losing the application's modification. The recommended mitigation is
**Option A: exclusive lock during hydration, return `EBUSY`** (rejected
Option B was copy-on-write with conflict markers — heavier and unnecessary
for the alpha).

The audit on 2026-05-28 (per [[feedback-validate-before-security-code]])
revealed:

- `FuseHandler::write()` at `filesystem.rs:1585` already branched on
  `ItemState::Hydrating` and returned `EIO`. The error code was wrong (the
  acceptance test demands `EBUSY`); more importantly, the guard relied on
  `InodeEntry.state` which is set at construction and **never updated**
  after — `WriteSerializer::update_state()` writes only to SQLite, not to
  the in-memory `InodeEntry`. The real, live signal for "this inode is
  being hydrated" is `HydrationManager::is_hydrating(ino)` which consults
  the `Arc<DashMap<u64, ActiveHydration>>` updated as hydrations are
  registered and cleaned up.
- `DehydrationManager` (at `dehydration.rs:302,497,506`) refuses to
  dehydrate any inode with `open_handles() > 0`, so under the current
  open/dehydrate flow the state cannot regress `Hydrated → Online →
  Hydrating` while a `write()` is in flight. The runtime invariant
  underlying the existing guard *does* hold today — but it is implicit and
  any future change to the open or dehydration flow could silently
  reintroduce the race.

The operator decision on 2026-05-28: **minimum viable + lock per-inode
now** (chosen over the equally-viable "minimum viable + TDE", per
[[feedback-minimum-viable-plus-tde]]) — fix the error code, plug the
TOCTOU window with an explicit per-inode mutex, and ship cobertura that
locks in the property.

## Change

### Code

- **`lnxdrive-engine/Cargo.toml` + `crates/lnxdrive-fuse/Cargo.toml`** —
  add `parking_lot = "0.12"` workspace dependency and crate alias.

- **`crates/lnxdrive-fuse/src/inode_entry.rs`** — new field
  `state_guard: parking_lot::Mutex<()>` on `InodeEntry`, exposed via
  `pub fn lock_state_guard(&self) -> MutexGuard<'_, ()>`. The lock is
  *separate from* the existing `state: ItemState` field (which remains
  the construction-time snapshot) because the goal is serialization with
  `HydrationManager::hydrate()`, not making the state value atomic.

- **`crates/lnxdrive-fuse/src/filesystem.rs::write()`** — two changes:
  (1) pre-lock fast-path `is_hydrating(ino)` check that returns `EBUSY`
  without acquiring the inode lock; (2) acquire `entry.lock_state_guard()`,
  re-check `is_hydrating(ino)` under the lock, then branch on
  `entry.state()`. The `Hydrating` arm now returns `EBUSY` (was `EIO`);
  the `Online` arm keeps `EIO` (different semantics: file isn't local at
  all, not "busy"). The cache write still happens under the lock so no
  hydration can register between the check and the write.

- **`crates/lnxdrive-fuse/src/hydration.rs`** — `HydrationManager` gains
  an `inode_table: Arc<InodeTable>` field (the daemon constructs both
  and wires them together; today no production code calls `new()` so the
  signature change has no callers to update). `hydrate()` is reordered:
  the `active.insert()` now happens **before** the `update_state(...)
  .await` and the spawn, under the per-inode `lock_state_guard`. The
  `_task_handle` field of `ActiveHydration` becomes `Option<JoinHandle>`
  so the entry can be inserted before the spawn returns the handle; the
  handle is patched in via `DashMap::get_mut` after the spawn. If
  `update_state` fails, the active-map registration is rolled back.

- **`crates/lnxdrive-fuse/src/hydration.rs`** — new `#[doc(hidden)]`
  helpers `test_register_active` / `test_unregister_active` that let
  integration tests exercise the active-map path without standing up a
  mocked `GraphCloudProvider`. They reuse the same per-inode lock
  acquisition pattern as `hydrate()`, so they are faithful witnesses of
  the production behaviour.

### Tests

- **`crates/lnxdrive-fuse/tests/integration_write_during_hydration.rs`**
  (new) — three tests:

  1. `state_guard_provides_mutual_exclusion` — proves the
     `lock_state_guard()` primitive is a real mutex across threads.
  2. `hydration_registration_makes_is_hydrating_true` — proves
     `test_register_active` flips the `is_hydrating(ino)` flag that
     `FuseHandler::write()` consults to return `EBUSY`.
  3. `hydration_registration_serializes_with_inode_lock` — proves a
     concurrent simulated FUSE write holding the inode lock blocks
     hydration registration; the spawned registration takes ≥50 ms when
     contended (matching the manual `sleep`), and `is_hydrating` flips
     to true only after the lock release.

- The SIM-L2-002 spec text demands a true integration test of
  `write_to_file(...).unwrap_err().raw_os_error() == Some(libc::EBUSY)`.
  `LnxDriveFs::write()` cannot be driven from a unit test because
  `fuser::ReplyWrite` has no public constructor — exercising the full
  callback would require a real FUSE mount with a mocked
  `GraphCloudProvider`. The three integration tests above verify the
  *property* that SIM-L2-002 is checking (the lock prevents the race;
  `is_hydrating` is the signal) and the `EBUSY` constant is enforced by
  the matched code in `filesystem.rs::write()`. Standing up the full
  FUSE-mount harness is tracked as future work for the test infrastructure
  in `lnxdrive-testing/`, not in scope for this PR.

### Governance

- **Charter `## Files to modify`** — the `RISK-003` row is rewritten to
  list the three real files (`inode_entry.rs`, `filesystem.rs`,
  `hydration.rs`) plus the new test, with a sentence explaining why the
  original entry (pointing at `write_serializer.rs`) was inaccurate. This
  is the atomic-update pattern from [[feedback-strict-governance]]: drift
  fixed in the same PR as the work, not deferred to a housekeeping PR.

## Verification

```bash
cd lnxdrive-engine

# Integration test (the test added in this PR)
cargo test -p lnxdrive-fuse --test integration_write_during_hydration
# Expected: 3 passed; 0 failed.

# Unit tests for the affected crate
cargo test -p lnxdrive-fuse --lib
# Expected: 172 passed; 0 failed.

# Full workspace
cargo test --workspace --no-fail-fast
# Expected: 1 failed = config::tests::default_path_ends_with_config_yaml
# (pre-existing, cwd-sensitive, documented in [[project-lnxdrive-stack]]).
```

Governance:

```bash
straymark validate
straymark charter status CHARTER-01-road-to-v0-1-0-alpha-1
```

## Drift

- **R6 (new, not in Charter)** — The Charter's original `## Files to
  modify` entry for RISK-003 named
  `lnxdrive-engine/crates/lnxdrive-fuse/src/write_serializer.rs` as a
  stub to implement. The audit revealed the file is fully implemented;
  the real change surface is in `inode_entry.rs`, `filesystem.rs`,
  `hydration.rs`, and the new `tests/integration_write_during_hydration.rs`.
  Charter `## Files to modify` row updated atomically in this PR.
- New file `crates/lnxdrive-fuse/tests/integration_write_during_hydration.rs`
  was not in the Charter's enumeration; per the R4 pattern documented in
  the Charter itself, it is listed in the updated `## Files to modify`
  row and called out here for the drift log.

## Risk

This is a defensive code change in the FUSE write path. The lock is
sync (`parking_lot::Mutex<()>`), held only across very short critical
sections (an `is_hydrating` `DashMap` lookup + an in-memory cache file
write), and never across an `.await`. Risk of new regressions:

- **R1 — Deadlock from misuse of `lock_state_guard`.** Low. The lock
  is acquired in two places (`FuseHandler::write` and
  `HydrationManager::hydrate`/`test_register_active`), both for short
  synchronous sections, both with `Drop`-based release. No nested
  acquisition. No cross-inode dependency.
- **R2 — Lock contention under heavy concurrent writes to the same
  inode.** Low. The protected section is microseconds (DashMap lookup
  + bounded memory write). Multi-process write-contention on the same
  inode is rare and inherently serialised by the underlying filesystem
  anyway.
- **R3 — The integration test exercises only the primitive, not the
  real `EBUSY` return path through `fuser::ReplyWrite`.** Accepted
  trade-off. A true end-to-end test would require mocking
  `GraphCloudProvider` and standing up a FUSE mount in a container —
  out of scope for RISK-003. The error code mapping is straightforward
  matched code in `filesystem.rs::write()` and is reviewable by eye.

No new emergent risks beyond R6 above.

## Telemetry

| Metric | Estimated | Actual |
|---|---|---|
| Effort | 2 days | ~1 day |
| Lines added | ~150 | ~210 (including tests + AILOG) |
| Lines removed | ~10 | ~20 |
| New files | 2 (test, AILOG) | 2 |
| Existing tests broken | 0 | 0 |
| Tests added | 1 integration | 3 integration |
| Pre-commit hook failures | n/a | none |
