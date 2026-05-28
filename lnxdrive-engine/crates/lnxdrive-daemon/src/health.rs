//! D-Bus session bus health monitor + reconnect (RISK-001 mitigation).
//!
//! The session bus is the only channel between the daemon and the UI. If the
//! bus restarts (or our connection is otherwise lost) the daemon would silently
//! stop serving its interfaces until manually restarted. This module spawns a
//! background task that actively probes the live connection and, on failure,
//! rebuilds it and re-registers every interface with exponential backoff.
//!
//! # Scope
//!
//! Monitor + reconnect only. A full Unix-socket fallback is deferred to v0.2
//! per Charter-01. Detection is **active probing** (`DBusProxy::get_id()` with a
//! timeout): zbus 4.x exposes no "connection closed" future, and the `NameLost`
//! fast-path is intentionally omitted to avoid pulling a `Stream` adapter
//! dependency for a marginal latency gain over the periodic probe.

use std::sync::Arc;
use std::time::Duration;

use lnxdrive_ipc::service::{DaemonState, DbusService};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// D-Bus transport health, surfaced to the UI via [`DaemonState::dbus_health`].
///
/// This is orthogonal to `DaemonState::connection_status`, which tracks the
/// cloud (OneDrive) network connection — not the local session bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbusHealth {
    /// Connection to the session bus is live and the name is held.
    Online,
    /// The connection dropped; the monitor is attempting to re-register.
    Reconnecting,
    /// Another instance acquired the name during the outage; this daemon yields.
    Lost,
}

impl DbusHealth {
    /// Stable string form stored in [`DaemonState::dbus_health`] and exposed
    /// over D-Bus.
    pub fn as_str(self) -> &'static str {
        match self {
            DbusHealth::Online => "online",
            DbusHealth::Reconnecting => "reconnecting",
            DbusHealth::Lost => "lost",
        }
    }
}

/// Tunables for the health monitor. [`Default`] provides production values.
#[derive(Clone, Debug)]
pub struct HealthConfig {
    /// How often to probe the live connection with `DBusProxy::get_id()`.
    pub probe_interval: Duration,
    /// Per-probe timeout; a hung probe is treated as a dropped bus.
    pub probe_timeout: Duration,
    /// First backoff delay after a detected drop.
    pub backoff_base: Duration,
    /// Cap on the backoff delay.
    pub backoff_max: Duration,
    /// Multiplier applied per failed reconnect attempt.
    pub backoff_factor: f64,
    /// Symmetric jitter fraction applied to each delay (0.0..=1.0).
    pub backoff_jitter: f64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            probe_interval: Duration::from_secs(5),
            probe_timeout: Duration::from_secs(2),
            backoff_base: Duration::from_millis(500),
            backoff_max: Duration::from_secs(30),
            backoff_factor: 2.0,
            backoff_jitter: 0.2,
        }
    }
}

/// Result of a single liveness probe, factored out for unit testing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// The bus answered the probe.
    Alive,
    /// The probe errored or timed out — treat the bus as gone.
    Dropped,
}

/// Exponential backoff delay for a 0-based `attempt`, before jitter, clamped to
/// [`HealthConfig::backoff_max`]. Pure function — unit-testable without a bus.
pub fn backoff_delay(cfg: &HealthConfig, attempt: u32) -> Duration {
    let base = cfg.backoff_base.as_secs_f64();
    let scaled = base * cfg.backoff_factor.powi(attempt as i32);
    let clamped = scaled.min(cfg.backoff_max.as_secs_f64());
    Duration::from_secs_f64(clamped)
}

/// Apply symmetric jitter to `base`. `rand01` is the RNG sample in `[0, 1)`,
/// injected so the function is deterministic under test. A `rand01` of `0.5`
/// is the identity; `0.0` and `~1.0` hit the `±jitter_frac` extremes.
pub fn apply_jitter(base: Duration, jitter_frac: f64, rand01: f64) -> Duration {
    if jitter_frac <= 0.0 {
        return base;
    }
    let frac = jitter_frac.clamp(0.0, 1.0);
    let delta = (rand01 * 2.0 - 1.0) * frac;
    let secs = (base.as_secs_f64() * (1.0 + delta)).max(0.0);
    Duration::from_secs_f64(secs)
}

/// Classify a probe result. A transport error or timeout means the bus is gone.
pub fn classify_probe(probe_ok: bool) -> ProbeOutcome {
    if probe_ok {
        ProbeOutcome::Alive
    } else {
        ProbeOutcome::Dropped
    }
}

