//! GNOME Online Accounts (GOA) implementation of [`AuthBackend`].
//!
//! This backend fulfils the mitigation for **RISK-002** (CVSS 9.1, OAuth
//! tokens transmitted in cleartext over the D-Bus session bus). The
//! D-Bus surface in `lnxdrive-ipc` no longer accepts raw tokens as method
//! arguments; instead it accepts a **GOA account D-Bus path** and
//! delegates to this backend, which:
//!
//! 1. Calls `org.gnome.OnlineAccounts.OAuth2Based.GetAccessToken` on the
//!    given account path to obtain the access token internally.
//! 2. Calls `org.freedesktop.DBus.Properties.Get` on the
//!    `org.gnome.OnlineAccounts.Account` interface to read the
//!    `PresentationIdentity` property (the user e-mail).
//! 3. Persists the token in the system keyring via
//!    [`lnxdrive_graph::auth::KeyringTokenStorage`].
//! 4. Returns the user e-mail to the caller. **No tokens are ever
//!    returned, logged at info level, or sent back over D-Bus.**
//!
//! Note that GOA does not expose the refresh token on its public D-Bus
//! API — it manages refreshes internally and exposes only the current
//! access token via `GetAccessToken`. We store the access token (and a
//! `None` refresh token) in the keyring; the daemon's existing
//! [`lnxdrive_graph::auth::OAuth2Provider::refresh_via_goa`] is used for
//! subsequent refreshes.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use lnxdrive_core::ports::Tokens;
use lnxdrive_graph::auth::KeyringTokenStorage;
use lnxdrive_ipc::auth_backend::{AuthBackend, AuthBackendError, AuthBackendResult};
use tracing::{debug, error, info, warn};
use zbus::Connection;

const GOA_BUS: &str = "org.gnome.OnlineAccounts";
const GOA_ACCOUNT_PATH_PREFIX: &str = "/org/gnome/OnlineAccounts/Accounts/";
const GOA_OAUTH2_INTERFACE: &str = "org.gnome.OnlineAccounts.OAuth2Based";
const GOA_ACCOUNT_INTERFACE: &str = "org.gnome.OnlineAccounts.Account";

/// `AuthBackend` that talks to GNOME Online Accounts over D-Bus.
///
/// Holds its own D-Bus session connection (cloned `Arc` internally by
/// `zbus`). The connection is acquired lazily on the first call so that
/// daemons started in environments without a session bus still boot.
pub struct GoaAuthBackend {
    /// Optional pre-acquired session bus connection. When `None`, the
    /// backend opens a fresh connection on every call. Mainly useful for
    /// tests that want to inject a custom connection.
    connection: Option<Connection>,
}

impl GoaAuthBackend {
    /// Returns a backend that opens a fresh `zbus::Connection::session()`
    /// on every call.
    pub fn new() -> Self {
        Self { connection: None }
    }

    /// Returns a backend that reuses the supplied D-Bus connection.
    #[allow(dead_code)] // reserved for integration tests in lnxdrive-testing
    pub fn with_connection(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }

    async fn session_connection(&self) -> Result<Connection, AuthBackendError> {
        if let Some(conn) = &self.connection {
            return Ok(conn.clone());
        }
        Connection::session().await.map_err(|err| {
            error!("GoaAuthBackend: failed to acquire session bus: {}", err);
            AuthBackendError::GoaCallFailed
        })
    }
}

impl Default for GoaAuthBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthBackend for GoaAuthBackend {
    async fn complete_auth_via_goa(&self, goa_account_path: &str) -> AuthBackendResult {
        if !goa_account_path.starts_with(GOA_ACCOUNT_PATH_PREFIX) {
            warn!(
                "GoaAuthBackend rejected non-GOA path: {}",
                goa_account_path
            );
            return Err(AuthBackendError::InvalidAccount);
        }

        let conn = self.session_connection().await?;

        // (1) Fetch the access token from GOA. The call is performed
        //     daemon-side; the token never appears as a public D-Bus
        //     method argument.
        let (access_token, expires_in) = call_goa_get_access_token(&conn, goa_account_path)
            .await
            .map_err(|err| {
                warn!(
                    "GoaAuthBackend: GetAccessToken failed for {}: {}",
                    goa_account_path, err
                );
                AuthBackendError::GoaCallFailed
            })?;

        // (2) Fetch the user e-mail via the standard Properties API.
        let email = call_goa_presentation_identity(&conn, goa_account_path)
            .await
            .map_err(|err| {
                warn!(
                    "GoaAuthBackend: PresentationIdentity lookup failed for {}: {}",
                    goa_account_path, err
                );
                AuthBackendError::GoaCallFailed
            })?;

        let expires_at = Utc::now() + Duration::seconds(i64::from(expires_in));
        let tokens = Tokens {
            access_token,
            // GOA does not expose the refresh token; refreshes are
            // delegated back to GOA via `refresh_via_goa`.
            refresh_token: None,
            expires_at,
        };

        // (3) Persist in the system keyring.
        KeyringTokenStorage::store(&email, &tokens).map_err(|err| {
            error!(
                "GoaAuthBackend: keyring store failed for {}: {}",
                email, err
            );
            AuthBackendError::KeyringStoreFailed
        })?;

        debug!(
            "GoaAuthBackend: stored GOA-issued tokens in keyring for {} \
             (expires_in={}s)",
            email, expires_in
        );
        info!(
            "GoaAuthBackend: completed authentication for GOA account {} \
             (user e-mail captured, no tokens left D-Bus)",
            goa_account_path
        );
        Ok(email)
    }
}

/// Calls `org.gnome.OnlineAccounts.OAuth2Based.GetAccessToken` on the
/// given account path and returns `(access_token, expires_in_seconds)`.
async fn call_goa_get_access_token(
    conn: &Connection,
    goa_account_path: &str,
) -> anyhow::Result<(String, i32)> {
    let reply = conn
        .call_method(
            Some(zbus::names::BusName::from_static_str(GOA_BUS)?),
            goa_account_path,
            Some(zbus::names::InterfaceName::from_static_str(
                GOA_OAUTH2_INTERFACE,
            )?),
            "GetAccessToken",
            &(),
        )
        .await?;
    let (access_token, expires_in): (String, i32) = reply.body().deserialize()?;
    Ok((access_token, expires_in))
}

/// Reads the `PresentationIdentity` property from the GOA `Account`
/// interface — typically the user e-mail address.
async fn call_goa_presentation_identity(
    conn: &Connection,
    goa_account_path: &str,
) -> anyhow::Result<String> {
    let reply = conn
        .call_method(
            Some(zbus::names::BusName::from_static_str(GOA_BUS)?),
            goa_account_path,
            Some(zbus::names::InterfaceName::from_static_str(
                "org.freedesktop.DBus.Properties",
            )?),
            "Get",
            &(GOA_ACCOUNT_INTERFACE, "PresentationIdentity"),
        )
        .await?;
    // The Properties.Get reply is a `Variant<String>`. Deserializing into
    // `OwnedValue` decouples the lifetime from the (temporary) message body.
    let owned: zbus::zvariant::OwnedValue = reply.body().deserialize()?;
    let email: String = TryInto::<String>::try_into(owned).map_err(|err| {
        anyhow::anyhow!("PresentationIdentity is not a string: {}", err)
    })?;
    Ok(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_non_goa_path() {
        let backend = GoaAuthBackend::new();
        let result = backend
            .complete_auth_via_goa("/wrong/prefix/Accounts/1234")
            .await;
        assert_eq!(result, Err(AuthBackendError::InvalidAccount));
    }
}
