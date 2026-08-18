//! D-Bus service implementation for LNXDrive
//!
//! Provides the D-Bus interfaces that UI clients and CLI tools use to
//! communicate with the running LNXDrive daemon:
//!
//! - `com.strangedaystech.LNXDrive.SyncController` - Start, pause, and query sync (legacy)
//! - `com.strangedaystech.LNXDrive.Account` - Account information and auth status (legacy)
//! - `com.strangedaystech.LNXDrive.Conflicts` - Conflict listing and resolution
//! - `com.strangedaystech.LNXDrive.Files` - File status queries, pin/unpin, sync-by-path
//! - `com.strangedaystech.LNXDrive.Sync` - Global sync control with properties and signals
//! - `com.strangedaystech.LNXDrive.Status` - Account and quota information
//! - `com.strangedaystech.LNXDrive.Auth` - OAuth2 authentication flow
//! - `com.strangedaystech.LNXDrive.Settings` - Configuration management
//! - `com.strangedaystech.LNXDrive.Manager` - Daemon lifecycle management
//!
//! Signals are emitted on state changes, sync progress, and errors.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use zbus::zvariant::{OwnedValue, Value};

use crate::auth_backend::AuthBackend;

/// D-Bus well-known name for the LNXDrive daemon
pub const DBUS_NAME: &str = "com.strangedaystech.LNXDrive";

/// D-Bus object path for the service
pub const DBUS_PATH: &str = "/com/strangedaystech/LNXDrive";

// ============================================================================
// Daemon state shared with D-Bus interfaces
// ============================================================================

/// Possible daemon sync states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonSyncState {
    /// Daemon is idle, waiting for next poll interval
    Idle,
    /// Sync cycle is currently running
    Syncing,
    /// Sync is paused by user request
    Paused,
    /// Daemon is waiting for authentication
    WaitingForAuth,
    /// Daemon encountered an error
    Error(String),
}

impl std::fmt::Display for DaemonSyncState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonSyncState::Idle => write!(f, "idle"),
            DaemonSyncState::Syncing => write!(f, "syncing"),
            DaemonSyncState::Paused => write!(f, "paused"),
            DaemonSyncState::WaitingForAuth => write!(f, "waiting_for_auth"),
            DaemonSyncState::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

/// Shared state between the daemon and D-Bus interfaces
pub struct DaemonState {
    /// Current sync state
    pub sync_state: DaemonSyncState,
    /// Whether sync has been requested while paused
    pub sync_requested: bool,
    /// Account email (if authenticated)
    pub account_email: Option<String>,
    /// Account display name (if authenticated)
    pub account_display_name: Option<String>,
    /// Last sync result summary (JSON)
    pub last_sync_result: Option<String>,
    /// Unresolved conflicts as JSON array
    pub conflicts_json: String,

    // -- Files interface state --

    /// Cached file statuses: absolute path → status string
    /// (synced, cloud-only, syncing, pending, conflict, error, excluded, unknown)
    pub file_statuses: HashMap<String, String>,
    /// Queue of pin requests (absolute paths)
    pub pin_requests: Vec<String>,
    /// Queue of unpin requests (absolute paths)
    pub unpin_requests: Vec<String>,
    /// Queue of sync-by-path requests (absolute paths)
    pub sync_path_requests: Vec<String>,

    // -- Sync interface state --

    /// Unix timestamp of last completed sync (0 = never)
    pub last_sync_time: i64,
    /// Number of pending file operations
    pub pending_changes: u32,

    // -- Status interface state --

    /// Network connection status: "online", "offline", "reconnecting"
    pub connection_status: String,
    /// D-Bus session-bus transport health: "online", "reconnecting", "lost".
    ///
    /// Distinct from `connection_status` (which tracks the cloud/OneDrive
    /// network link). Updated by the daemon's D-Bus health monitor (RISK-001).
    pub dbus_health: String,
    /// Storage quota used in bytes
    pub quota_used: u64,
    /// Storage quota total in bytes
    pub quota_total: u64,

    // -- Auth interface state --

    /// Whether the daemon has valid authentication
    pub is_authenticated: bool,
    /// Last generated OAuth2 URL (for in-progress auth flow)
    pub auth_url: Option<String>,
    /// CSRF state token for in-progress auth flow
    pub auth_csrf_state: Option<String>,
    /// PKCE code verifier for the in-progress browser auth flow.
    ///
    /// Retained between `Auth.StartAuth` (which arms the flow via the
    /// backend) and the backend's loopback completion task, which consumes
    /// it for the code exchange. Never leaves the process.
    pub pkce_verifier: Option<String>,
    /// Authentication source: "browser", "goa", or None
    pub auth_source: Option<String>,

    // -- Settings interface state --

    /// Full configuration as YAML string
    pub config_yaml: String,
    /// Currently synced remote folders
    pub selected_folders: Vec<String>,
    /// File exclusion patterns (glob)
    pub exclusion_patterns: Vec<String>,
    /// Remote folder tree as JSON string
    pub remote_folder_tree: String,

    // -- Manager interface state --

    /// Daemon version string
    pub version: String,
    /// Whether the daemon is actively running
    pub is_running: bool,
}

impl Default for DaemonState {
    fn default() -> Self {
        Self {
            sync_state: DaemonSyncState::Idle,
            sync_requested: false,
            account_email: None,
            account_display_name: None,
            last_sync_result: None,
            conflicts_json: "[]".to_string(),
            file_statuses: HashMap::new(),
            pin_requests: Vec::new(),
            unpin_requests: Vec::new(),
            sync_path_requests: Vec::new(),
            last_sync_time: 0,
            pending_changes: 0,
            connection_status: "online".to_string(),
            dbus_health: "online".to_string(),
            quota_used: 0,
            quota_total: 0,
            is_authenticated: false,
            auth_url: None,
            auth_csrf_state: None,
            pkce_verifier: None,
            auth_source: None,
            config_yaml: String::new(),
            selected_folders: Vec::new(),
            exclusion_patterns: Vec::new(),
            remote_folder_tree: "{}".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            is_running: true,
        }
    }
}

// ============================================================================
// T219-T220: SyncController interface
// ============================================================================

/// D-Bus interface for controlling synchronization
///
/// Provides methods to start/pause sync and query the current status.
/// Connected to the daemon's shared state via an `Arc<Mutex<DaemonState>>`.
pub struct SyncControllerInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl SyncControllerInterface {
    /// Creates a new SyncControllerInterface with the given shared state
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.SyncController")]
impl SyncControllerInterface {
    /// Triggers an immediate sync cycle
    ///
    /// If the daemon is paused, the sync request is queued and will run
    /// when the daemon is resumed. If already syncing, this is a no-op.
    async fn start_sync(&self) {
        let mut state = self.state.lock().await;
        match state.sync_state {
            DaemonSyncState::Syncing => {
                debug!("StartSync called but sync is already running");
            }
            DaemonSyncState::Paused => {
                info!("StartSync called while paused, queueing sync request");
                state.sync_requested = true;
            }
            _ => {
                info!("StartSync called, requesting sync cycle");
                state.sync_requested = true;
            }
        }
    }

    /// Pauses synchronization
    ///
    /// The daemon will finish any in-progress sync cycle but will not
    /// start new ones until resumed via `StartSync`.
    async fn pause_sync(&self) {
        let mut state = self.state.lock().await;
        if state.sync_state != DaemonSyncState::Paused {
            info!("PauseSync called, pausing sync");
            state.sync_state = DaemonSyncState::Paused;
        } else {
            debug!("PauseSync called but already paused");
        }
    }

    /// Returns the current daemon status as a JSON string
    ///
    /// The returned JSON contains:
    /// - `state`: Current sync state (idle, syncing, paused, etc.)
    /// - `account_email`: Email of the authenticated account (if any)
    /// - `last_sync_result`: Summary of the last sync cycle (if any)
    async fn get_status(&self) -> String {
        let state = self.state.lock().await;
        let status = serde_json::json!({
            "state": state.sync_state.to_string(),
            "account_email": state.account_email,
            "account_display_name": state.account_display_name,
            "last_sync_result": state.last_sync_result,
        });
        status.to_string()
    }

    // T223: D-Bus signals

    /// Emitted when the sync state changes
    #[zbus(signal)]
    async fn state_changed(signal_ctxt: &zbus::SignalContext<'_>, state: &str) -> zbus::Result<()>;

    /// Emitted to report sync progress
    #[zbus(signal)]
    async fn sync_progress(
        signal_ctxt: &zbus::SignalContext<'_>,
        current: u32,
        total: u32,
    ) -> zbus::Result<()>;

    /// Emitted when an error occurs
    #[zbus(signal)]
    async fn error_occurred(
        signal_ctxt: &zbus::SignalContext<'_>,
        message: &str,
    ) -> zbus::Result<()>;
}

// ============================================================================
// T221: Account interface
// ============================================================================

/// D-Bus interface for account information
///
/// Provides read-only access to the authenticated account's details
/// and authentication status.
pub struct AccountInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl AccountInterface {
    /// Creates a new AccountInterface with the given shared state
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Account")]
impl AccountInterface {
    /// Returns account information as a JSON string
    ///
    /// The returned JSON contains:
    /// - `email`: Account email address
    /// - `display_name`: Account display name
    async fn get_info(&self) -> String {
        let state = self.state.lock().await;
        let info = serde_json::json!({
            "email": state.account_email,
            "display_name": state.account_display_name,
        });
        info.to_string()
    }