/// Whether an error from [`DbusService::start`] means another process already
/// owns the well-known name (single-instance contention), as opposed to a
/// transient bus failure. Mirrors the string match historically inlined in
/// `main::run`; kept here so both call sites share one definition.
pub fn is_name_taken_error(err: &anyhow::Error) -> bool {
    let s = format!("{err:#}");
    s.contains("already taken")
        || s.contains("already owned")
        || s.contains("NameTaken")
        || s.contains("name already")
}

/// Cheap, non-cryptographic `[0, 1)` sample for jitter, avoiding a `rand`
/// dependency. Quality is irrelevant here — it only de-synchronizes reconnect
/// storms across concurrent user-session daemons.
fn rng_sample() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1_000_000) as f64 / 1_000_000.0
}

/// Update the D-Bus health string in shared state under the lock.
async fn set_dbus_health(state: &Arc<Mutex<DaemonState>>, health: DbusHealth) {
    let mut s = state.lock().await;
    s.dbus_health = health.as_str().to_string();
}

/// Spawns the health monitor task. The monitor **owns** the connection for the
/// rest of the process lifetime; the caller keeps only the returned handle and
/// awaits it during shutdown.
///
/// The caller must establish `initial_connection` synchronously *before*
/// spawning (so single-instance detection happens at startup), then hand it
/// over here.
pub fn spawn_health_monitor(
    dbus_service: Arc<DbusService>,
    initial_connection: zbus::Connection,
    state: Arc<Mutex<DaemonState>>,
    shutdown: CancellationToken,
    cfg: HealthConfig,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        monitor_loop(dbus_service, initial_connection, state, shutdown, cfg).await;
    })
}

/// Why the healthy phase ended.
enum PhaseExit {
    /// Graceful shutdown requested.
    Shutdown,
    /// The bus connection was lost.
    BusDropped,
}

/// Outcome of the reconnect phase.
enum ReconnectResult {
    /// Re-registered successfully; carries the new connection.
    Connected(zbus::Connection),
    /// Shutdown requested mid-backoff.
    Shutdown,
    /// Another instance owns the name; this daemon must yield.
    NameTaken,
}

/// Two-phase supervision loop: hold a live connection and probe it; on loss,
/// rebuild it with backoff. Exits on shutdown or on losing the name to a peer.
async fn monitor_loop(
    dbus_service: Arc<DbusService>,
    mut connection: zbus::Connection,
    state: Arc<Mutex<DaemonState>>,
    shutdown: CancellationToken,
    cfg: HealthConfig,
) {
    info!("D-Bus health monitor started");
    loop {
        match healthy_phase(&connection, &shutdown, &cfg).await {
            PhaseExit::Shutdown => {
                info!("D-Bus health monitor stopping (shutdown)");
                return;
            }
            PhaseExit::BusDropped => {
                warn!("D-Bus session bus connection lost; entering reconnect");
            }
        }

        set_dbus_health(&state, DbusHealth::Reconnecting).await;
        // Release the dead connection so the well-known name can be re-acquired.
        drop(connection);

        match reconnect_phase(&dbus_service, &shutdown, &cfg).await {
            ReconnectResult::Connected(conn) => {
                connection = conn;
                set_dbus_health(&state, DbusHealth::Online).await;
                info!("D-Bus service re-registered after bus recovery");
            }
            ReconnectResult::Shutdown => {
                info!("D-Bus health monitor stopping during reconnect (shutdown)");
                return;
            }
            ReconnectResult::NameTaken => {
                error!(
                    "D-Bus name taken by another instance after reconnect; \
                     yielding single-instance ownership and shutting down"
                );
                set_dbus_health(&state, DbusHealth::Lost).await;
                shutdown.cancel();
                return;
            }
        }
    }
}

/// Hold the connection and probe it at `probe_interval` until shutdown or loss.
async fn healthy_phase(
    connection: &zbus::Connection,
    shutdown: &CancellationToken,
    cfg: &HealthConfig,
) -> PhaseExit {
    let proxy = match zbus::fdo::DBusProxy::new(connection).await {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "Failed to create D-Bus probe proxy; treating bus as dropped");
            return PhaseExit::BusDropped;
        }
    };

    let mut ticker = tokio::time::interval(cfg.probe_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // The first tick fires immediately; consume it so we don't probe the instant
    // we (re)connect.
    ticker.tick().await;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => return PhaseExit::Shutdown,
            _ = ticker.tick() => {
                let probe_ok = match tokio::time::timeout(cfg.probe_timeout, proxy.get_id()).await {
                    Ok(Ok(_)) => true,
                    Ok(Err(e)) => {
                        warn!(error = %e, "D-Bus liveness probe failed");
                        false
                    }
                    Err(_) => {
                        warn!(timeout_ms = cfg.probe_timeout.as_millis() as u64,
                              "D-Bus liveness probe timed out");
                        false
                    }
                };
                match classify_probe(probe_ok) {
                    ProbeOutcome::Alive => debug!("D-Bus liveness probe ok"),
                    ProbeOutcome::Dropped => return PhaseExit::BusDropped,
                }
            }
        }
    }
}

