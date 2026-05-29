---
id: AILOG-2026-05-28-002
title: Mitigate RISK-001 — D-Bus session bus health monitor + reconnect
status: accepted
created: 2026-05-28
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [availability, dbus, reconnection, single-point-of-failure, charter-01, risk-001, sim-l1-001]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-05-28-001
  - AILOG-2026-05-29-002
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security]
iso_42001_clause: [8]
---

# AILOG: Mitigate RISK-001 — D-Bus session bus health monitor

## Summary

Mitigates RISK-001 (A1/D1 "D-Bus SPOF", SIM-L1-001, P0) — the session bus is
the only channel between the daemon and every UI client, and the daemon held
its `zbus::Connection` in a fire-and-forget local binding (`_dbus_connection`)
with **no detection of loss and no reconnection**. If the session bus restarted
(logout/login of the bus, `systemctl --user restart`, OOM of `dbus-daemon`), the
daemon kept running headless — serving nothing — until manually restarted.

The mitigation adds a supervised, self-healing connection:

1. **`lnxdrive-daemon/src/health.rs`** (new) — a background task that *owns* the
   connection and runs a two-phase loop: a **healthy phase** that actively
   probes the live bus with `zbus::fdo::DBusProxy::get_id()` (wrapped in a
   `tokio::time::timeout`, since zbus 4.x exposes no "connection closed"
   future), and a **reconnect phase** that, on a failed/timed-out probe, drops
   the dead connection and re-runs `DbusService::start()` — re-registering all
   nine interfaces and re-acquiring the well-known name — with exponential
   backoff + jitter.

2. **Single-instance safety preserved.** If a reconnect attempt fails because
   another `lnxdrived` acquired the name during our outage, the monitor does
   **not** fight for it: it logs, sets D-Bus health to `lost`, and triggers a
   graceful shutdown (`CancellationToken::cancel`). At most one daemon ever owns
   the name.

3. **UI visibility.** A new `DaemonState::dbus_health` field
   (`"online"|"reconnecting"|"lost"`) plus a read-only `dbus_health` property on
   `StatusInterface`, kept **distinct** from the existing `connection_status`
   (which tracks cloud/OneDrive network health, an orthogonal failure mode).

Per Charter-01, the full Unix-socket fallback stays **out of scope** (deferred
to v0.2); this is the health-monitor-and-reconnect slice only.

## Context

`RISK-001-critical-paths.md` flags SPOF-001 as CRITICAL: a single point of
failure on the session bus with no recovery path. The audit on 2026-05-28 (per
[[feedback-validate-before-security-code]]) confirmed the current state:

- `DbusService::start()` (`service.rs:1178`) builds the connection via
  `zbus::connection::Builder::session().name(DBUS_NAME).serve_at(...).build()`
  and returns it; `main::run()` bound it to `_dbus_connection` purely to keep it
  alive for the lifetime of `sync_loop`. Nothing observed the connection.
- A grep for `reconnect`/`health`/`NameLost`/`monitor` across the daemon and IPC
  crates returned only passive comments. No recovery logic existed.
- zbus **4.4.0** has no passive closure signal (`is_closed()`/`closed()` are
  5.x). Detection must therefore be **active probing**. `DBusProxy::get_id()` is
  a cheap real round-trip to `org.freedesktop.DBus`; a transport error or a
  timeout on it is a reliable "bus is gone" signal.

Two operator decisions shaped the scope, both following
[[feedback-minimum-viable-plus-tde]]:

- **Active probe, no `NameLost` fast-path.** Subscribing to the `NameLost`
  signal would shave up to one `probe_interval` (5 s) off detection latency but
  requires a `Stream` adapter (`futures-util`/`tokio-stream`) as a new direct
  dependency. For a session-bus recovery scenario a ≤5 s detection window is
  immaterial, so the periodic probe is the whole mechanism. The fast-path is
  recorded as deferrable polish, not debt that blocks anything.
