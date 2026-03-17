/* lnxdrive-goa-dbus.h — D-Bus helper for token handoff to the daemon
 *
 * Copyright 2026 Strange Days Tech <https://strangedays.tech>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Thin wrapper over g_dbus_proxy_call_sync() to pass GOA-obtained tokens
 * to the LNXDrive daemon via com.strangedaystech.LNXDrive.Auth.
 */

#pragma once

#include <gio/gio.h>

G_BEGIN_DECLS

/**
 * lnxdrive_goa_dbus_complete_auth_with_tokens:
 * @access_token: OAuth2 access token obtained from GOA
 * @refresh_token: OAuth2 refresh token obtained from GOA
 * @expires_at_unix: Token expiration as Unix timestamp (seconds since epoch)
 * @error: (nullable): return location for a #GError
 *
 * Calls Auth.CompleteAuthWithTokens on the LNXDrive daemon to hand off
 * tokens obtained through GNOME Online Accounts.
 *
 * Returns: %TRUE if the daemon accepted the tokens, %FALSE on error
 */
gboolean lnxdrive_goa_dbus_complete_auth_with_tokens (const char *access_token,
                                                       const char *refresh_token,
                                                       gint64      expires_at_unix,
                                                       GError    **error);

/**
 * lnxdrive_goa_dbus_logout:
 *
 * Calls Auth.Logout on the LNXDrive daemon to revoke credentials.
 * Errors are logged but not propagated.
 */
void lnxdrive_goa_dbus_logout (void);

G_END_DECLS
