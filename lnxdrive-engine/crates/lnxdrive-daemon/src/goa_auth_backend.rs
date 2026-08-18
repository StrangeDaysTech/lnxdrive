//! Auth backend for the daemon: GNOME Online Accounts + browser/PKCE.
//!
//! This backend fulfils the mitigation for **RISK-002** (CVSS 9.1, OAuth
//! tokens transmitted in cleartext over the D-Bus session bus). The
//! D-Bus surface in `lnxdrive-ipc` never accepts raw tokens or
//! authorization codes as method arguments; it delegates to this backend,
//! which owns every secret-handling step:
//!
//! **GOA route** (primary on GNOME, decisión D2):
//!
//! 1. Calls `org.gnome.OnlineAccounts.OAuth2Based.GetAccessToken` on the
//!    given account path to obtain the access token internally.
//! 2. Calls `org.freedesktop.DBus.Properties.Get` on the
//!    `org.gnome.OnlineAccounts.Account` interface to read the
//!    `PresentationIdentity` property (the user e-mail).
//! 3. Persists the token in the system keyring via
//!    [`lnxdrive_graph::auth::KeyringTokenStorage`].
//! 4. Best-effort enriches and persists the account via Graph (risk R1:
//!    if the GOA token lacks usable scopes this step fails without
//!    aborting the login — see `complete_auth_via_goa`).
//! 5. Returns non-sensitive identifiers to the caller. **No tokens are
//!    ever returned, logged at info level, or sent back over D-Bus.**
//!
//! **Browser/PKCE route** (universal fallback, issue #70):
//!
//! 1. `start_browser_auth` arms a real OAuth2 PKCE flow (client_id,
//!    scopes, redirect_uri, state, code_challenge) via
//!    [`lnxdrive_graph::auth::arm_pkce_flow`].
//! 2. `complete_browser_auth` awaits the redirect on the loopback server
//!    INSIDE this process ([`lnxdrive_graph::auth::LocalCallbackServer`]),
//!    validates the CSRF state, exchanges the captured code for tokens,
//!    resolves the user profile, and persists tokens + account. The code
//!    never crosses D-Bus.
//!
//! Both routes persist the account in SQLite so the daemon's
//! `wait_for_auth_loop` picks the login up without a restart.
//!
//! Note that GOA does not expose the refresh token on its public D-Bus
//! API — it manages refreshes internally and exposes only the current
//! access token via `GetAccessToken`. We store the access token (and a
//! `None` refresh token) in the keyring; the daemon's existing
//! [`lnxdrive_graph::auth::GraphAuthAdapter::refresh_via_goa`] is used for
//! subsequent refreshes.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use lnxdrive_cache::SqliteStateRepository;
use lnxdrive_core::domain::{Account, AuditAction, AuditEntry, AuditResult, Email, SyncPath};
use lnxdrive_core::ports::{IStateRepository, Tokens};
use lnxdrive_graph::auth::{
    arm_pkce_flow, exchange_pkce_code, KeyringTokenStorage, LocalCallbackServer, OAuth2Config,
};
use lnxdrive_graph::{client::GraphClient, provider::GraphCloudProvider};
use lnxdrive_ipc::auth_backend::{
    AuthBackend, AuthBackendError, AuthBackendResult, AuthenticatedAccount, BrowserAuthStart,
};
use tracing::{debug, error, info, warn};
use zbus::Connection;

const GOA_BUS: &str = "org.gnome.OnlineAccounts";
const GOA_ACCOUNT_PATH_PREFIX: &str = "/org/gnome/OnlineAccounts/Accounts/";
const GOA_OAUTH2_INTERFACE: &str = "org.gnome.OnlineAccounts.OAuth2Based";
const GOA_ACCOUNT_INTERFACE: &str = "org.gnome.OnlineAccounts.Account";

/// How long `complete_browser_auth` waits for the OAuth loopback redirect
/// before giving up. Bounds the lifetime of the loopback listener when the
/// user abandons the flow mid-login.
const BROWSER_AUTH_TIMEOUT: Duration = Duration::from_secs(300);

/// `AuthBackend` that talks to GNOME Online Accounts over D-Bus and owns
/// the browser/PKCE loopback capture.
///
/// Holds its own D-Bus session connection (cloned `Arc` internally by
/// `zbus`). The connection is acquired lazily on the first call so that
/// daemons started in environments without a session bus still boot.
pub struct GoaAuthBackend {
    /// Optional pre-acquired session bus connection. When `None`, the
    /// backend opens a fresh connection on every call. Mainly useful for
    /// tests that want to inject a custom connection.
    connection: Option<Connection>,
    /// OAuth2 application (client) ID, resolved by the daemon
    /// (config `auth.app_id` → default).
    app_id: String,
    /// Repository where the authenticated account is persisted so the
    /// daemon's auth-wait loop can pick it up.
    state_repo: Arc<SqliteStateRepository>,
    /// Sync root for newly created accounts (config `sync.root`).
    sync_root: PathBuf,
}

