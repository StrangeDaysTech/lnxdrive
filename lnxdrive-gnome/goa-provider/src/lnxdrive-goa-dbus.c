/* lnxdrive-goa-dbus.c — D-Bus helper for token handoff to the daemon
 *
 * Copyright 2026 Strange Days Tech <https://strangedays.tech>
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include "lnxdrive-goa-dbus.h"

#include <gio/gio.h>

#define LNXDRIVE_BUS_NAME   "com.strangedaystech.LNXDrive"
#define LNXDRIVE_OBJECT_PATH "/com/strangedaystech/LNXDrive"
#define LNXDRIVE_AUTH_IFACE  "com.strangedaystech.LNXDrive.Auth"

/**
 * Creates a synchronous D-Bus proxy to the LNXDrive Auth interface.
 * Returns NULL on error (caller owns the ref).
 */
static GDBusProxy *
create_auth_proxy (GError **error)
{
    return g_dbus_proxy_new_for_bus_sync (G_BUS_TYPE_SESSION,
                                          G_DBUS_PROXY_FLAGS_NONE,
                                          NULL, /* GDBusInterfaceInfo */
                                          LNXDRIVE_BUS_NAME,
                                          LNXDRIVE_OBJECT_PATH,
                                          LNXDRIVE_AUTH_IFACE,
                                          NULL, /* GCancellable */
                                          error);
}

gboolean
lnxdrive_goa_dbus_complete_auth_with_tokens (const char *access_token,
                                              const char *refresh_token,
                                              gint64      expires_at_unix,
                                              GError    **error)
{
    g_return_val_if_fail (access_token != NULL, FALSE);
    g_return_val_if_fail (refresh_token != NULL, FALSE);

    g_autoptr (GDBusProxy) proxy = create_auth_proxy (error);
    if (proxy == NULL)
        return FALSE;

    g_autoptr (GVariant) result =
        g_dbus_proxy_call_sync (proxy,
                                "CompleteAuthWithTokens",
                                g_variant_new ("(ssx)",
                                               access_token,
                                               refresh_token,
                                               expires_at_unix),
                                G_DBUS_CALL_FLAGS_NONE,
                                5000, /* timeout ms */
                                NULL, /* GCancellable */
                                error);
    if (result == NULL)
        return FALSE;

    gboolean accepted = FALSE;
    g_variant_get (result, "(b)", &accepted);
    return accepted;
}

void
lnxdrive_goa_dbus_logout (void)
{
    g_autoptr (GError) error = NULL;
    g_autoptr (GDBusProxy) proxy = create_auth_proxy (&error);

    if (proxy == NULL) {
        g_warning ("LNXDrive GOA: cannot create Auth proxy for logout: %s",
                   error->message);
        return;
    }

    g_autoptr (GVariant) result =
        g_dbus_proxy_call_sync (proxy,
                                "Logout",
                                NULL,
                                G_DBUS_CALL_FLAGS_NONE,
                                5000,
                                NULL,
                                &error);
    if (result == NULL) {
        g_warning ("LNXDrive GOA: Auth.Logout call failed: %s",
                   error->message);
    }
}