/// Rebuild the service with exponential backoff until it re-registers, the name
/// is lost to a peer, or shutdown is requested.
async fn reconnect_phase(
    dbus_service: &Arc<DbusService>,
    shutdown: &CancellationToken,
    cfg: &HealthConfig,
) -> ReconnectResult {
    let mut attempt: u32 = 0;
    loop {
        // `Builder::session().name()` arbitrates name ownership atomically at the
        // bus, so we attempt `start()` directly and classify the error — no
        // separate `try_acquire_name` probe (which would introduce a TOCTOU gap).
        match dbus_service.start().await {
            Ok(conn) => return ReconnectResult::Connected(conn),
            Err(e) if is_name_taken_error(&e) => return ReconnectResult::NameTaken,
            Err(e) => {
                let delay = apply_jitter(
                    backoff_delay(cfg, attempt),
                    cfg.backoff_jitter,
                    rng_sample(),
                );
                warn!(
                    attempt,
                    delay_ms = delay.as_millis() as u64,
                    error = %e,
                    "D-Bus reconnect attempt failed; backing off"
                );
                tokio::select! {
                    _ = shutdown.cancelled() => return ReconnectResult::Shutdown,
                    _ = tokio::time::sleep(delay) => {}
                }
                attempt = attempt.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_delay_is_geometric_and_clamped() {
        let cfg = HealthConfig::default();
        assert_eq!(backoff_delay(&cfg, 0), Duration::from_millis(500));
        assert_eq!(backoff_delay(&cfg, 1), Duration::from_secs(1));
        assert_eq!(backoff_delay(&cfg, 2), Duration::from_secs(2));
        assert_eq!(backoff_delay(&cfg, 3), Duration::from_secs(4));
        // 0.5 * 2^6 = 32s, clamped to backoff_max (30s).
        assert_eq!(backoff_delay(&cfg, 6), Duration::from_secs(30));
        assert_eq!(backoff_delay(&cfg, 100), Duration::from_secs(30));
    }

    #[test]
    fn apply_jitter_zero_frac_is_identity() {
        let base = Duration::from_secs(4);
        assert_eq!(apply_jitter(base, 0.0, 0.9), base);
    }

    #[test]
    fn apply_jitter_midpoint_is_identity() {
        let base = Duration::from_secs(4);
        // rand01 = 0.5 -> delta 0 -> unchanged.
        assert_eq!(apply_jitter(base, 0.2, 0.5), base);
    }

    #[test]
    fn apply_jitter_stays_within_bounds() {
        let base = Duration::from_secs(10);
        let frac = 0.2;
        let lo = base.as_secs_f64() * (1.0 - frac);
        let hi = base.as_secs_f64() * (1.0 + frac);
        for &r in &[0.0_f64, 0.25, 0.5, 0.75, 0.999] {
            let d = apply_jitter(base, frac, r).as_secs_f64();
            assert!(d >= lo - 1e-9, "below lower bound: r={r} d={d}");
            assert!(d <= hi + 1e-9, "above upper bound: r={r} d={d}");
        }
    }

    #[test]
    fn jitter_extremes_are_symmetric() {
        let base = Duration::from_secs(10);
        let lo = apply_jitter(base, 0.2, 0.0).as_secs_f64();
        let hi = apply_jitter(base, 0.2, 0.999).as_secs_f64();
        assert!((lo - 8.0).abs() < 0.01, "lo={lo}");
        assert!((hi - 12.0).abs() < 0.05, "hi={hi}");
    }

    #[test]
    fn dbus_health_strings() {
        assert_eq!(DbusHealth::Online.as_str(), "online");
        assert_eq!(DbusHealth::Reconnecting.as_str(), "reconnecting");
        assert_eq!(DbusHealth::Lost.as_str(), "lost");
    }

    #[test]
    fn classify_probe_maps_bool() {
        assert_eq!(classify_probe(true), ProbeOutcome::Alive);
        assert_eq!(classify_probe(false), ProbeOutcome::Dropped);
    }

    #[test]
    fn is_name_taken_error_matches_known_strings() {
        for msg in [
            "name already taken",
            "the name is already owned",
            "NameTaken error from bus",
            "that name already exists",
        ] {
            assert!(
                is_name_taken_error(&anyhow::anyhow!("{msg}")),
                "should match: {msg}"
            );
        }
        assert!(!is_name_taken_error(&anyhow::anyhow!("connection refused")));
        assert!(!is_name_taken_error(&anyhow::anyhow!("bus not available")));
    }
}