    /// Checks whether the daemon has valid authentication
    ///
    /// Returns `true` if the daemon has a configured account with tokens,
    /// `false` otherwise.
    async fn check_auth(&self) -> bool {
        let state = self.state.lock().await;
        state.account_email.is_some()
    }
}

// ============================================================================
// T222: Conflicts interface
// ============================================================================

/// D-Bus interface for conflict management
///
/// Provides methods to list unresolved conflicts and resolve them
/// using a specified strategy.
pub struct ConflictsInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl ConflictsInterface {
    /// Creates a new ConflictsInterface with the given shared state
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Conflicts")]
impl ConflictsInterface {
    /// Returns a JSON array of unresolved conflicts
    ///
    /// Each conflict contains its ID, file path, and detection timestamp.
    async fn list(&self) -> String {
        let state = self.state.lock().await;
        state.conflicts_json.clone()
    }

    /// Returns the full JSON details for a single conflict
    ///
    /// # Arguments
    /// * `id` - The conflict's unique identifier (or a prefix of it)
    ///
    /// # Returns
    /// A JSON object string with the conflict details, or `"{}"` if not found.
    async fn get_details(&self, id: String) -> String {
        let state = self.state.lock().await;
        let json_str = &state.conflicts_json;

        // Parse the JSON array and find the matching conflict
        if let Ok(serde_json::Value::Array(conflicts)) =
            serde_json::from_str::<serde_json::Value>(json_str)
        {
            for conflict in &conflicts {
                if let Some(cid) = conflict.get("id").and_then(|v| v.as_str()) {
                    if cid == id || cid.starts_with(&id) {
                        return conflict.to_string();
                    }
                }
            }
        }

        "{}".to_string()
    }

    /// Attempts to resolve a conflict with the given strategy
    ///
    /// # Arguments
    /// * `id` - The conflict's unique identifier
    /// * `strategy` - Resolution strategy: "keep_local", "keep_remote", or "keep_both"
    ///
    /// # Returns
    /// `true` if the conflict was found and resolution was initiated,
    /// `false` if the conflict was not found or the strategy is invalid
    async fn resolve(&self, id: String, strategy: String) -> bool {
        let valid_strategies = ["keep_local", "keep_remote", "keep_both"];
        if !valid_strategies.contains(&strategy.as_str()) {
            warn!(
                strategy = %strategy,
                "Invalid conflict resolution strategy"
            );
            return false;
        }

        info!(
            conflict_id = %id,
            strategy = %strategy,
            "Conflict resolution requested via D-Bus"
        );

        // Remove the resolved conflict from the state
        let mut state = self.state.lock().await;
        if let Ok(serde_json::Value::Array(mut conflicts)) =
            serde_json::from_str::<serde_json::Value>(&state.conflicts_json)
        {
            let original_len = conflicts.len();
            conflicts.retain(|c| {
                c.get("id")
                    .and_then(|v| v.as_str())
                    .map(|cid| cid != id && !cid.starts_with(&id))
                    .unwrap_or(true)
            });
            if conflicts.len() < original_len {
                state.conflicts_json = serde_json::to_string(&conflicts).unwrap_or_default();
                debug!(
                    "Conflict '{}' resolved with strategy '{}' and removed from state",
                    id, strategy
                );
                return true;
            }
        }

        warn!(conflict_id = %id, "Conflict not found for resolution");
        false
    }

    /// Resolves all unresolved conflicts with the given strategy
    ///
    /// # Arguments
    /// * `strategy` - Resolution strategy: "keep_local", "keep_remote", or "keep_both"
    ///
    /// # Returns
    /// The number of conflicts that were resolved
    async fn resolve_all(&self, strategy: String) -> u32 {
        let valid_strategies = ["keep_local", "keep_remote", "keep_both"];
        if !valid_strategies.contains(&strategy.as_str()) {
            warn!(
                strategy = %strategy,
                "Invalid conflict resolution strategy for resolve_all"
            );
            return 0;
        }

        let mut state = self.state.lock().await;
        let count = if let Ok(serde_json::Value::Array(conflicts)) =
            serde_json::from_str::<serde_json::Value>(&state.conflicts_json)
        {
            conflicts.len() as u32
        } else {
            0
        };

        if count > 0 {
            state.conflicts_json = "[]".to_string();
            info!(
                count = count,
                strategy = %strategy,
                "All conflicts resolved via D-Bus"
            );
        }

        count
    }