impl GoaAuthBackend {
    /// Returns a backend with the daemon's resolved `app_id`, state
    /// repository, and sync root.
    pub fn new(
        app_id: String,
        state_repo: Arc<SqliteStateRepository>,
        sync_root: PathBuf,
    ) -> Self {
        Self {
            connection: None,
            app_id,
            state_repo,
            sync_root,
        }
    }

    /// Returns a backend that reuses the supplied D-Bus connection.
    #[allow(dead_code)] // reserved for integration tests in lnxdrive-testing
    pub fn with_connection(
        connection: Connection,
        app_id: String,
        state_repo: Arc<SqliteStateRepository>,
        sync_root: PathBuf,
    ) -> Self {
        Self {
            connection: Some(connection),
            app_id,
            state_repo,
            sync_root,
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

    /// OAuth2 configuration for the browser/PKCE route (default redirect
    /// and scopes — the daemon owns no overrides for them).
    fn oauth2_config(&self) -> OAuth2Config {
        OAuth2Config::new(&self.app_id)
    }

    /// Persists the authenticated account (SQLite) and records the login
    /// audit entry, mirroring the CLI's `auth login` steps 5-6. This is
    /// what lets the daemon's `wait_for_auth_loop` resume without a
    /// process restart.
    async fn persist_account_and_audit(
        &self,
        email: &str,
        display_name: &str,
        onedrive_id: &str,
        quota_used: u64,
        quota_total: u64,
    ) -> Result<(), AuthBackendError> {
        let email_addr = Email::new(email.to_string()).map_err(|err| {
            error!("GoaAuthBackend: invalid e-mail from provider: {}", err);
            AuthBackendError::UserInfoFailed
        })?;
        let sync_root = SyncPath::new(self.sync_root.clone()).map_err(|err| {
            error!("GoaAuthBackend: invalid sync root: {}", err);
            AuthBackendError::Internal
        })?;

        let mut account = Account::new(email_addr, display_name, onedrive_id, sync_root);
        account.update_quota(quota_used, quota_total);

        self.state_repo
            .save_account(&account)
            .await
            .map_err(|err| {
                error!("GoaAuthBackend: failed to persist account: {}", err);
                AuthBackendError::Internal
            })?;

        let audit_entry = AuditEntry::new(AuditAction::AuthLogin, AuditResult::success())
            .with_details(serde_json::json!({
                "email": email,
                "display_name": display_name,
                "drive_id": onedrive_id,
            }));
        self.state_repo.save_audit(&audit_entry).await.map_err(|err| {
            error!("GoaAuthBackend: failed to record login audit: {}", err);
            AuthBackendError::Internal
        })?;

        info!(email, "GoaAuthBackend: account persisted after login");
        Ok(())
    }

    /// Best-effort Graph profile lookup with the given access token.
    ///
    /// Used by the GOA route to enrich the account (display name, drive
    /// id, quota) and — empirically — to prove risk R1 (the GOA token
    /// carries scopes usable against Graph). Failures are logged and
    /// reported without aborting the login.
    async fn fetch_user_info(access_token: &str) -> anyhow::Result<lnxdrive_core::ports::UserInfo> {
        use lnxdrive_core::ports::cloud_provider::ICloudProvider;
        let client = GraphClient::new(access_token);
        let provider = GraphCloudProvider::new(client);
        provider.get_user_info().await
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

        let expires_at = Utc::now() + ChronoDuration::seconds(i64::from(expires_in));
        let tokens = Tokens {
            access_token: access_token.clone(),
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

        // (4) Best-effort account enrichment + persistence. If the GOA
        //     token cannot operate against Graph (risk R1 materializing)
        //     we still persist a minimal account: the login is real, and
        //     the daemon is better off surfacing the Graph failure from
        //     sync than silently waiting for auth that already happened.
        let display_name = match Self::fetch_user_info(&access_token).await {
            Ok(user_info) => {
                if let Err(err) = self
                    .persist_account_and_audit(
                        &user_info.email,
                        &user_info.display_name,
                        &user_info.id,
                        user_info.quota_used,
                        user_info.quota_total,
                    )
                    .await
                {
                    warn!(
                        "GoaAuthBackend: account persistence failed (login continues): {}",
                        err
                    );
                }
                user_info.display_name
            }
            Err(err) => {
                warn!(
                    "GoaAuthBackend: Graph profile lookup failed for {} (risk R1?): {}",
                    email, err
                );
                if let Err(err) = self
                    .persist_account_and_audit(&email, &email, "", 0, 0)
                    .await
                {
                    warn!(
                        "GoaAuthBackend: fallback account persistence failed: {}",
                        err
                    );
                }
                email.clone()
            }
        };

        info!(
            "GoaAuthBackend: completed authentication for GOA account {} \
             (user e-mail captured, no tokens left D-Bus)",
            goa_account_path
        );
        Ok(AuthenticatedAccount {
            email,
            display_name: Some(display_name),
        })
    }

    async fn start_browser_auth(&self) -> Result<BrowserAuthStart, AuthBackendError> {
        let (auth_url, csrf_state, pkce_verifier) =
            arm_pkce_flow(&self.oauth2_config()).map_err(|err| {
                error!("GoaAuthBackend: failed to arm PKCE flow: {}", err);
                AuthBackendError::Internal
            })?;
        debug!("GoaAuthBackend: armed browser/PKCE flow");
        Ok(BrowserAuthStart {
            auth_url,
            csrf_state,
            pkce_verifier,
        })
    }

    async fn complete_browser_auth(
        &self,
        expected_csrf: &str,
        pkce_verifier: &str,
    ) -> AuthBackendResult {
        // (1) Await the OAuth loopback redirect. The server runs inside
        //     this process; the authorization code never crosses D-Bus.
        let callback =
            match tokio::time::timeout(BROWSER_AUTH_TIMEOUT, LocalCallbackServer::start()).await {
                Ok(Ok(callback)) => callback,
                Ok(Err(err)) => {
                    warn!("GoaAuthBackend: loopback callback server failed: {}", err);
                    return Err(AuthBackendError::TokenExchangeFailed);
                }
                Err(_) => {
                    warn!(
                        "GoaAuthBackend: no OAuth callback within {:?}; abandoning flow",
                        BROWSER_AUTH_TIMEOUT
                    );
                    return Err(AuthBackendError::TokenExchangeFailed);
                }
            };

        // (2) Validate the CSRF state before touching the code.
        if callback.state != expected_csrf {
            warn!("GoaAuthBackend: OAuth CSRF state mismatch; rejecting callback");
            return Err(AuthBackendError::TokenExchangeFailed);
        }

        // (3) Exchange the code for tokens (PKCE).
        let tokens = exchange_pkce_code(&self.oauth2_config(), callback.code, pkce_verifier)
            .await
            .map_err(|err| {
                warn!("GoaAuthBackend: OAuth code exchange failed: {}", err);
                AuthBackendError::TokenExchangeFailed
            })?;

        // (4) Resolve the user profile — identifies the account (e-mail
        //     doubles as the keyring key) and supplies the drive id.
        let user_info = Self::fetch_user_info(&tokens.access_token)
            .await
            .map_err(|err| {
                warn!("GoaAuthBackend: user profile lookup failed: {}", err);
                AuthBackendError::UserInfoFailed
            })?;

        // (5) Persist tokens in the keyring, then account + audit.
        KeyringTokenStorage::store(&user_info.email, &tokens).map_err(|err| {
            error!(
                "GoaAuthBackend: keyring store failed for {}: {}",
                user_info.email, err
            );
            AuthBackendError::KeyringStoreFailed
        })?;

        self.persist_account_and_audit(
            &user_info.email,
            &user_info.display_name,
            &user_info.id,
            user_info.quota_used,
            user_info.quota_total,
        )
        .await?;

        info!(
            "GoaAuthBackend: browser/PKCE login completed (no code or tokens crossed D-Bus)"
        );
        Ok(AuthenticatedAccount {
            email: user_info.email,
            display_name: Some(user_info.display_name),
        })
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
    use lnxdrive_cache::pool::DatabasePool;

    async fn test_backend() -> GoaAuthBackend {
        let pool = DatabasePool::in_memory()
            .await
            .expect("in-memory test database");
        GoaAuthBackend::new(
            "test-app-id".to_string(),
            Arc::new(SqliteStateRepository::new(pool.pool().clone())),
            PathBuf::from("/tmp/OneDrive-test"),
        )
    }

    #[tokio::test]
    async fn rejects_non_goa_path() {
        let backend = test_backend().await;
        let result = backend
            .complete_auth_via_goa("/wrong/prefix/Accounts/1234")
            .await;
        assert_eq!(result, Err(AuthBackendError::InvalidAccount));
    }

    #[tokio::test]
    async fn start_browser_auth_arms_real_pkce_url() {
        let backend = test_backend().await;
        let start = backend
            .start_browser_auth()
            .await
            .expect("arming must not fail");

        assert!(start.auth_url.contains("client_id=test-app-id"));
        assert!(start.auth_url.contains("code_challenge="));
        assert!(start.auth_url.contains("code_challenge_method=S256"));
        assert!(start.auth_url.contains("redirect_uri="));
        assert!(start.auth_url.contains("scope="));
        assert!(start.auth_url.contains("state="));
        assert!(!start.csrf_state.is_empty());
        assert!(start.pkce_verifier.len() >= 43, "PKCE verifiers are 43-128 chars");

        // Two arming calls produce distinct CSRF states (fresh randomness).
        let start2 = backend.start_browser_auth().await.expect("second arming");
        assert_ne!(start.csrf_state, start2.csrf_state);
    }
}
