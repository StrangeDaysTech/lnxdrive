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

/// Returns the D-Bus object path of the existing "lnxdrive_microsoft" GOA
/// account, if any.
///
/// Post-RISK-002 the client no longer fetches tokens itself: it hands this path
/// to the daemon via `Auth.CompleteAuthViaGOA`, and the daemon reads the tokens
/// from GOA and stores them in the keyring, so tokens never cross D-Bus.
pub async fn lnxdrive_goa_account_path() -> Result<Option<String>, String> {
    find_goa_account_path()
        .await
        .map_err(|e| format!("D-Bus error: {e}"))
}

/// Finds the D-Bus object path of the first GOA account with provider
/// type "lnxdrive_microsoft".
async fn find_goa_account_path() -> Result<Option<String>, zbus::Error> {
    let conn = Connection::session().await?;

    // Use the ObjectManager to enumerate all GOA accounts
    let msg = conn
        .call_method(
            Some(GOA_BUS_NAME),
            GOA_MANAGER_PATH,
            Some("org.freedesktop.DBus.ObjectManager"),
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