    /// Signal emitted when a new conflict is detected
    #[zbus(signal)]
    pub async fn conflict_detected(
        signal_ctxt: &zbus::SignalContext<'_>,
        conflict_json: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when a conflict is resolved
    #[zbus(signal)]
    pub async fn conflict_resolved(
        signal_ctxt: &zbus::SignalContext<'_>,
        conflict_id: &str,
        strategy: &str,
    ) -> zbus::Result<()>;
}

// ============================================================================
// Files interface
// ============================================================================

/// D-Bus interface for file status queries and actions
///
/// Provides methods to query individual/batch file statuses, pin/unpin files,
/// force sync on specific paths, and list conflicts.
/// Connected to the daemon's shared state via an `Arc<Mutex<DaemonState>>`.
pub struct FilesInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl FilesInterface {
    /// Creates a new FilesInterface with the given shared state
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Files")]
impl FilesInterface {
    /// Returns the sync status of a single file
    ///
    /// # Arguments
    /// * `path` - Absolute path to the file
    ///
    /// # Returns
    /// Status string: "synced", "cloud-only", "syncing", "pending",
    /// "conflict", "error", "excluded", or "unknown"
    async fn get_file_status(&self, path: String) -> String {
        let state = self.state.lock().await;
        state
            .file_statuses
            .get(&path)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Returns sync statuses for multiple files in a single call
    ///
    /// # Arguments
    /// * `paths` - List of absolute paths to query
    ///
    /// # Returns
    /// Map of path → status string. Unknown files get "unknown".
    async fn get_batch_file_status(&self, paths: Vec<String>) -> HashMap<String, String> {
        let state = self.state.lock().await;
        paths
            .into_iter()
            .map(|p| {
                let status = state
                    .file_statuses
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                (p, status)
            })
            .collect()
    }

    /// Marks a file to keep available offline (pin + hydrate)
    ///
    /// The request is queued and processed by the sync engine.
    /// Duplicate requests for the same path are ignored.
    async fn pin_file(&self, path: String) {
        let mut state = self.state.lock().await;
        if !state.pin_requests.contains(&path) {
            info!(path = %path, "Pin file requested via D-Bus");
            state.pin_requests.push(path);
        } else {
            debug!(path = %path, "Pin request already queued, ignoring duplicate");
        }
    }

    /// Marks a file to free local space (unpin + dehydrate)
    ///
    /// The request is queued and processed by the sync engine.
    /// Duplicate requests for the same path are ignored.
    async fn unpin_file(&self, path: String) {
        let mut state = self.state.lock().await;
        if !state.unpin_requests.contains(&path) {
            info!(path = %path, "Unpin file requested via D-Bus");
            state.unpin_requests.push(path);
        } else {
            debug!(path = %path, "Unpin request already queued, ignoring duplicate");
        }
    }

    /// Forces immediate synchronization of a specific path
    ///
    /// The request is queued and processed by the sync engine.
    async fn sync_path(&self, path: String) {
        let mut state = self.state.lock().await;
        info!(path = %path, "Sync path requested via D-Bus");
        state.sync_path_requests.push(path);
    }

    /// Returns a list of file paths that are in conflict state
    async fn get_conflicts(&self) -> Vec<String> {
        let state = self.state.lock().await;
        state
            .file_statuses
            .iter()
            .filter(|(_, status)| status.as_str() == "conflict")
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Emitted when a file's sync status changes
    #[zbus(signal)]
    async fn file_status_changed(
        signal_ctxt: &zbus::SignalContext<'_>,
        path: &str,
        status: &str,
    ) -> zbus::Result<()>;
}

// ============================================================================
// Sync interface (com.strangedaystech.LNXDrive.Sync)
// ============================================================================

/// D-Bus interface for global sync control
///
/// Provides methods to trigger/pause/resume sync, read-only properties
/// for sync status, and signals for sync lifecycle events.
/// Coexists with the legacy `SyncController` interface.
pub struct SyncInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl SyncInterface {
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Sync")]
impl SyncInterface {
    /// Trigger an immediate full sync cycle
    async fn sync_now(&self) {
        let mut state = self.state.lock().await;
        match state.sync_state {
            DaemonSyncState::Syncing => {
                debug!("SyncNow called but sync is already running");
            }
            _ => {
                info!("SyncNow called, requesting sync cycle");
                state.sync_requested = true;
            }
        }
    }

    /// Pause synchronization
    async fn pause(&self) {
        let mut state = self.state.lock().await;
        if state.sync_state != DaemonSyncState::Paused {
            info!("Sync.Pause called, pausing sync");
            state.sync_state = DaemonSyncState::Paused;
        } else {
            debug!("Sync.Pause called but already paused");
        }
    }

    /// Resume synchronization from paused state
    async fn resume(&self) {
        let mut state = self.state.lock().await;
        if state.sync_state == DaemonSyncState::Paused {
            info!("Sync.Resume called, resuming sync");
            state.sync_state = DaemonSyncState::Idle;
        } else {
            debug!("Sync.Resume called but not paused (state: {})", state.sync_state);
        }
    }

    /// Global sync state: idle, syncing, paused, error
    #[zbus(property)]
    async fn sync_status(&self) -> String {
        let state = self.state.lock().await;
        match &state.sync_state {
            DaemonSyncState::Idle => "idle".to_string(),
            DaemonSyncState::Syncing => "syncing".to_string(),
            DaemonSyncState::Paused => "paused".to_string(),
            DaemonSyncState::WaitingForAuth => "idle".to_string(),
            DaemonSyncState::Error(_) => "error".to_string(),
        }
    }

    /// Unix timestamp of last completed sync (0 = never)
    #[zbus(property)]
    async fn last_sync_time(&self) -> i64 {
        let state = self.state.lock().await;
        state.last_sync_time
    }

    /// Number of pending file operations
    #[zbus(property)]
    async fn pending_changes(&self) -> u32 {
        let state = self.state.lock().await;
        state.pending_changes
    }

    /// Emitted when a sync cycle begins
    #[zbus(signal)]
    async fn sync_started(signal_ctxt: &zbus::SignalContext<'_>) -> zbus::Result<()>;

    /// Emitted when a sync cycle completes
    #[zbus(signal)]
    async fn sync_completed(
        signal_ctxt: &zbus::SignalContext<'_>,
        files_synced: u32,
        errors: u32,
    ) -> zbus::Result<()>;

    /// Emitted to report per-file sync progress
    #[zbus(signal)]
    async fn sync_progress(
        signal_ctxt: &zbus::SignalContext<'_>,
        file: &str,
        current: u32,
        total: u32,
    ) -> zbus::Result<()>;

    /// Emitted when a new conflict is detected
    #[zbus(signal)]
    async fn conflict_detected(
        signal_ctxt: &zbus::SignalContext<'_>,
        path: &str,
        conflict_type: &str,
    ) -> zbus::Result<()>;
}

// ============================================================================
// Status interface (com.strangedaystech.LNXDrive.Status)
// ============================================================================

/// D-Bus interface for account and quota information
///
/// Provides methods to query storage quota and account details,
/// a read-only connection status property, and change signals.
pub struct StatusInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl StatusInterface {
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Status")]
impl StatusInterface {
    /// Returns storage quota as (used_bytes, total_bytes)
    async fn get_quota(&self) -> (u64, u64) {
        let state = self.state.lock().await;
        (state.quota_used, state.quota_total)
    }

    /// Returns account details as a variant dictionary
    ///
    /// Keys: "email" (s), "display_name" (s), "provider" (s)
    async fn get_account_info(&self) -> HashMap<String, OwnedValue> {
        let state = self.state.lock().await;
        let mut info = HashMap::new();

        let email = state.account_email.clone().unwrap_or_default();
        let name = state.account_display_name.clone().unwrap_or_default();

        info.insert(
            "email".to_string(),
            Value::from(email).try_to_owned().unwrap(),
        );
        info.insert(
            "display_name".to_string(),
            Value::from(name).try_to_owned().unwrap(),
        );
        info.insert(
            "provider".to_string(),
            Value::from("onedrive".to_string()).try_to_owned().unwrap(),
        );

        info
    }

    /// Network connection status: "online", "offline", "reconnecting"
    #[zbus(property)]
    async fn connection_status(&self) -> String {
        let state = self.state.lock().await;
        state.connection_status.clone()
    }

    /// D-Bus session-bus health: "online", "reconnecting", "lost".
    ///
    /// Lets the UI distinguish a session-bus outage (this daemon reconnecting)
    /// from a cloud/network outage. Set by the daemon's health monitor.
    #[zbus(property)]
    async fn dbus_health(&self) -> String {
        let state = self.state.lock().await;
        state.dbus_health.clone()
    }

    /// Emitted when storage quota changes
    #[zbus(signal)]
    async fn quota_changed(
        signal_ctxt: &zbus::SignalContext<'_>,
        used: u64,
        total: u64,
    ) -> zbus::Result<()>;

    /// Emitted when network connection state changes
    #[zbus(signal)]
    async fn connection_changed(
        signal_ctxt: &zbus::SignalContext<'_>,
        status: &str,
    ) -> zbus::Result<()>;
}

// ============================================================================
// Auth interface (com.strangedaystech.LNXDrive.Auth)
// ============================================================================

/// D-Bus interface for authentication
///
/// Provides methods to initiate and complete the OAuth2 PKCE flow,
/// check authentication status, and log out.
pub struct AuthInterface {
    state: Arc<Mutex<DaemonState>>,
    /// Backend that completes authentication without exposing tokens over D-Bus.
    ///
    /// `None` when the interface is constructed for unit tests that do not
    /// exercise the auth backends. Production code must use
    /// [`Self::with_backend`] so that `complete_auth_via_goa` /
    /// `start_auth` can actually fetch tokens and persist them in the
    /// keyring. See [`crate::auth_backend::AuthBackend`] for the expected
    /// contract.
    backend: Option<Arc<dyn AuthBackend>>,
    /// Live D-Bus connection slot, populated by [`DbusService::start`].
    ///
    /// Used to emit `AuthStateChanged` from contexts that are not a zbus
    /// method call (the browser/PKCE loopback completion task). Reads the
    /// CURRENT connection on every emit, so signals keep working after the
    /// health monitor re-registers the service on a fresh connection
    /// (RISK-001 reconnects).
    connection_slot: ConnectionSlot,
}

/// Shared slot holding the daemon's currently-live session-bus connection.
///
/// Written by [`DbusService::start`] every time the service is (re)built —
/// including health-monitor reconnects — and read by [`AuthInterface`] when
/// it needs to emit signals from a background task.
pub type ConnectionSlot = Arc<tokio::sync::RwLock<Option<zbus::Connection>>>;

impl AuthInterface {
    /// Constructs an `AuthInterface` without a backend.
    ///
    /// Calls to `complete_auth_via_goa` will return `false` and `start_auth`
    /// falls back to the legacy placeholder URL until a backend is wired
    /// with [`Self::with_backend`]. This constructor exists so unit tests
    /// that only exercise `start_auth` / `logout` / `is_authenticated`
    /// continue to compile unchanged.
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self {
            state,
            backend: None,
            connection_slot: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Constructs an `AuthInterface` wired to an `AuthBackend`.
    ///
    /// This is the constructor that production code in `lnxdrive-daemon`
    /// uses (via [`DbusService::with_auth_backend`]). The backend
    /// implementation owns the GOA D-Bus client, the OAuth2 PKCE loopback
    /// capture, and the keyring storage; the interface itself never touches
    /// any of them.
    pub fn with_backend(
        state: Arc<Mutex<DaemonState>>,
        backend: Arc<dyn AuthBackend>,
        connection_slot: ConnectionSlot,
    ) -> Self {
        Self {
            state,
            backend: Some(backend),
            connection_slot,
        }
    }
}

/// Emits `AuthStateChanged` on the daemon's live connection, if any.
///
/// Best-effort: when no connection is registered (unit tests, or a window
/// during bus reconnect) the state change is logged and the signal skipped.
/// The interface registration is looked up fresh on every call so reconnects
/// are transparent.
async fn emit_auth_state_changed(slot: &ConnectionSlot, new_state: &str) {
    let conn = slot.read().await.clone();
    let Some(conn) = conn else {
        debug!(
            state = new_state,
            "AuthStateChanged skipped: no live D-Bus connection"
        );
        return;
    };

    // Bind the lookup to a statement so the `object_server()` guard is
    // dropped before `conn` (async temporary-lifetime rules).
    let iface = conn
        .object_server()
        .interface::<_, AuthInterface>(DBUS_PATH)
        .await;
    match iface {
        Ok(iface_ref) => {
            if let Err(e) =
                AuthInterface::auth_state_changed(iface_ref.signal_context(), new_state).await
            {
                warn!(state = new_state, error = %e, "Failed to emit AuthStateChanged");
            } else {
                info!(state = new_state, "AuthStateChanged emitted");
            }
        }
        Err(e) => {
            warn!(
                state = new_state,
                error = %e,
                "AuthInterface not registered; cannot emit AuthStateChanged"
            );
        }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Auth")]
impl AuthInterface {
    /// Starts the browser/PKCE authentication flow.
    ///
    /// Returns (auth_url, csrf_state). The client should open auth_url in a
    /// browser and then wait for the `AuthStateChanged` signal — the
    /// daemon captures the OAuth2 redirect on its own loopback server,
    /// exchanges the code for tokens, and persists them in the keyring
    /// without any secret crossing D-Bus (RISK-002). The historical
    /// `CompleteAuth(code, state)` method was removed because it carried
    /// the authorization code over the bus.
    async fn start_auth(&self) -> (String, String) {
        // Idempotence: a flow already armed returns its URL/state instead
        // of binding a second loopback server.
        {
            let state = self.state.lock().await;
            if let (Some(url), Some(csrf)) = (&state.auth_url, &state.auth_csrf_state) {
                info!("Auth.StartAuth called with a flow already in progress");
                return (url.clone(), csrf.clone());
            }
        }

        let backend = match &self.backend {
            Some(b) => Arc::clone(b),
            None => {
                // Test/no-backend fallback: legacy placeholder so unit tests
                // that construct AuthInterface::new keep working. Production
                // always wires a backend (see DbusService::start).
                let mut state = self.state.lock().await;
                let auth_url = state.auth_url.clone().unwrap_or_else(|| {
                    "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string()
                });
                let csrf_state = state.auth_csrf_state.clone().unwrap_or_else(|| {
                    "pending".to_string()
                });
                info!("Auth.StartAuth called (no backend; placeholder URL)");
                state.auth_url = Some(auth_url.clone());
                state.auth_csrf_state = Some(csrf_state.clone());
                return (auth_url, csrf_state);
            }
        };

        let start = match backend.start_browser_auth().await {
            Ok(start) => start,
            Err(err) => {
                warn!("Auth.StartAuth failed to arm PKCE flow: {}", err);
                return (String::new(), String::new());
            }
        };

        {
            let mut state = self.state.lock().await;
            state.auth_url = Some(start.auth_url.clone());
            state.auth_csrf_state = Some(start.csrf_state.clone());
            state.pkce_verifier = Some(start.pkce_verifier.clone());
        }
        info!("Auth.StartAuth armed a real PKCE flow");

        // Completion task: the backend awaits the loopback redirect,
        // validates CSRF, exchanges the code, persists tokens + account,
        // and this task then updates shared state and emits the signal.
        let state = Arc::clone(&self.state);
        let slot = Arc::clone(&self.connection_slot);
        let csrf_expected = start.csrf_state.clone();
        let verifier = start.pkce_verifier.clone();
        tokio::spawn(async move {
            let result = backend.complete_browser_auth(&csrf_expected, &verifier).await;

            let mut s = state.lock().await;
            // The flow is over either way; clear the armed markers.
            s.auth_url = None;
            s.auth_csrf_state = None;
            s.pkce_verifier = None;

            match result {
                Ok(account) => {
                    s.is_authenticated = true;
                    s.account_email = Some(account.email.clone());
                    if let Some(name) = account.display_name.clone() {
                        s.account_display_name = Some(name);
                    }
                    s.auth_source = Some("browser".to_string());
                    drop(s);
                    emit_auth_state_changed(&slot, "authenticated").await;
                }
                Err(err) => {
                    warn!("Browser/PKCE authentication failed: {}", err);
                    drop(s);
                    emit_auth_state_changed(&slot, "error").await;
                }
            }
        });

        (start.auth_url, start.csrf_state)
    }

    /// Completes authentication for a GNOME Online Accounts account.
    ///
    /// The caller passes only the **GOA account D-Bus path** (e.g.
    /// `/org/gnome/OnlineAccounts/Accounts/1234`), which is a non-sensitive
    /// identifier. The daemon then resolves the path to a Microsoft account
    /// internally — fetching tokens from `org.gnome.OnlineAccounts.OAuth2Based`
    /// and persisting them in the system keyring — without the tokens ever
    /// crossing the D-Bus session bus as method arguments.
    ///
    /// This method replaces the historical `CompleteAuthWithTokens`, which
    /// accepted raw `access_token` and `refresh_token` strings as D-Bus
    /// parameters and was vulnerable to interception by any local process
    /// listening on the session bus (RISK-002, CVSS 9.1). See the AILOG that
    /// closes RISK-002 for the full rationale and the
    /// `lnxdrive-testing/scripts/leak-test-dbus-tokens.sh` integration test
    /// that guards against regressions.
    ///
    /// # Arguments
    /// * `goa_account_path` - D-Bus object path of the GOA account.
    ///
    /// # Returns
    /// `true` if authentication completed and tokens were persisted in the
    /// system keyring; `false` otherwise. The daemon never returns or logs
    /// the tokens themselves; callers learn only of the boolean outcome.
    async fn complete_auth_via_goa(&self, goa_account_path: String) -> bool {
        info!(
            "Auth.CompleteAuthViaGOA called (account_path={})",
            goa_account_path
        );

        // Reject malformed paths early so the backend never sees them.
        if !goa_account_path.starts_with("/org/gnome/OnlineAccounts/Accounts/") {
            warn!(
                "Auth.CompleteAuthViaGOA rejected: path does not look like a GOA account ({})",
                goa_account_path
            );
            return false;
        }

        let backend = match &self.backend {
            Some(b) => Arc::clone(b),
            None => {
                warn!(
                    "Auth.CompleteAuthViaGOA called without a configured backend; rejecting"
                );
                return false;
            }
        };

        let account = match backend.complete_auth_via_goa(&goa_account_path).await {
            Ok(account) => account,
            Err(err) => {
                warn!(
                    "Auth.CompleteAuthViaGOA backend failed: {} (account_path={})",
                    err, goa_account_path
                );
                emit_auth_state_changed(&self.connection_slot, "error").await;
                return false;
            }
        };

        let mut state = self.state.lock().await;
        state.is_authenticated = true;
        if let Some(name) = account.display_name.clone() {
            state.account_display_name = Some(name);
        }
        state.account_email = Some(account.email);
        state.auth_source = Some("goa".to_string());
        state.auth_url = None;
        state.auth_csrf_state = None;
        state.pkce_verifier = None;
        drop(state);

        info!(
            "Auth.CompleteAuthViaGOA succeeded (account_path={})",
            goa_account_path
        );
        emit_auth_state_changed(&self.connection_slot, "authenticated").await;
        true
    }

    /// Checks whether the daemon has valid authentication
    async fn is_authenticated(&self) -> bool {
        let state = self.state.lock().await;
        state.is_authenticated
    }

    /// Removes account credentials and tokens
    async fn logout(&self) {
        let mut state = self.state.lock().await;
        info!("Auth.Logout called");
        state.is_authenticated = false;
        state.auth_source = None;
        state.account_email = None;
        state.account_display_name = None;
        state.auth_url = None;
        state.auth_csrf_state = None;
        state.pkce_verifier = None;
        drop(state);
        emit_auth_state_changed(&self.connection_slot, "unauthenticated").await;
    }

    /// Emitted when authentication state changes
    #[zbus(signal)]
    async fn auth_state_changed(
        signal_ctxt: &zbus::SignalContext<'_>,
        state: &str,
    ) -> zbus::Result<()>;
}

// ============================================================================
// Settings interface (com.strangedaystech.LNXDrive.Settings)
// ============================================================================

/// D-Bus interface for configuration management
///
/// Provides methods to get/set the full YAML config, manage selective
/// sync folders and exclusion patterns, and query the remote folder tree.
pub struct SettingsInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl SettingsInterface {
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Settings")]
impl SettingsInterface {
    /// Returns the full configuration as a YAML string
    async fn get_config(&self) -> String {
        let state = self.state.lock().await;
        state.config_yaml.clone()
    }

    /// Applies a full configuration from a YAML string
    ///
    /// The daemon validates the YAML before applying. Invalid
    /// configurations are rejected silently (logged as warning).
    async fn set_config(&self, yaml: String) {
        let mut state = self.state.lock().await;
        if yaml.is_empty() {
            warn!("Settings.SetConfig called with empty YAML");
            return;
        }
        info!(yaml_len = yaml.len(), "Settings.SetConfig called");
        state.config_yaml = yaml;
    }

    /// Returns the list of currently synced remote folders
    async fn get_selected_folders(&self) -> Vec<String> {
        let state = self.state.lock().await;
        state.selected_folders.clone()
    }

    /// Updates the selective sync folder list
    async fn set_selected_folders(&self, folders: Vec<String>) {
        let mut state = self.state.lock().await;
        info!(count = folders.len(), "Settings.SetSelectedFolders called");
        state.selected_folders = folders;
    }

    /// Returns the current file exclusion patterns
    async fn get_exclusion_patterns(&self) -> Vec<String> {
        let state = self.state.lock().await;
        state.exclusion_patterns.clone()
    }

    /// Updates the file exclusion patterns
    async fn set_exclusion_patterns(&self, patterns: Vec<String>) {
        let mut state = self.state.lock().await;
        info!(count = patterns.len(), "Settings.SetExclusionPatterns called");
        state.exclusion_patterns = patterns;
    }

    /// Returns a JSON tree of remote folders for the selective sync UI
    async fn get_remote_folder_tree(&self) -> String {
        let state = self.state.lock().await;
        state.remote_folder_tree.clone()
    }

    /// Emitted when any configuration value changes
    #[zbus(signal)]
    async fn config_changed(
        signal_ctxt: &zbus::SignalContext<'_>,
        key: &str,
    ) -> zbus::Result<()>;
}

// ============================================================================
// Manager interface (com.strangedaystech.LNXDrive.Manager)
// ============================================================================

/// D-Bus interface for daemon lifecycle management
///
/// Provides methods to start/stop/restart the daemon and
/// read-only properties for version and running state.
pub struct ManagerInterface {
    state: Arc<Mutex<DaemonState>>,
}

impl ManagerInterface {
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self { state }
    }
}

#[zbus::interface(name = "com.strangedaystech.LNXDrive.Manager")]
impl ManagerInterface {
    /// Starts the daemon
    async fn start(&self) {
        let mut state = self.state.lock().await;
        info!("Manager.Start called");
        state.is_running = true;
    }

    /// Stops the daemon
    async fn stop(&self) {
        let mut state = self.state.lock().await;
        info!("Manager.Stop called");
        state.is_running = false;
    }

    /// Restarts the daemon
    async fn restart(&self) {
        let mut state = self.state.lock().await;
        info!("Manager.Restart called");
        state.is_running = false;
        state.is_running = true;
    }

    /// Returns the current daemon status as a string
    async fn get_status(&self) -> String {
        let state = self.state.lock().await;
        if state.is_running { "running" } else { "stopped" }.to_string()
    }

    /// Daemon version string
    #[zbus(property)]
    async fn version(&self) -> String {
        let state = self.state.lock().await;
        state.version.clone()
    }

    /// Whether the daemon is actively running
    #[zbus(property)]
    async fn is_running(&self) -> bool {
        let state = self.state.lock().await;
        state.is_running
    }
}

// ============================================================================
// DbusService - high-level service orchestrator
// ============================================================================

/// High-level D-Bus service that manages all interfaces
///
/// Creates a `zbus::Connection` on the session bus, registers all
/// interface objects at the well-known path, and requests the
/// well-known name `com.strangedaystech.LNXDrive`.
pub struct DbusService {
    state: Arc<Mutex<DaemonState>>,
    auth_backend: Option<Arc<dyn AuthBackend>>,
    /// Slot where the live connection is published after each `start()`,
    /// so [`AuthInterface`] can emit signals from background tasks.
    connection_slot: ConnectionSlot,
}

impl DbusService {
    /// Creates a new DbusService with the given shared state
    pub fn new(state: Arc<Mutex<DaemonState>>) -> Self {
        Self {
            state,
            auth_backend: None,
            connection_slot: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Creates a new DbusService with default state
    pub fn with_default_state() -> Self {
        Self {
            state: Arc::new(Mutex::new(DaemonState::default())),
            auth_backend: None,
            connection_slot: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Attaches an [`AuthBackend`] so that `AuthInterface::complete_auth_via_goa`
    /// can fetch tokens from GOA and persist them in the system keyring.
    ///
    /// Production callers (`lnxdrive-daemon`) MUST install a backend before
    /// calling `start()`; otherwise the GOA path returns `false` at runtime
    /// with a warning log (this is a deliberate safety net for tests).
    #[must_use]
    pub fn with_auth_backend(mut self, backend: Arc<dyn AuthBackend>) -> Self {
        self.auth_backend = Some(backend);
        self
    }

    /// Returns a reference to the shared daemon state
    pub fn state(&self) -> &Arc<Mutex<DaemonState>> {
        &self.state
    }

    /// Starts the D-Bus service on the session bus
    ///
    /// Registers all interfaces and requests the well-known name.
    /// Returns the connection which must be kept alive for the service
    /// to remain active.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The session bus is not available
    /// - The well-known name is already owned (another instance running)
    /// - Interface registration fails
    pub async fn start(&self) -> anyhow::Result<zbus::Connection> {
        info!("Starting D-Bus service on session bus");

        let sync_controller = SyncControllerInterface::new(Arc::clone(&self.state));
        let account_iface = AccountInterface::new(Arc::clone(&self.state));
        let conflicts_iface = ConflictsInterface::new(Arc::clone(&self.state));
        let files_iface = FilesInterface::new(Arc::clone(&self.state));
        let sync_iface = SyncInterface::new(Arc::clone(&self.state));
        let status_iface = StatusInterface::new(Arc::clone(&self.state));
        let auth_iface = match &self.auth_backend {
            Some(backend) => AuthInterface::with_backend(
                Arc::clone(&self.state),
                Arc::clone(backend),
                Arc::clone(&self.connection_slot),
            ),
            None => {
                warn!(
                    "DbusService starting without an AuthBackend; \
                     CompleteAuthViaGOA calls will be rejected at runtime"
                );
                AuthInterface::new(Arc::clone(&self.state))
            }
        };
        let settings_iface = SettingsInterface::new(Arc::clone(&self.state));
        let manager_iface = ManagerInterface::new(Arc::clone(&self.state));

        let connection = zbus::connection::Builder::session()?
            .name(DBUS_NAME)?
            .serve_at(DBUS_PATH, sync_controller)?
            .serve_at(DBUS_PATH, account_iface)?
            .serve_at(DBUS_PATH, conflicts_iface)?
            .serve_at(DBUS_PATH, files_iface)?
            .serve_at(DBUS_PATH, sync_iface)?
            .serve_at(DBUS_PATH, status_iface)?
            .serve_at(DBUS_PATH, auth_iface)?
            .serve_at(DBUS_PATH, settings_iface)?
            .serve_at(DBUS_PATH, manager_iface)?
            .build()
            .await?;

        // Publish the live connection so AuthInterface can emit signals from
        // background tasks. Overwritten on every (re)start, so the slot
        // always holds the current connection after health-monitor reconnects.
        *self.connection_slot.write().await = Some(connection.clone());

        info!(
            name = DBUS_NAME,
            path = DBUS_PATH,
            "D-Bus service started successfully"
        );

        Ok(connection)
    }

    /// Attempts to acquire the D-Bus well-known name to act as a single-instance lock
    ///
    /// If the name is already owned by another process, returns `false`.
    /// This is used by the daemon to ensure only one instance runs at a time.
    pub async fn try_acquire_name() -> anyhow::Result<bool> {
        let connection = zbus::Connection::session().await?;
        let dbus_proxy = zbus::fdo::DBusProxy::new(&connection).await?;

        // Check if the name has an owner
        match dbus_proxy.get_name_owner(DBUS_NAME.try_into()?).await {
            Ok(_owner) => {
                // Name is already owned by another process
                Ok(false)
            }
            Err(_) => {
                // Name is not owned, the daemon can claim it
                Ok(true)
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_backend::{
        AuthBackend, AuthBackendError, AuthBackendResult, AuthenticatedAccount, BrowserAuthStart,
    };
    use async_trait::async_trait;

    /// In-process AuthBackend used by the AuthInterface tests below.
    ///
    /// It returns a configurable result without ever talking to GOA, the
    /// keyring, or the network. Tests build it via the helpers
    /// `MockAuthBackend::ok(email)` / `MockAuthBackend::err(error)`.
    struct MockAuthBackend {
        result: AuthBackendResult,
        last_call: tokio::sync::Mutex<Option<String>>,
    }

    impl MockAuthBackend {
        fn ok(email: &str) -> Arc<Self> {
            Arc::new(Self {
                result: Ok(AuthenticatedAccount {
                    email: email.to_string(),
                    display_name: None,
                }),
                last_call: tokio::sync::Mutex::new(None),
            })
        }

        fn err(error: AuthBackendError) -> Arc<Self> {
            Arc::new(Self {
                result: Err(error),
                last_call: tokio::sync::Mutex::new(None),
            })
        }

        async fn last_call(&self) -> Option<String> {
            self.last_call.lock().await.clone()
        }
    }

    #[async_trait]
    impl AuthBackend for MockAuthBackend {
        async fn complete_auth_via_goa(&self, goa_account_path: &str) -> AuthBackendResult {
            *self.last_call.lock().await = Some(goa_account_path.to_string());
            self.result.clone()
        }

        async fn start_browser_auth(&self) -> Result<BrowserAuthStart, AuthBackendError> {
            *self.last_call.lock().await = Some("start_browser_auth".to_string());
            Ok(BrowserAuthStart {
                auth_url: "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?\
                           client_id=test&code_challenge=test"
                    .to_string(),
                csrf_state: "mock-csrf".to_string(),
                pkce_verifier: "mock-verifier".to_string(),
            })
        }

        async fn complete_browser_auth(
            &self,
            expected_csrf: &str,
            pkce_verifier: &str,
        ) -> AuthBackendResult {
            *self.last_call.lock().await =
                Some(format!("complete_browser_auth:{expected_csrf}:{pkce_verifier}"));
            self.result.clone()
        }
    }

    #[test]
    fn test_daemon_sync_state_display() {
        assert_eq!(DaemonSyncState::Idle.to_string(), "idle");
        assert_eq!(DaemonSyncState::Syncing.to_string(), "syncing");
        assert_eq!(DaemonSyncState::Paused.to_string(), "paused");
        assert_eq!(
            DaemonSyncState::WaitingForAuth.to_string(),
            "waiting_for_auth"
        );
        assert_eq!(
            DaemonSyncState::Error("test".to_string()).to_string(),
            "error: test"
        );
    }

    #[test]
    fn test_daemon_state_default() {
        let state = DaemonState::default();
        assert_eq!(state.sync_state, DaemonSyncState::Idle);
        assert!(!state.sync_requested);
        assert!(state.account_email.is_none());
        assert!(state.account_display_name.is_none());
        assert!(state.last_sync_result.is_none());
        assert_eq!(state.conflicts_json, "[]");
    }

    #[test]
    fn test_dbus_constants() {
        assert_eq!(DBUS_NAME, "com.strangedaystech.LNXDrive");
        assert_eq!(DBUS_PATH, "/com/strangedaystech/LNXDrive");
    }

    #[tokio::test]
    async fn test_sync_controller_get_status() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let controller = SyncControllerInterface::new(Arc::clone(&state));

        let status_json = controller.get_status().await;
        let status: serde_json::Value = serde_json::from_str(&status_json).unwrap();

        assert_eq!(status["state"], "idle");
        assert!(status["account_email"].is_null());
    }

    #[tokio::test]
    async fn test_sync_controller_start_sync() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let controller = SyncControllerInterface::new(Arc::clone(&state));

        controller.start_sync().await;

        let locked = state.lock().await;
        assert!(locked.sync_requested);
    }

    #[tokio::test]
    async fn test_sync_controller_pause_sync() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let controller = SyncControllerInterface::new(Arc::clone(&state));

        controller.pause_sync().await;

        let locked = state.lock().await;
        assert_eq!(locked.sync_state, DaemonSyncState::Paused);
    }

    #[tokio::test]
    async fn test_sync_controller_start_while_paused() {
        let state = Arc::new(Mutex::new(DaemonState {
            sync_state: DaemonSyncState::Paused,
            ..DaemonState::default()
        }));
        let controller = SyncControllerInterface::new(Arc::clone(&state));

        controller.start_sync().await;

        let locked = state.lock().await;
        assert!(locked.sync_requested);
        // State remains paused; the daemon loop handles resumption
        assert_eq!(locked.sync_state, DaemonSyncState::Paused);
    }

    #[tokio::test]
    async fn test_account_get_info_no_account() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let account = AccountInterface::new(Arc::clone(&state));

        let info_json = account.get_info().await;
        let info: serde_json::Value = serde_json::from_str(&info_json).unwrap();

        assert!(info["email"].is_null());
        assert!(info["display_name"].is_null());
    }

    #[tokio::test]
    async fn test_account_get_info_with_account() {
        let state = Arc::new(Mutex::new(DaemonState {
            account_email: Some("user@example.com".to_string()),
            account_display_name: Some("Test User".to_string()),
            ..DaemonState::default()
        }));
        let account = AccountInterface::new(Arc::clone(&state));

        let info_json = account.get_info().await;
        let info: serde_json::Value = serde_json::from_str(&info_json).unwrap();

        assert_eq!(info["email"], "user@example.com");
        assert_eq!(info["display_name"], "Test User");
    }

    #[tokio::test]
    async fn test_account_check_auth_no_account() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let account = AccountInterface::new(state);

        assert!(!account.check_auth().await);
    }

    #[tokio::test]
    async fn test_account_check_auth_with_account() {
        let state = Arc::new(Mutex::new(DaemonState {
            account_email: Some("user@example.com".to_string()),
            ..DaemonState::default()
        }));
        let account = AccountInterface::new(state);

        assert!(account.check_auth().await);
    }

    #[tokio::test]
    async fn test_conflicts_list_empty() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let conflicts = ConflictsInterface::new(state);

        let result = conflicts.list().await;
        assert_eq!(result, "[]");
    }

    #[tokio::test]
    async fn test_conflicts_list_with_data() {
        let conflicts_data = serde_json::json!([
            {"id": "c1", "path": "/test/file.txt"},
            {"id": "c2", "path": "/test/other.txt"},
        ])
        .to_string();

        let state = Arc::new(Mutex::new(DaemonState {
            conflicts_json: conflicts_data.clone(),
            ..DaemonState::default()
        }));
        let conflicts = ConflictsInterface::new(state);

        let result = conflicts.list().await;
        assert_eq!(result, conflicts_data);
    }

    #[tokio::test]
    async fn test_conflicts_resolve_valid_strategy() {
        let conflicts_json = serde_json::json!([
            {"id": "c1", "path": "/file1.txt", "type": "edit"},
            {"id": "c2", "path": "/file2.txt", "type": "edit"},
            {"id": "c3", "path": "/file3.txt", "type": "rename"}
        ])
        .to_string();
        let state = Arc::new(Mutex::new(DaemonState {
            conflicts_json,
            ..DaemonState::default()
        }));
        let conflicts = ConflictsInterface::new(state.clone());

        assert!(
            conflicts
                .resolve("c1".to_string(), "keep_local".to_string())
                .await
        );
        assert!(
            conflicts
                .resolve("c2".to_string(), "keep_remote".to_string())
                .await
        );
        assert!(
            conflicts
                .resolve("c3".to_string(), "keep_both".to_string())
                .await
        );

        // Verify all conflicts were removed
        let final_state = state.lock().await;
        assert_eq!(final_state.conflicts_json, "[]");
    }

    #[tokio::test]
    async fn test_conflicts_resolve_invalid_strategy() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let conflicts = ConflictsInterface::new(state);

        assert!(
            !conflicts
                .resolve("c1".to_string(), "invalid".to_string())
                .await
        );
        assert!(
            !conflicts
                .resolve("c2".to_string(), "delete_all".to_string())
                .await
        );
    }

    #[test]
    fn test_dbus_service_with_default_state() {
        let service = DbusService::with_default_state();
        // Just verify it constructs without panic
        let _state = service.state();
    }

    #[test]
    fn test_dbus_service_with_custom_state() {
        let state = Arc::new(Mutex::new(DaemonState {
            account_email: Some("user@test.com".to_string()),
            ..DaemonState::default()
        }));
        let service = DbusService::new(state);
        let _state = service.state();
    }

    #[tokio::test]
    async fn test_sync_controller_status_with_last_result() {
        let state = Arc::new(Mutex::new(DaemonState {
            sync_state: DaemonSyncState::Idle,
            account_email: Some("user@test.com".to_string()),
            last_sync_result: Some(
                serde_json::json!({
                    "files_downloaded": 5,
                    "files_uploaded": 2,
                    "errors": [],
                })
                .to_string(),
            ),
            ..DaemonState::default()
        }));
        let controller = SyncControllerInterface::new(state);

        let status_json = controller.get_status().await;
        let status: serde_json::Value = serde_json::from_str(&status_json).unwrap();

        assert_eq!(status["state"], "idle");
        assert_eq!(status["account_email"], "user@test.com");
        assert!(status["last_sync_result"].is_string());
    }

    // -- FilesInterface tests --

    #[test]
    fn test_daemon_state_default_includes_files_fields() {
        let state = DaemonState::default();
        assert!(state.file_statuses.is_empty());
        assert!(state.pin_requests.is_empty());
        assert!(state.unpin_requests.is_empty());
        assert!(state.sync_path_requests.is_empty());
    }

    #[tokio::test]
    async fn test_files_get_file_status_known() {
        let mut statuses = HashMap::new();
        statuses.insert("/home/user/doc.txt".to_string(), "synced".to_string());
        statuses.insert("/home/user/photo.jpg".to_string(), "cloud-only".to_string());

        let state = Arc::new(Mutex::new(DaemonState {
            file_statuses: statuses,
            ..DaemonState::default()
        }));
        let files = FilesInterface::new(state);

        assert_eq!(files.get_file_status("/home/user/doc.txt".to_string()).await, "synced");
        assert_eq!(files.get_file_status("/home/user/photo.jpg".to_string()).await, "cloud-only");
    }

    #[tokio::test]
    async fn test_files_get_file_status_unknown() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(state);

        assert_eq!(files.get_file_status("/nonexistent/file.txt".to_string()).await, "unknown");
    }

    #[tokio::test]
    async fn test_files_get_batch_file_status() {
        let mut statuses = HashMap::new();
        statuses.insert("/home/user/a.txt".to_string(), "synced".to_string());
        statuses.insert("/home/user/b.txt".to_string(), "syncing".to_string());

        let state = Arc::new(Mutex::new(DaemonState {
            file_statuses: statuses,
            ..DaemonState::default()
        }));
        let files = FilesInterface::new(state);

        let result = files
            .get_batch_file_status(vec![
                "/home/user/a.txt".to_string(),
                "/home/user/b.txt".to_string(),
                "/home/user/missing.txt".to_string(),
            ])
            .await;

        assert_eq!(result.len(), 3);
        assert_eq!(result["/home/user/a.txt"], "synced");
        assert_eq!(result["/home/user/b.txt"], "syncing");
        assert_eq!(result["/home/user/missing.txt"], "unknown");
    }

    #[tokio::test]
    async fn test_files_get_batch_file_status_empty() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(state);

        let result = files.get_batch_file_status(vec![]).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_files_pin_file() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(Arc::clone(&state));

        files.pin_file("/home/user/important.pdf".to_string()).await;

        let locked = state.lock().await;
        assert_eq!(locked.pin_requests, vec!["/home/user/important.pdf"]);
    }

    #[tokio::test]
    async fn test_files_pin_does_not_duplicate() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(Arc::clone(&state));

        files.pin_file("/home/user/doc.txt".to_string()).await;
        files.pin_file("/home/user/doc.txt".to_string()).await;
        files.pin_file("/home/user/other.txt".to_string()).await;

        let locked = state.lock().await;
        assert_eq!(locked.pin_requests.len(), 2);
        assert_eq!(locked.pin_requests[0], "/home/user/doc.txt");
        assert_eq!(locked.pin_requests[1], "/home/user/other.txt");
    }

    #[tokio::test]
    async fn test_files_unpin_file() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(Arc::clone(&state));

        files.unpin_file("/home/user/large-video.mp4".to_string()).await;

        let locked = state.lock().await;
        assert_eq!(locked.unpin_requests, vec!["/home/user/large-video.mp4"]);
    }

    #[tokio::test]
    async fn test_files_unpin_does_not_duplicate() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(Arc::clone(&state));

        files.unpin_file("/home/user/file.txt".to_string()).await;
        files.unpin_file("/home/user/file.txt".to_string()).await;

        let locked = state.lock().await;
        assert_eq!(locked.unpin_requests.len(), 1);
    }

    #[tokio::test]
    async fn test_files_sync_path() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(Arc::clone(&state));

        files.sync_path("/home/user/urgent/".to_string()).await;

        let locked = state.lock().await;
        assert_eq!(locked.sync_path_requests, vec!["/home/user/urgent/"]);
    }

    #[tokio::test]
    async fn test_files_get_conflicts_empty() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let files = FilesInterface::new(state);

        let conflicts = files.get_conflicts().await;
        assert!(conflicts.is_empty());
    }

    #[tokio::test]
    async fn test_files_get_conflicts_with_data() {
        let mut statuses = HashMap::new();
        statuses.insert("/home/user/ok.txt".to_string(), "synced".to_string());
        statuses.insert("/home/user/bad.txt".to_string(), "conflict".to_string());
        statuses.insert("/home/user/worse.txt".to_string(), "conflict".to_string());
        statuses.insert("/home/user/err.txt".to_string(), "error".to_string());

        let state = Arc::new(Mutex::new(DaemonState {
            file_statuses: statuses,
            ..DaemonState::default()
        }));
        let files = FilesInterface::new(state);

        let mut conflicts = files.get_conflicts().await;
        conflicts.sort();
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0], "/home/user/bad.txt");
        assert_eq!(conflicts[1], "/home/user/worse.txt");
    }

