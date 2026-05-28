//! Authentication backend trait.
//!
//! `AuthBackend` is the boundary between the D-Bus surface (`AuthInterface`)
//! and the secret-handling machinery (Microsoft Graph OAuth + system keyring).
//! The D-Bus surface never sees raw tokens — it only invokes the backend
//! through this trait, which is responsible for fetching tokens from
//! GNOME Online Accounts (or any future provider), persisting them in the
//! system keyring, and returning a non-sensitive identifier (the account
//! e-mail) to the caller.
//!
//! Production code wires a `GoaAuthBackend` (in `lnxdrive-daemon`) that
//! talks to `org.gnome.OnlineAccounts` via D-Bus and uses
//! `lnxdrive_graph::auth::KeyringTokenStorage` for persistence. Tests inject
//! a `MockAuthBackend` (see `service.rs` test module) so that unit tests
//! never touch GOA or the keyring.

use async_trait::async_trait;
use std::fmt;

/// Outcome of a backend operation that completes authentication.
///
/// On success the backend returns the **account e-mail** (used as the keyring
/// username key). On failure the backend returns a [`AuthBackendError`]
/// describing the cause. Error variants are intentionally coarse-grained:
/// the D-Bus surface only differentiates "completed" vs "failed", and the
/// detailed reason is reported through `tracing` logs by the backend itself.
pub type AuthBackendResult = Result<String, AuthBackendError>;

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
/// 1. Fetching tokens from the upstream provider (GOA today).
/// 2. Persisting them in the system keyring.
/// 3. Returning the account e-mail so the daemon can update its state.
///
/// The trait deliberately accepts only the GOA account D-Bus path as input,
/// which is a non-sensitive identifier (it does NOT carry the token).
#[async_trait]
pub trait AuthBackend: Send + Sync {
    /// Completes authentication for the GOA account identified by
    /// `goa_account_path` (e.g. `/org/gnome/OnlineAccounts/Accounts/1234`).
    ///
    /// Returns the account e-mail on success. Implementations MUST NOT
    /// return raw tokens through this method.
    async fn complete_auth_via_goa(&self, goa_account_path: &str) -> AuthBackendResult;
}
