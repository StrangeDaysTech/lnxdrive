---
id: AILOG-2026-05-31-001
title: Fase 2 — FUSE functional hardening + T101 performance validation
status: draft
created: 2026-05-31
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [fuse, readdir, inode, performance, charter-01, t101, files-on-demand, robustness]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-05-28-001
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security]
iso_42001_clause: [8]
---

# AILOG: Fase 2 — FUSE functional hardening + T101

## Summary

Closes T101 (the one remaining task of `002-files-on-demand`, the sole declared
work item of Charter-01 Fase 2): validate FUSE performance — `getattr` < 1ms,
`readdir` < 10ms for 1000 entries, idle memory < 50MB with 10k tracked files.

Implementing T101 as a real-mount integration test
(`crates/lnxdrive-fuse/tests/integration_perf_t101.rs`) made it the **first
end-to-end exercise of a real FUSE mount in the codebase** — every prior FUSE
test exercises internal primitives, and the doc-tests that mount are ignored in
CI for lack of `/dev/fuse`. That first real mount surfaced a chain of four
functional bugs that made directory listing non-functional; each is fixed here,
with CI-runnable regression tests, before the performance numbers could even be
taken.

**T101 result (real mount, this PR):**

| Metric | Target | Measured |
|---|---|---|
| `getattr` (lookup+getattr upper bound) | < 1 ms | 43.7 µs |
| `readdir`, 1000 entries | < 10 ms | 1.40 ms |
| idle RSS, 10k tracked files | < 50 MB | 37.9 MB |

## Context

The Charter scoped Fase 2 as: close T101, remove `~4 todo!()/unimplemented!()`
sites and `~10 debug println!`, and enable `cargo test --workspace` in CI. An
audit of `main` at the start of this work found the latter three **already
done during Fase 1** (zero `todo!`/`unimplemented!` in crates, zero debug
`println!` outside the CLI, `cargo test --workspace` live at
`.github/workflows/engine-ci.yml:66`). T101 was the only real remaining work —
recorded in the Charter telemetry below so the declared-vs-actual gap is explicit
([[feedback-strict-governance]]).

Per the operator's choice, T101 is validated with an `#[ignore]` integration
test that mounts a real FUSE filesystem (needs `/dev/fuse` + `fusermount3`; CI
has neither, so it is a local gate) backed by a file-based DB populated with N
entries, then issues real syscalls and times them. A `multi_thread` runtime is
mandatory: FUSE `init()` runs on fuser's own OS thread and `block_on`s DB work
served by the `WriteSerializer`; a `current_thread` runtime deadlocks while the
test thread blocks in `read_dir`/`stat` syscalls.

## The bug chain (discovered by the T101 mount)

1. **`init()` panics on every mount — `tokio::spawn` with no runtime.**
   `init()` (`filesystem.rs:554`) calls `DehydrationManager::start_periodic()`,
   which does `tokio::spawn` (`dehydration.rs:430`). `init()` runs on fuser's
   thread, which has no Tokio runtime entered, so the spawn panics with *"there
   is no reactor running"*. The daemon mounts identically
   (`main.rs:276` → `spawn_mount2`), so **auto-mount was broken** — undetected
   because no test mounted for real. **Fix:** enter the runtime already held in
   `self.rt_handle` for the duration of the spawn.

2. **`children()` lists the root as its own child → empty `ls`.**
   The root entry is built with `parent_ino == ino == 1`, so `children(1)`
   matched it. With an empty name (`String::new()`), `reply.add` stalls `readdir`
   right after `.`/`..`, so only `.`/`..` are returned — which `read_dir` filters
   out, yielding **zero entries**. **Fix:** `children()` excludes the entry whose
   `ino == parent_ino`. This was the root cause of the empty/partial listings.

3. **`readdir` paginates over an unstable order → large `ls` loses files.**
   `children()` collected from `DashMap::iter()`, whose order is not stable
   between calls. `readdir` pages by positional offset across multiple kernel
   calls, so the reshuffle between pages skipped/duplicated entries
   non-deterministically (1000-entry dirs lost files at random). **Fix:** sort
   `children()` by inode for a deterministic cross-page order.

4. **`opendir` returns `FOPEN_KEEP_CACHE` on a dynamic directory.**
   That flag tells the kernel its cached listing is still valid, so after the
   first open the kernel serves the cached page and stops issuing `readdir`. The
   listing is lazily populated and dynamic (the sync engine mutates it), so the
   kernel must re-issue `readdir` each open. **Fix:** `reply.opened(fh, 0)`.

5. **Inode not persisted across save — unstable inodes between mounts.**
   `save_item`'s `INSERT OR REPLACE` omitted the `inode` column (reset to NULL on
   every re-save) and `sync_item_from_row` never read it back, so the filesystem
   re-allocated a fresh inode for every item on every mount. **Fix:** `save_item`
   preserves the inode (its own, else the stored one, mirroring the `account_id`
   handling) and `sync_item_from_row` reads the column back via `set_inode`.

**Not a bug — discarded:** an early read suggested `get_next_inode` did a
non-atomic `SELECT`+`UPDATE`. It already wraps both in a transaction
(`repository.rs:1023` `begin()`…`commit()`); no change made. The duplicate-inode
symptom was bug #2 (inodes never read back) compounded by #3.

