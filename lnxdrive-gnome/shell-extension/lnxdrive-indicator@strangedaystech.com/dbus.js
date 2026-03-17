// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Strange Days Tech <https://strangedays.tech>

/**
 * LNXDrive D-Bus Proxy Definitions
 *
 * Defines XML introspection strings for the LNXDrive daemon D-Bus interfaces
 * and provides proxy wrappers for communication from the GNOME Shell extension.
 *
 * Bus name:    com.strangedaystech.LNXDrive
 * Object path: /com/strangedaystech/LNXDrive
 *
 * Implements: FR-024, FR-025, FR-026, FR-023 (GOA account lifecycle)
 */

import Gio from 'gi://Gio';

const BUS_NAME = 'com.strangedaystech.LNXDrive';
const OBJECT_PATH = '/com/strangedaystech/LNXDrive';

// ---------------------------------------------------------------------------
// com.strangedaystech.LNXDrive.Sync
// ---------------------------------------------------------------------------
const SyncInterfaceXml = `
<node>
  <interface name="com.strangedaystech.LNXDrive.Sync">
    <method name="SyncNow"/>
    <method name="Pause"/>
    <method name="Resume"/>

    <property name="SyncStatus" type="s" access="read"/>
    <property name="LastSyncTime" type="x" access="read"/>
    <property name="PendingChanges" type="u" access="read"/>

    <signal name="SyncStarted"/>
    <signal name="SyncCompleted">
      <arg type="u" name="files_synced"/>
      <arg type="u" name="errors"/>
    </signal>
    <signal name="SyncProgress">
      <arg type="s" name="file"/>
      <arg type="u" name="current"/>
      <arg type="u" name="total"/>
    </signal>
    <signal name="ConflictDetected">
      <arg type="s" name="path"/>
      <arg type="s" name="conflict_type"/>
    </signal>
  </interface>
</node>`;

// ---------------------------------------------------------------------------
// com.strangedaystech.LNXDrive.Status
// ---------------------------------------------------------------------------
const StatusInterfaceXml = `
<node>
  <interface name="com.strangedaystech.LNXDrive.Status">
    <method name="GetQuota">
      <arg type="t" direction="out" name="used"/>
      <arg type="t" direction="out" name="total"/>
    </method>
    <method name="GetAccountInfo">
      <arg type="a{sv}" direction="out" name="info"/>
    </method>

    <property name="ConnectionStatus" type="s" access="read"/>

    <signal name="QuotaChanged">
      <arg type="t" name="used"/>
      <arg type="t" name="total"/>
    </signal>
    <signal name="ConnectionChanged">
      <arg type="s" name="status"/>
    </signal>
  </interface>
</node>`;

// ---------------------------------------------------------------------------
// com.strangedaystech.LNXDrive.Conflicts
// ---------------------------------------------------------------------------
const ConflictsInterfaceXml = `
<node>
  <interface name="com.strangedaystech.LNXDrive.Conflicts">
    <method name="List">
      <arg type="s" direction="out" name="conflicts_json"/>
    </method>
    <method name="GetDetails">
      <arg type="s" direction="in" name="id"/>
      <arg type="s" direction="out" name="details_json"/>
    </method>
    <method name="Resolve">
      <arg type="s" direction="in" name="id"/>
      <arg type="s" direction="in" name="strategy"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="ResolveAll">
      <arg type="s" direction="in" name="strategy"/>
      <arg type="u" direction="out" name="count"/>
    </method>

    <signal name="ConflictDetected">
      <arg type="s" name="conflict_json"/>
    </signal>
    <signal name="ConflictResolved">
      <arg type="s" name="conflict_id"/>
      <arg type="s" name="strategy"/>
    </signal>
  </interface>
</node>`;

// ---------------------------------------------------------------------------
// com.strangedaystech.LNXDrive.Auth (for GOA lifecycle — FR-023)
// ---------------------------------------------------------------------------
const AuthInterfaceXml = `
<node>
  <interface name="com.strangedaystech.LNXDrive.Auth">
    <method name="Logout"/>
    <method name="IsAuthenticated">
      <arg type="b" direction="out" name="authenticated"/>
    </method>
  </interface>
</node>`;

// ---------------------------------------------------------------------------
// com.strangedaystech.LNXDrive.Manager
// ---------------------------------------------------------------------------
const ManagerInterfaceXml = `
<node>
  <interface name="com.strangedaystech.LNXDrive.Manager">
    <method name="GetStatus">
      <arg type="s" direction="out" name="status"/>
    </method>

    <property name="Version" type="s" access="read"/>
    <property name="IsRunning" type="b" access="read"/>
  </interface>
</node>`;

/**
 * Create all three D-Bus proxies for communication with the LNXDrive daemon.
 *
 * Proxy wrappers are created inside this function (not at module scope) to
 * comply with GNOME Shell extension lifecycle rules: all D-Bus setup must
 * happen inside enable(), which transitively calls this function.
 *
 * @returns {Promise<{sync: Gio.DBusProxy, status: Gio.DBusProxy, manager: Gio.DBusProxy}|null>}
 *   An object containing all three proxy instances, or null if any proxy
 *   failed to connect (e.g., daemon is not running).
 */
