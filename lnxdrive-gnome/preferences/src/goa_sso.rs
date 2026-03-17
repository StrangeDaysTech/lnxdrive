// GOA SSO helper — detects existing Microsoft account in GNOME Online Accounts
//
// Implements FR-021: SSO detection for onboarding.
// Only compiled when the "goa" feature is enabled.
//
// Uses the org.gnome.OnlineAccounts D-Bus API to check for and retrieve
// tokens from an existing "lnxdrive_microsoft" provider account.

use zbus::Connection;

const GOA_BUS_NAME: &str = "org.gnome.OnlineAccounts";
const GOA_MANAGER_PATH: &str = "/org/gnome/OnlineAccounts";

/// Checks whether a GOA account with provider type "lnxdrive_microsoft" exists.
pub async fn has_lnxdrive_goa_account() -> bool {
    match find_goa_account_path().await {
        Ok(Some(_)) => true,
        _ => false,
    }
}

/// Retrieves OAuth2 tokens from the existing GOA account.
///
/// Returns (access_token, refresh_token, expires_at_unix) on success.
pub async fn get_goa_tokens() -> Result<(String, String, i64), String> {
    let path = find_goa_account_path()
        .await
        .map_err(|e| format!("D-Bus error: {e}"))?
        .ok_or_else(|| "No LNXDrive GOA account found".to_string())?;

    let conn = Connection::session()
        .await
        .map_err(|e| format!("Session bus: {e}"))?;

    // Call GetAccessToken on the OAuth2Based interface
    let msg = conn
        .call_method(
            Some(GOA_BUS_NAME.into()),
            &path,
            Some("org.gnome.OnlineAccounts.OAuth2Based".into()),
            "GetAccessToken",
            &(),
        )
        .await
        .map_err(|e| format!("GetAccessToken: {e}"))?;

    let (access_token, expires_in): (String, i32) = msg
        .body()
        .deserialize()
        .map_err(|e| format!("Deserialize: {e}"))?;

    // GOA doesn't expose refresh_token via D-Bus; the GOA daemon manages it.
    // For daemon-side refresh, we pass a sentinel and rely on GOA-aware refresh.
    let refresh_token = "__goa_managed__".to_string();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let expires_at = now + expires_in as i64;

    Ok((access_token, refresh_token, expires_at))
}

/// Finds the D-Bus object path of the first GOA account with provider
/// type "lnxdrive_microsoft".
async fn find_goa_account_path() -> Result<Option<String>, zbus::Error> {
    let conn = Connection::session().await?;

    // Use the ObjectManager to enumerate all GOA accounts
    let msg = conn
        .call_method(
            Some(GOA_BUS_NAME.into()),
            GOA_MANAGER_PATH,
            Some("org.freedesktop.DBus.ObjectManager".into()),
            "GetManagedObjects",
            &(),
        )
        .await?;

    // Result type: a{oa{sa{sv}}}
    let objects: std::collections::HashMap<
        zbus::zvariant::OwnedObjectPath,
        std::collections::HashMap<String, std::collections::HashMap<String, zbus::zvariant::OwnedValue>>,
    > = msg.body().deserialize()?;

    for (path, interfaces) in &objects {
        if let Some(account_props) = interfaces.get("org.gnome.OnlineAccounts.Account") {
            if let Some(provider_type) = account_props.get("ProviderType") {
                if let Ok(pt) = <String>::try_from(provider_type.clone()) {
                    if pt == "lnxdrive_microsoft" {
                        return Ok(Some(path.to_string()));
                    }
                }
            }
        }
    }

    Ok(None)
}