## Change

### Code

- **`crates/lnxdrive-fuse/src/filesystem.rs`** — `init()` enters `rt_handle`
  before `start_periodic()` (#1); `opendir` replies with no cache flags (#4).
- **`crates/lnxdrive-fuse/src/inode.rs`** — `children()` excludes the
  self-referential root and sorts by inode (#2, #3).
- **`crates/lnxdrive-cache/src/repository.rs`** — `save_item` persists/preserves
  the `inode` column; `sync_item_from_row` reads it back (#5).

### Tests

- **`crates/lnxdrive-fuse/tests/integration_perf_t101.rs`** (new, `#[ignore]`) —
  the two T101 tests (latency + idle memory) over a real mount. Configurable via
  `LNXDRIVE_PERF_N`.
- **`crates/lnxdrive-fuse/src/inode.rs`** — two CI-runnable regression tests:
  `test_children_excludes_self_referential_root` (#2) and
  `test_children_stable_sorted_order` (#3). These run without `/dev/fuse`, so the
  functional contract is guarded in CI even though the mount test is not.
- **`crates/lnxdrive-cache/tests/repository_tests.rs`** —
  `test_save_item_preserves_and_reads_back_inode` (#5).

## Verification

```bash
cd lnxdrive-engine

# CI-runnable regression tests (no /dev/fuse needed)
cargo test -p lnxdrive-fuse --lib inode::tests          # incl. the two children regressions
cargo test -p lnxdrive-cache --test repository_tests inode

# Full workspace — no regressions
cargo test --workspace                                  # all green
cargo clippy -p lnxdrive-fuse -p lnxdrive-cache --all-targets -- -D warnings

# T101 performance gate (LOCAL ONLY — needs /dev/fuse + fusermount3)
cargo test -p lnxdrive-fuse --test integration_perf_t101 -- --ignored --nocapture
# Expected: readdir ~1.4ms/1000, getattr ~44µs, idle RSS ~38MB/10k — all under target.
```

## Drift

- **Fase 2 scope was 4 items; 3 were already done in Fase 1.** Only T101
  remained. The Charter `## Scope` Fase 2 row is updated to reflect this
  (declared-vs-actual, per [[feedback-strict-governance]]).
- **Scope grew from "measure performance" to "fix the FUSE listing path".** T101
  could not run until bugs #1–#4 were fixed; #5 was fixed in the same pass as it
  is the same subsystem and a real correctness defect. Approved by the operator
  as Fase 2 = FUSE functional hardening.
- **Idle-memory metric implemented as a Rust test, not a `lnxdrive-testing`
  shell script.** Reading `/proc/self/statm` in the existing integration test is
  more robust and CI-consistent than standing up the full daemon (which needs
  GOA auth) from bash, and reuses the mount setup. The measured process RSS is a
  conservative upper bound on the daemon's tracked-file footprint.
- **T101 latency is measured as observed syscall latency**, which includes the
  unavoidable FUSE round-trip and (for `getattr`) the preceding `lookup`. The
  reported `getattr` figure is therefore an upper bound on `getattr` alone; it is
  well under target regardless.

## Risk

Per-bug regression surface (all on the FUSE read/mount path, all now covered by
tests):

- **#1 (init runtime enter).** Low. `Handle::enter()` returns a guard scoped to
  the spawn; the spawned task outlives the guard correctly. Without it, no mount
  worked at all, so this strictly restores function.
- **#2 (root self-exclusion).** Low. The filter only ever removes the single
  self-referential entry (the root); regular nested dirs are unaffected
  (`ino != parent_ino` for them). Covered by a unit test.
- **#3 (sorted children).** Low. Sorting is O(n log n) per `readdir`; for a
  10k-entry directory this is negligible against the syscall cost, and T101's
  1.40ms/1000 confirms headroom. Determinism is required for correctness, not
  just tidiness.
- **#4 (no dir cache).** Low–medium. Dropping `FOPEN_KEEP_CACHE` means the kernel
  re-issues `readdir` per open instead of serving a cached page; correct for a
  dynamic directory and measured fast. A future `notify_inval_entry`-based cache
  invalidation could re-enable caching safely — deferred polish, not debt.
- **#5 (inode persistence).** Low. `save_item` now writes one more column and
  preserves the existing value; `from_row` sets it post-deserialization without
  touching the serde representation. Covered by a round-trip test.

No emergent risks. `cargo test --workspace` is green and clippy is clean.

## Telemetry

| Metric | Estimated | Actual |
|---|---|---|
| Effort | ~0.5 day (T101 only) | ~1 day (T101 + 5 bug fixes) |
| Lines added | ~120 (perf test) | ~360 (perf test + fixes + regressions + AILOG) |
| Lines removed | ~0 | ~10 |
| New files | 1 (perf test) | 2 (perf test, AILOG) |
| Bugs found | 0 (validation only) | 5 functional (4 fixed on the listing path + 1 inode) |
| Existing tests broken | 0 | 0 |
| Tests added | T101 | 2 T101 (ignore) + 3 CI regressions |
| Pre-commit hook failures | n/a | none |