    // -- DaemonState defaults for new fields --

    #[test]
    fn test_daemon_state_default_includes_all_new_fields() {
        let state = DaemonState::default();
        // Sync
        assert_eq!(state.last_sync_time, 0);
        assert_eq!(state.pending_changes, 0);
        // Status
        assert_eq!(state.connection_status, "online");
        assert_eq!(state.quota_used, 0);
        assert_eq!(state.quota_total, 0);
        // Auth
        assert!(!state.is_authenticated);
        assert!(state.auth_url.is_none());
        assert!(state.auth_csrf_state.is_none());
        // Settings
        assert!(state.config_yaml.is_empty());
        assert!(state.selected_folders.is_empty());
        assert!(state.exclusion_patterns.is_empty());
        assert_eq!(state.remote_folder_tree, "{}");
        // Manager
        assert!(!state.version.is_empty());
        assert!(state.is_running);
    }

    // -- SyncInterface tests --

    #[tokio::test]
    async fn test_sync_sync_now_from_idle() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let sync = SyncInterface::new(Arc::clone(&state));

        sync.sync_now().await;

        let locked = state.lock().await;
        assert!(locked.sync_requested);
    }

    #[tokio::test]
    async fn test_sync_sync_now_while_syncing() {
        let state = Arc::new(Mutex::new(DaemonState {
            sync_state: DaemonSyncState::Syncing,
            ..DaemonState::default()
        }));
        let sync = SyncInterface::new(Arc::clone(&state));

        sync.sync_now().await;

        let locked = state.lock().await;
        assert!(!locked.sync_requested); // no-op when already syncing
    }