- **Monitor owns the connection.** Because the connection is now *replaced* over
  time, a stack-local binding in `run()` is wrong. The monitor task is the sole
  owner; `run()` keeps only the `JoinHandle` and awaits it at shutdown. The
  alternative (`Arc<Mutex<Option<Connection>>>` shared with `run()`) was
  rejected — `sync_loop` never touches the connection, so a shared lock would
  guard a value nobody else reads.

## Change

### Code

- **`crates/lnxdrive-daemon/src/health.rs`** (new) — the monitor module:
  - `HealthConfig` (probe 5 s, probe-timeout 2 s, backoff 0.5 s→30 s ×2, ±20 %
    jitter) with production `Default`.
  - Pure, bus-free helpers: `backoff_delay()` (geometric, clamped),
    `apply_jitter()` (symmetric, RNG sample injected for determinism),
    `classify_probe()`, and `is_name_taken_error()` — the latter extracted from
    the string match previously inlined in `main::run` so both call sites share
    one definition (kept daemon-local to avoid touching the IPC crate for this).
  - `DbusHealth { Online, Reconnecting, Lost }` ↔ string mapping.
  - `spawn_health_monitor(Arc<DbusService>, Connection, Arc<Mutex<DaemonState>>,
    CancellationToken, HealthConfig) -> JoinHandle<()>` driving the two-phase
    `monitor_loop`. Reconnect attempts `start()` directly and classifies the
    error rather than pre-probing with `try_acquire_name()` (which would open a
    TOCTOU window — the bus arbitrates name ownership atomically in `build()`).

- **`crates/lnxdrive-daemon/src/main.rs`** — integration:
  - `mod health;`.
  - `DbusService` is now wrapped in `Arc` (it takes `&self` in `start()`, so no
    `Clone` impl is needed on the struct → no `service.rs` change for ownership).
  - The single-instance bail-out at startup is unchanged in behaviour but now
    uses the shared `health::is_name_taken_error(&e)` instead of an inline
    string match.
  - The initial connection is handed to `spawn_health_monitor`; the body after
    D-Bus startup (account/token load → SyncEngine → FUSE → `sync_loop`) is
    extracted verbatim into a new `run_inner()` so the monitor `JoinHandle` is
    awaited at one common exit point regardless of which early return
    (`wait_for_auth_loop`) fires.

- **`crates/lnxdrive-ipc/src/service.rs`** — UI-visible state:
  - New `DaemonState::dbus_health: String` (default `"online"`).
  - New read-only `#[zbus(property)] dbus_health` on `StatusInterface`, mirroring
    `connection_status`.

### Tests

- **`crates/lnxdrive-daemon/src/health.rs` unit tests** (8) cover the genuinely
  testable core without a bus: backoff geometry + clamp, jitter identity at the
  midpoint / bounds / symmetry at the extremes, `DbusHealth` string mapping,
  `classify_probe`, and `is_name_taken_error` classification (known strings vs.
  unrelated errors).
- **Kill-the-bus path (`test_dbus_reconnect_after_crash`) is verified by a
  documented manual smoke**, not an automated test. A faithful automated test
  must spawn a private `dbus-daemon --session`, kill and restart it, and assert
  re-registration — inherently flaky and non-portable in shared CI. The
  procedure is recorded in the Verification section below; standing up a
  reusable harness for it is future work for `lnxdrive-testing/`, consistent
  with the same trade-off accepted for SIM-L2-002 in [[AILOG-2026-05-28-001]].

### Governance

- **Charter `## Files to modify`** — the `RISK-001` row (which named only
  `health.rs`) is rewritten atomically in this PR to also list
  `lnxdrive-daemon/src/main.rs` (integration) and `lnxdrive-ipc/src/service.rs`
  (the `dbus_health` state field + property), per the atomic-update discipline
  in [[feedback-strict-governance]].

## Verification

