//! Authentication backend trait.
//!
//! `AuthBackend` is the boundary between the D-Bus surface (`AuthInterface`)
//! and the secret-handling machinery (Microsoft Graph OAuth + system keyring).
//! The D-Bus surface never sees raw tokens — it only invokes the backend
//! through this trait, which is responsible for fetching/generating tokens
//! (GNOME Online Accounts or the OAuth2 PKCE loopback flow), persisting them
//! in the system keyring, and returning non-sensitive identifiers (the
//! account e-mail / display name) to the caller.
//!
//! Production code wires a `GoaAuthBackend` (in `lnxdrive-daemon`) that
//! talks to `org.gnome.OnlineAccounts` via D-Bus, owns the OAuth2 PKCE
//! loopback capture, and uses `lnxdrive_graph::auth::KeyringTokenStorage`
//! for persistence. Tests inject a `MockAuthBackend` (see `service.rs` test
//! module) so that unit tests never touch GOA, the keyring, or the network.

use async_trait::async_trait;
use std::fmt;

/// Non-sensitive account identifiers returned by the backend after a
/// successful authentication.
///
/// `display_name` is `None` when the backend could not resolve it (e.g. a
/// GOA login whose token lacks the scopes to query Graph `/me`); callers
/// should fall back to the e-mail for display purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedAccount {
    /// Account e-mail (also the keyring username key).
    pub email: String,
    /// Human-readable display name, when resolvable.
    pub display_name: Option<String>,
}

/// Outcome of a backend operation that completes authentication.
///
/// On success the backend returns the [`AuthenticatedAccount`]. On failure
/// it returns an [`AuthBackendError`] describing the cause. Error variants
/// are intentionally coarse-grained: the D-Bus surface only differentiates
/// "completed" vs "failed", and the detailed reason is reported through
/// `tracing` logs by the backend itself.
pub type AuthBackendResult = Result<AuthenticatedAccount, AuthBackendError>;

/// Result of arming a browser/PKCE authentication flow.
///
/// None of these values is secret on its own: `auth_url` is meant to be
/// handed to the user's browser, `csrf_state` is the public CSRF token, and
/// `pkce_verifier` stays process-local (it is never returned over D-Bus;
/// it is consumed by [`AuthBackend::complete_browser_auth`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserAuthStart {
    /// Full OAuth2 authorization URL (client_id, PKCE challenge, scopes,
    /// redirect_uri, state — all populated).
    pub auth_url: String,
    /// CSRF state token embedded in the URL; must match the callback.
    pub csrf_state: String,
    /// PKCE code verifier retained for the code exchange step.
    pub pkce_verifier: String,
}

/// Errors that an `AuthBackend` can report.
///
/// These do not carry sensitive material. The backend is expected to log
/// any details it captures (D-Bus error names, GOA error bodies, keyring
/// failure modes) through `tracing` before returning the error variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthBackendError {
    /// The GOA account path was rejected by the backend before any call
    /// was made (e.g. malformed path, unsupported provider).
    InvalidAccount,
    /// GOA returned an error or the call itself failed (D-Bus error,
    /// timeout, unknown service, …).
    GoaCallFailed,
    /// The browser/PKCE leg failed: the loopback callback never arrived
    /// (timeout), the CSRF state did not match, or the authorization code
    /// could not be exchanged for tokens.
    TokenExchangeFailed,
    /// Tokens were obtained but the user profile (e-mail) could not be
    /// resolved, so the account cannot be identified/persisted.
    UserInfoFailed,
    /// Token persistence in the system keyring failed.
    KeyringStoreFailed,
    /// Catch-all for unexpected backend failures. Reserved for situations
    /// that are programming errors rather than expected runtime conditions.
    Internal,
}

impl fmt::Display for AuthBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAccount => write!(f, "invalid GOA account path"),
            Self::GoaCallFailed => write!(f, "GOA D-Bus call failed"),
            Self::TokenExchangeFailed => write!(f, "OAuth code exchange failed"),
            Self::UserInfoFailed => write!(f, "user profile lookup failed"),
            Self::KeyringStoreFailed => write!(f, "keyring store failed"),
            Self::Internal => write!(f, "internal backend error"),
        }
    }
}

impl std::error::Error for AuthBackendError {}

/// Backend that the `AuthInterface` delegates to in order to complete
/// authentication without ever exposing raw tokens over D-Bus.
///
/// Implementations are responsible for:
/// 1. Obtaining tokens from the upstream source (GOA, or the OAuth2 PKCE
///    loopback capture fed by the user's browser).
/// 2. Persisting them in the system keyring.
/// 3. Persisting the account so the daemon can pick it up.
/// 4. Returning non-sensitive identifiers so the daemon can update its
///    state.
///
/// The trait deliberately accepts only non-sensitive inputs: the GOA
/// account D-Bus path, and the CSRF state + PKCE verifier of a flow this
/// process armed itself. Raw tokens and authorization codes never cross
/// this boundary as arguments coming from D-Bus clients.
#[async_trait]
pub trait AuthBackend: Send + Sync {
    /// Completes authentication for the GOA account identified by
    /// `goa_account_path` (e.g. `/org/gnome/OnlineAccounts/Accounts/1234`).
    ///
    /// Returns the authenticated account on success. Implementations MUST
    /// NOT return raw tokens through this method.
    async fn complete_auth_via_goa(&self, goa_account_path: &str) -> AuthBackendResult;

    /// Arms a browser/PKCE authentication flow without performing network
    /// I/O: generates the authorization URL (with PKCE challenge), the CSRF
    /// state token, and the PKCE verifier.
    ///
    /// The caller stores `csrf_state`/`pkce_verifier` and hands `auth_url`
    /// to the user's browser, then awaits
    /// [`complete_browser_auth`](Self::complete_browser_auth).
    async fn start_browser_auth(&self) -> Result<BrowserAuthStart, AuthBackendError>;

    /// Waits for the OAuth2 loopback redirect, validates the CSRF state,
    /// exchanges the captured authorization code for tokens, resolves the
    /// user profile, and persists tokens + account.
    ///
    /// `expected_csrf` is the CSRF token returned by
    /// [`start_browser_auth`](Self::start_browser_auth); a callback whose
    /// `state` differs MUST be rejected. `pkce_verifier` is the verifier
    /// from the same arming call.
    ///
    /// The authorization code is captured by the backend itself (loopback
    /// server inside this process) and never crosses D-Bus — the RISK-002
    /// invariant applies to the code as well as to the tokens.
    async fn complete_browser_auth(
        &self,
        expected_csrf: &str,
        pkce_verifier: &str,
    ) -> AuthBackendResult;
}