    #[tokio::test]
    async fn test_sync_pause() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let sync = SyncInterface::new(Arc::clone(&state));

        sync.pause().await;

        let locked = state.lock().await;
        assert_eq!(locked.sync_state, DaemonSyncState::Paused);
    }

    #[tokio::test]
    async fn test_sync_resume_from_paused() {
        let state = Arc::new(Mutex::new(DaemonState {
            sync_state: DaemonSyncState::Paused,
            ..DaemonState::default()
        }));
        let sync = SyncInterface::new(Arc::clone(&state));

        sync.resume().await;

        let locked = state.lock().await;
        assert_eq!(locked.sync_state, DaemonSyncState::Idle);
    }

    #[tokio::test]
    async fn test_sync_resume_when_not_paused() {
        let state = Arc::new(Mutex::new(DaemonState::default())); // Idle
        let sync = SyncInterface::new(Arc::clone(&state));

        sync.resume().await;

        let locked = state.lock().await;
        assert_eq!(locked.sync_state, DaemonSyncState::Idle); // unchanged
    }

    #[tokio::test]
    async fn test_sync_status_property_idle() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let sync = SyncInterface::new(state);
        assert_eq!(sync.sync_status().await, "idle");
    }

    #[tokio::test]
    async fn test_sync_status_property_syncing() {
        let state = Arc::new(Mutex::new(DaemonState {
            sync_state: DaemonSyncState::Syncing,
            ..DaemonState::default()
        }));
        let sync = SyncInterface::new(state);
        assert_eq!(sync.sync_status().await, "syncing");
    }