```bash
cd lnxdrive-engine

# Unit tests added in this PR
cargo test -p lnxdrive-daemon health::
# Expected: 8 passed; 0 failed.

# Affected crates build/lint/test clean
cargo clippy -p lnxdrive-daemon -p lnxdrive-ipc --all-targets -- -D warnings
cargo test -p lnxdrive-daemon -p lnxdrive-ipc      # 14 + 71 passed

# Full workspace — no regressions from the new DaemonState field
cargo test --workspace
# Expected: all green (the historically cwd-sensitive config::default_path test
# was fixed in bbe221a and now passes).
```

Manual smoke (requires a private session bus; not part of `cargo test`):

```bash
# 1. Start a throwaway session bus and the daemon against it.
export DBUS_SESSION_BUS_ADDRESS="$(dbus-daemon --session --print-address --fork)"
RUST_LOG=info ./target/debug/lnxdrived &        # logs "D-Bus health monitor started"

# 2. Confirm the name is held.
busctl --user list | grep com.strangedaystech.LNXDrive

# 3. Kill the bus, then start a fresh one at the SAME address.
#    Within a few backoff cycles the daemon logs
#    "D-Bus service re-registered after bus recovery" and the name reappears.
```

Governance:

```bash
straymark validate
straymark charter status CHARTER-01-road-to-v0-1-0-alpha-1
straymark charter drift CHARTER-01-road-to-v0-1-0-alpha-1 origin/main..HEAD
```

## Drift

- **R7 (new, not in Charter)** — The Charter's `## Files to modify` entry for
  RISK-001 named only `lnxdrive-daemon/src/health.rs`. The integration
  necessarily also touches `lnxdrive-daemon/src/main.rs` (wrap `DbusService` in
  `Arc`, spawn the monitor, split `run`/`run_inner`, share
  `is_name_taken_error`) and — **cross-crate** — `lnxdrive-ipc/src/service.rs`
  for the `dbus_health` state field + property. Charter `## Files to modify` row
  updated atomically in this PR.
- **NameLost fast-path omitted** — deferred to avoid a new direct dependency;
  detection relies solely on the 5 s active probe. Recorded as deferrable polish
  (v0.2), not a tracked debt.
- **Automated kill-the-bus test omitted** — replaced by the documented manual
  smoke above; reusable harness deferred to `lnxdrive-testing/`.

## Risk

This is an additive supervision layer around an existing, working connection
path. Regression surface considered:

- **R1 — Deadlock / task stall.** Low. The monitor is a single task; it holds no
  lock across `.await` beyond the connection it owns, and the `select!` arms all
  watch `shutdown.cancelled()`, so cancellation always wins. `run` cancels the
  token and awaits the handle on every exit path.
- **R2 — Probe overhead.** Negligible. One `get_id()` round-trip every 5 s on
  the session bus; bounded by a 2 s timeout so a hung bus is detected, not
  waited on indefinitely.
- **R3 — Reconnect storm across many user sessions.** Mitigated by ±20 % jitter
  on every backoff delay, de-synchronising daemons that all reconnect to a
  freshly-restarted bus.
- **R4 — Single-instance invariant.** Preserved: startup acquisition is
  unchanged, and on a contested reconnect the older instance yields rather than
  busy-looping for the name. No `try_acquire_name` pre-check (avoids TOCTOU).
- **R5 — Detection latency up to 5 s.** Accepted: a multi-second gap before a
  bus-restart is noticed is immaterial for a background sync daemon, and the
  `NameLost` fast-path that would shorten it was traded away for dependency
  minimalism.

No emergent risks beyond R7 above.

## Telemetry

| Metric | Estimated | Actual |
|---|---|---|
| Effort | 1.5 days | ~0.5 day |
| Lines added | ~200 | ~330 (incl. tests + AILOG) |
| Lines removed | ~15 | ~10 |
| New files | 2 (health.rs, AILOG) | 2 |
| Existing tests broken | 0 | 0 |
| Tests added | unit backoff/jitter | 8 unit |
| Pre-commit hook failures | n/a | none |