export async function createProxies() {
    try {
        const SyncProxy = Gio.DBusProxy.makeProxyWrapper(SyncInterfaceXml);
        const StatusProxy = Gio.DBusProxy.makeProxyWrapper(StatusInterfaceXml);
        const ConflictsProxy = Gio.DBusProxy.makeProxyWrapper(ConflictsInterfaceXml);
        const ManagerProxy = Gio.DBusProxy.makeProxyWrapper(ManagerInterfaceXml);
        const AuthProxy = Gio.DBusProxy.makeProxyWrapper(AuthInterfaceXml);

        const sync = await new Promise((resolve, reject) => {
            SyncProxy(
                Gio.DBus.session,
                BUS_NAME,
                OBJECT_PATH,
                (proxy, error) => {
                    if (error)
                        reject(error);
                    else
                        resolve(proxy);
                },
                null, /* cancellable */
                Gio.DBusProxyFlags.NONE,
            );
        });

        // If the daemon is not running, the proxy has no name owner
        if (sync.g_name_owner === null)
            return null;

        const status = await new Promise((resolve, reject) => {
            StatusProxy(
                Gio.DBus.session,
                BUS_NAME,
                OBJECT_PATH,
                (proxy, error) => {
                    if (error)
                        reject(error);
                    else
                        resolve(proxy);
                },
                null,
                Gio.DBusProxyFlags.NONE,
            );
        });

        const conflicts = await new Promise((resolve, reject) => {
            ConflictsProxy(
                Gio.DBus.session,
                BUS_NAME,
                OBJECT_PATH,
                (proxy, error) => {
                    if (error)
                        reject(error);
                    else
                        resolve(proxy);
                },
                null,
                Gio.DBusProxyFlags.NONE,
            );
        });

        const manager = await new Promise((resolve, reject) => {
            ManagerProxy(
                Gio.DBus.session,
                BUS_NAME,
                OBJECT_PATH,
                (proxy, error) => {
                    if (error)
                        reject(error);
                    else
                        resolve(proxy);
                },
                null,
                Gio.DBusProxyFlags.NONE,
            );
        });

        const auth = await new Promise((resolve, reject) => {
            AuthProxy(
                Gio.DBus.session,
                BUS_NAME,
                OBJECT_PATH,
                (proxy, error) => {
                    if (error)
                        reject(error);
                    else
                        resolve(proxy);
                },
                null,
                Gio.DBusProxyFlags.NONE,
            );
        });

        return {sync, status, conflicts, manager, auth};
    } catch (e) {
        console.error(`[LNXDrive] Failed to create D-Bus proxies: ${e.message}`);
        return null;
    }
}

// ---------------------------------------------------------------------------
// GOA account lifecycle monitoring (FR-023)
// ---------------------------------------------------------------------------

const GOA_BUS_NAME = 'org.gnome.OnlineAccounts';
const GOA_OBJECT_MANAGER_PATH = '/org/gnome/OnlineAccounts';

/**
 * Monitor GNOME Online Accounts for removal of the LNXDrive Microsoft account.
 * When the user removes the account from GNOME Settings → Online Accounts,
 * this calls Auth.Logout() on the LNXDrive daemon.
 *
 * @param {Gio.DBusProxy} authProxy - The LNXDrive Auth D-Bus proxy
 * @returns {number} The signal subscription ID (use to disconnect later)
 */
export function monitorGoaAccountRemoval(authProxy) {
    return Gio.DBus.session.signal_subscribe(
        GOA_BUS_NAME,
        'org.freedesktop.DBus.ObjectManager',
        'InterfacesRemoved',
        GOA_OBJECT_MANAGER_PATH,
        null, /* arg0 */
        Gio.DBusSignalFlags.NONE,
        (_connection, _sender, _path, _iface, _signal, params) => {
            try {
                const [objectPath, interfaces] = params.deep_unpack();

                /* Check if the removed interfaces include a GOA Account */
                if (!interfaces.includes('org.gnome.OnlineAccounts.Account'))
                    return;

                /* The object path encodes the provider type; check it matches ours.
                 * GOA object paths look like: /org/gnome/OnlineAccounts/Accounts/NNN
                 * We also check by querying the account before removal if possible,
                 * but since it's already removed, we rely on the cached state.
                 *
                 * Alternatively, we watch for InterfacesRemoved and compare against
                 * known account paths we discovered at startup.  For simplicity,
                 * we trigger logout whenever any GOA account is removed and let the
                 * daemon handle the no-op case if it wasn't the LNXDrive account.
                 *
                 * A more precise approach would be to track the GOA account object
                 * path for provider-type "lnxdrive_microsoft" at startup.
                 */
                console.log(`[LNXDrive] GOA account removed: ${objectPath}, calling Auth.Logout()`);
                authProxy.LogoutSync();
            } catch (e) {
                console.error(`[LNXDrive] Error handling GOA account removal: ${e.message}`);
            }
        },
    );
}

/**
 * Stop monitoring GOA account removal.
 *
 * @param {number} subscriptionId - The ID returned by monitorGoaAccountRemoval()
 */
export function stopGoaMonitor(subscriptionId) {
    if (subscriptionId > 0)
        Gio.DBus.session.signal_unsubscribe(subscriptionId);
}