    #[tokio::test]
    async fn test_sync_status_property_error() {
        let state = Arc::new(Mutex::new(DaemonState {
            sync_state: DaemonSyncState::Error("network".to_string()),
            ..DaemonState::default()
        }));
        let sync = SyncInterface::new(state);
        assert_eq!(sync.sync_status().await, "error");
    }

    #[tokio::test]
    async fn test_sync_last_sync_time_property() {
        let state = Arc::new(Mutex::new(DaemonState {
            last_sync_time: 1738900000,
            ..DaemonState::default()
        }));
        let sync = SyncInterface::new(state);
        assert_eq!(sync.last_sync_time().await, 1738900000);
    }

    #[tokio::test]
    async fn test_sync_pending_changes_property() {
        let state = Arc::new(Mutex::new(DaemonState {
            pending_changes: 42,
            ..DaemonState::default()
        }));
        let sync = SyncInterface::new(state);
        assert_eq!(sync.pending_changes().await, 42);
    }

    // -- StatusInterface tests --

    #[tokio::test]
    async fn test_status_get_quota_default() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let status = StatusInterface::new(state);

        let (used, total) = status.get_quota().await;
        assert_eq!(used, 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_status_get_quota_with_data() {
        let state = Arc::new(Mutex::new(DaemonState {
            quota_used: 5_368_709_120,
            quota_total: 16_106_127_360,
            ..DaemonState::default()
        }));
        let status = StatusInterface::new(state);

        let (used, total) = status.get_quota().await;
        assert_eq!(used, 5_368_709_120); // 5 GB
        assert_eq!(total, 16_106_127_360); // ~15 GB
    }

    #[tokio::test]
    async fn test_status_get_account_info_default() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let status = StatusInterface::new(state);

        let info = status.get_account_info().await;
        assert_eq!(info.len(), 3);
        assert!(info.contains_key("email"));
        assert!(info.contains_key("display_name"));
        assert!(info.contains_key("provider"));
    }

    #[tokio::test]
    async fn test_status_get_account_info_with_account() {
        let state = Arc::new(Mutex::new(DaemonState {
            account_email: Some("test@example.com".to_string()),
            account_display_name: Some("Test User".to_string()),
            ..DaemonState::default()
        }));
        let status = StatusInterface::new(state);

        let info = status.get_account_info().await;
        // Verify the variant dict contains expected keys
        assert_eq!(info.len(), 3);

        // Deserialize the OwnedValue for email
        let email: String = info["email"].try_clone().unwrap().try_into().unwrap();
        assert_eq!(email, "test@example.com");

        let name: String = info["display_name"].try_clone().unwrap().try_into().unwrap();
        assert_eq!(name, "Test User");
    }

    #[tokio::test]
    async fn test_status_connection_status_property() {
        let state = Arc::new(Mutex::new(DaemonState {
            connection_status: "offline".to_string(),
            ..DaemonState::default()
        }));
        let status = StatusInterface::new(state);
        assert_eq!(status.connection_status().await, "offline");
    }

    // -- AuthInterface tests --

    #[tokio::test]
    async fn test_auth_is_authenticated_default() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let auth = AuthInterface::new(state);
        assert!(!auth.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_auth_start_auth() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let auth = AuthInterface::new(Arc::clone(&state));

        let (url, csrf) = auth.start_auth().await;
        assert!(!url.is_empty());
        assert!(!csrf.is_empty());

        let locked = state.lock().await;
        assert!(locked.auth_url.is_some());
        assert!(locked.auth_csrf_state.is_some());
    }

    /// Connection slot for interfaces built in tests: always `None`, so
    /// signal emission is skipped (and logged) instead of touching a bus.
    fn test_slot() -> ConnectionSlot {
        Arc::new(tokio::sync::RwLock::new(None))
    }

    /// Polls until the spawned browser-auth completion task has flipped the
    /// state (or the deadline passes). Returns the final `is_authenticated`.
    async fn wait_until_flow_settles(state: &Arc<Mutex<DaemonState>>) -> bool {
        for _ in 0..200 {
            let s = state.lock().await;
            // The completion task clears the armed marker on both success
            // and failure, so its absence means the task has settled.
            if s.auth_url.is_none() {
                return s.is_authenticated;
            }
            drop(s);
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        state.lock().await.is_authenticated
    }

    #[tokio::test]
    async fn test_auth_logout() {
        let state = Arc::new(Mutex::new(DaemonState {
            is_authenticated: true,
            account_email: Some("user@example.com".to_string()),
            account_display_name: Some("User".to_string()),
            ..DaemonState::default()
        }));
        let auth = AuthInterface::new(Arc::clone(&state));

        auth.logout().await;

        let locked = state.lock().await;
        assert!(!locked.is_authenticated);
        assert!(locked.account_email.is_none());
        assert!(locked.account_display_name.is_none());
        assert!(locked.auth_source.is_none());
    }

    // -- Browser/PKCE flow tests (issue #70: StartAuth arms a real flow and
    //    the backend completes it without secrets crossing D-Bus.)

    #[tokio::test]
    async fn test_auth_start_auth_arms_real_flow_with_backend() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let backend = MockAuthBackend::ok("user@example.com");
        let auth = AuthInterface::with_backend(
            Arc::clone(&state),
            Arc::clone(&backend) as _,
            test_slot(),
        );

        let (url, csrf) = auth.start_auth().await;
        assert!(url.contains("client_id="), "URL must carry OAuth params: {url}");
        assert!(url.contains("code_challenge="), "URL must carry PKCE: {url}");
        assert_eq!(csrf, "mock-csrf");

        let locked = state.lock().await;
        assert_eq!(locked.auth_url.as_deref(), Some(url.as_str()));
        assert_eq!(locked.auth_csrf_state.as_deref(), Some("mock-csrf"));
        assert_eq!(locked.pkce_verifier.as_deref(), Some("mock-verifier"));
    }

    #[tokio::test]
    async fn test_auth_start_auth_is_idempotent_while_armed() {
        // State pre-armed as if a flow were already in progress.
        let state = Arc::new(Mutex::new(DaemonState {
            auth_url: Some("https://in-progress.example.com".to_string()),
            auth_csrf_state: Some("in-progress-csrf".to_string()),
            ..DaemonState::default()
        }));
        let backend = MockAuthBackend::ok("user@example.com");
        let auth = AuthInterface::with_backend(
            Arc::clone(&state),
            Arc::clone(&backend) as _,
            test_slot(),
        );

        let (url1, csrf1) = auth.start_auth().await;
        let (url2, csrf2) = auth.start_auth().await;
        assert_eq!(url1, "https://in-progress.example.com");
        assert_eq!(url1, url2);
        assert_eq!(csrf1, csrf2);
        assert_eq!(csrf1, "in-progress-csrf");
        // A flow already armed must NOT re-arm the backend.
        assert!(backend.last_call().await.is_none());
    }

    #[tokio::test]
    async fn test_auth_browser_flow_completes_and_sets_state() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let backend = MockAuthBackend::ok("user@example.com");
        let auth = AuthInterface::with_backend(
            Arc::clone(&state),
            Arc::clone(&backend) as _,
            test_slot(),
        );

        auth.start_auth().await;
        let authenticated = wait_until_flow_settles(&state).await;
        assert!(authenticated);

        let locked = state.lock().await;
        assert_eq!(locked.account_email.as_deref(), Some("user@example.com"));
        assert_eq!(locked.auth_source.as_deref(), Some("browser"));
        // Armed markers consumed by the completion task.
        assert!(locked.auth_url.is_none());
        assert!(locked.auth_csrf_state.is_none());
        assert!(locked.pkce_verifier.is_none());
        // The backend received the CSRF + verifier armed by start_auth.
        assert_eq!(
            backend.last_call().await.as_deref(),
            Some("complete_browser_auth:mock-csrf:mock-verifier")
        );
    }

    #[tokio::test]
    async fn test_auth_browser_flow_failure_leaves_daemon_unauthenticated() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let backend = MockAuthBackend::err(AuthBackendError::TokenExchangeFailed);
        let auth = AuthInterface::with_backend(
            Arc::clone(&state),
            Arc::clone(&backend) as _,
            test_slot(),
        );

        auth.start_auth().await;
        let authenticated = wait_until_flow_settles(&state).await;
        assert!(!authenticated);

        let locked = state.lock().await;
        assert!(locked.account_email.is_none());
        assert!(locked.auth_url.is_none());
        assert!(locked.pkce_verifier.is_none());
    }

    // -- CompleteAuthViaGOA tests (replace the deleted CompleteAuthWithTokens
    //    tests after RISK-002 / CVSS 9.1 was mitigated; see
    //    AILOG-2026-05-29-002 for the full rationale.)

    #[tokio::test]
    async fn test_auth_complete_via_goa_succeeds_when_backend_returns_email() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let backend = MockAuthBackend::ok("user@example.com");
        let auth = AuthInterface::with_backend(
            Arc::clone(&state),
            Arc::clone(&backend) as _,
            test_slot(),
        );

        let ok = auth
            .complete_auth_via_goa(
                "/org/gnome/OnlineAccounts/Accounts/1234".to_string(),
            )
            .await;
        assert!(ok);
        assert_eq!(
            backend.last_call().await.as_deref(),
            Some("/org/gnome/OnlineAccounts/Accounts/1234")
        );

        let locked = state.lock().await;
        assert!(locked.is_authenticated);
        assert_eq!(locked.auth_source.as_deref(), Some("goa"));
        assert_eq!(locked.account_email.as_deref(), Some("user@example.com"));
    }

    #[tokio::test]
    async fn test_auth_complete_via_goa_rejects_invalid_path_before_calling_backend() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let backend = MockAuthBackend::ok("user@example.com");
        let auth = AuthInterface::with_backend(
            Arc::clone(&state),
            Arc::clone(&backend) as _,
            test_slot(),
        );

        let ok = auth
            .complete_auth_via_goa("/wrong/prefix/Accounts/1234".to_string())
            .await;
        assert!(!ok);
        // Backend MUST NOT be invoked when the path is rejected up front.
        assert!(backend.last_call().await.is_none());
        assert!(!state.lock().await.is_authenticated);
    }

    #[tokio::test]
    async fn test_auth_complete_via_goa_without_backend_returns_false() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        // `new` rather than `with_backend` — no backend configured.
        let auth = AuthInterface::new(Arc::clone(&state));

        let ok = auth
            .complete_auth_via_goa(
                "/org/gnome/OnlineAccounts/Accounts/1234".to_string(),
            )
            .await;
        assert!(!ok);
        assert!(!state.lock().await.is_authenticated);
    }

    #[tokio::test]
    async fn test_auth_complete_via_goa_propagates_backend_failure() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let backend = MockAuthBackend::err(AuthBackendError::KeyringStoreFailed);
        let auth = AuthInterface::with_backend(
            Arc::clone(&state),
            Arc::clone(&backend) as _,
            test_slot(),
        );

        let ok = auth
            .complete_auth_via_goa(
                "/org/gnome/OnlineAccounts/Accounts/1234".to_string(),
            )
            .await;
        assert!(!ok);
        assert_eq!(
            backend.last_call().await.as_deref(),
            Some("/org/gnome/OnlineAccounts/Accounts/1234")
        );
        assert!(!state.lock().await.is_authenticated);
    }

    #[tokio::test]
    async fn test_auth_start_with_existing_url() {
        let state = Arc::new(Mutex::new(DaemonState {
            auth_url: Some("https://custom-auth.example.com".to_string()),
            auth_csrf_state: Some("custom-state".to_string()),
            ..DaemonState::default()
        }));
        let auth = AuthInterface::new(state);

        let (url, csrf) = auth.start_auth().await;
        assert_eq!(url, "https://custom-auth.example.com");
        assert_eq!(csrf, "custom-state");
    }

    // -- SettingsInterface tests --

    #[tokio::test]
    async fn test_settings_get_config_default() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let settings = SettingsInterface::new(state);

        let config = settings.get_config().await;
        assert!(config.is_empty()); // default is empty
    }

    #[tokio::test]
    async fn test_settings_set_and_get_config() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let settings = SettingsInterface::new(Arc::clone(&state));

        let yaml = "sync_root: ~/OneDrive\nsync_mode: hybrid\n".to_string();
        settings.set_config(yaml.clone()).await;

        let result = settings.get_config().await;
        assert_eq!(result, yaml);
    }

    #[tokio::test]
    async fn test_settings_set_config_empty_rejected() {
        let state = Arc::new(Mutex::new(DaemonState {
            config_yaml: "existing: config\n".to_string(),
            ..DaemonState::default()
        }));
        let settings = SettingsInterface::new(Arc::clone(&state));

        settings.set_config(String::new()).await;

        // Empty config is rejected, original preserved
        let locked = state.lock().await;
        assert_eq!(locked.config_yaml, "existing: config\n");
    }

    #[tokio::test]
    async fn test_settings_selected_folders() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let settings = SettingsInterface::new(Arc::clone(&state));

        assert!(settings.get_selected_folders().await.is_empty());

        let folders = vec!["/Documents".to_string(), "/Photos".to_string()];
        settings.set_selected_folders(folders.clone()).await;

        assert_eq!(settings.get_selected_folders().await, folders);
    }

    #[tokio::test]
    async fn test_settings_exclusion_patterns() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let settings = SettingsInterface::new(Arc::clone(&state));

        assert!(settings.get_exclusion_patterns().await.is_empty());

        let patterns = vec!["*.tmp".to_string(), "~$*".to_string(), "Thumbs.db".to_string()];
        settings.set_exclusion_patterns(patterns.clone()).await;

        assert_eq!(settings.get_exclusion_patterns().await, patterns);
    }

    #[tokio::test]
    async fn test_settings_remote_folder_tree() {
        let tree_json = r#"{"name":"root","children":[{"name":"Docs"}]}"#.to_string();
        let state = Arc::new(Mutex::new(DaemonState {
            remote_folder_tree: tree_json.clone(),
            ..DaemonState::default()
        }));
        let settings = SettingsInterface::new(state);

        assert_eq!(settings.get_remote_folder_tree().await, tree_json);
    }

    #[tokio::test]
    async fn test_settings_replace_folders() {
        let state = Arc::new(Mutex::new(DaemonState {
            selected_folders: vec!["/Old".to_string()],
            ..DaemonState::default()
        }));
        let settings = SettingsInterface::new(Arc::clone(&state));

        settings.set_selected_folders(vec!["/New".to_string()]).await;

        let locked = state.lock().await;
        assert_eq!(locked.selected_folders, vec!["/New"]);
    }

    // -- ManagerInterface tests --

    #[tokio::test]
    async fn test_manager_get_status_default() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let manager = ManagerInterface::new(state);
        assert_eq!(manager.get_status().await, "running");
    }

    #[tokio::test]
    async fn test_manager_stop() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let manager = ManagerInterface::new(Arc::clone(&state));

        manager.stop().await;

        let locked = state.lock().await;
        assert!(!locked.is_running);
    }

    #[tokio::test]
    async fn test_manager_start_after_stop() {
        let state = Arc::new(Mutex::new(DaemonState {
            is_running: false,
            ..DaemonState::default()
        }));
        let manager = ManagerInterface::new(Arc::clone(&state));

        manager.start().await;

        let locked = state.lock().await;
        assert!(locked.is_running);
    }

    #[tokio::test]
    async fn test_manager_restart() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let manager = ManagerInterface::new(Arc::clone(&state));

        manager.restart().await;

        let locked = state.lock().await;
        assert!(locked.is_running); // should end running
    }

    #[tokio::test]
    async fn test_manager_get_status_stopped() {
        let state = Arc::new(Mutex::new(DaemonState {
            is_running: false,
            ..DaemonState::default()
        }));
        let manager = ManagerInterface::new(state);
        assert_eq!(manager.get_status().await, "stopped");
    }

    #[tokio::test]
    async fn test_manager_version_property() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let manager = ManagerInterface::new(state);

        let version = manager.version().await;
        assert!(!version.is_empty());
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_manager_is_running_property() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let manager = ManagerInterface::new(state);
        assert!(manager.is_running().await);
    }
}
