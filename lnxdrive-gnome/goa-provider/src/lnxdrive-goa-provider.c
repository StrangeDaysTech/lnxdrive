/* lnxdrive-goa-provider.c — GoaOAuth2Provider subclass for Microsoft/OneDrive
 *
 * Copyright 2026 Strange Days Tech <https://strangedays.tech>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Implements FR-019: GOA provider registration
 * Implements FR-020: OAuth2 flow via GOA
 *
 * This provider appears as "Microsoft (LNXDrive)" in GNOME Settings →
 * Online Accounts.  After the user completes the OAuth2 flow, the tokens
 * are handed off to the LNXDrive daemon via D-Bus.
 */

#include "lnxdrive-goa-provider.h"
#include "lnxdrive-goa-dbus.h"

#include <glib/gi18n-lib.h>

/* Microsoft OAuth2 endpoints (consumers tenant) */
#define MS_AUTH_URI    "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize"
#define MS_TOKEN_URI   "https://login.microsoftonline.com/consumers/oauth2/v2.0/token"
#define MS_REDIRECT_URI "https://login.microsoftonline.com/common/oauth2/nativeclient"

/* Azure AD application (client) ID registered for LNXDrive */
#define MS_CLIENT_ID   "d50ca740-c83f-4d1b-b616-12c519384f0c"

/* OAuth2 scopes */
#define MS_SCOPE       "Files.ReadWrite.All User.Read offline_access"

struct _LnxdriveGoaProvider
{
    GoaOAuth2Provider parent_instance;
};

G_DEFINE_TYPE (LnxdriveGoaProvider, lnxdrive_goa_provider, GOA_TYPE_OAUTH2_PROVIDER)

/* -- GoaProvider vfuncs --------------------------------------------------- */

static const gchar *
get_provider_type (GoaProvider *self)
{
    return "lnxdrive_microsoft";
}

static gchar *
get_provider_name (GoaProvider *self, GoaObject *object)
{
    return g_strdup (_("Microsoft (LNXDrive)"));
}

static GIcon *
get_provider_icon (GoaProvider *self, GoaObject *object)
{
    return g_themed_icon_new ("com.strangedaystech.LNXDrive");
}

static GoaProviderGroup
get_provider_group (GoaProvider *self)
{
    return GOA_PROVIDER_GROUP_BRANDED;
}

/* -- GoaOAuth2Provider vfuncs --------------------------------------------- */

static const gchar *
get_authorization_uri (GoaOAuth2Provider *self)
{
    return MS_AUTH_URI;
}

static const gchar *
get_token_uri (GoaOAuth2Provider *self)
{
    return MS_TOKEN_URI;
}

static const gchar *
get_redirect_uri (GoaOAuth2Provider *self)
{
    return MS_REDIRECT_URI;
}

static const gchar *
get_client_id (GoaOAuth2Provider *self)
{
    return MS_CLIENT_ID;
}

static const gchar *
get_client_secret (GoaOAuth2Provider *self)
{
    /* Public client — no secret required (PKCE or implicit) */
    return NULL;
}

static const gchar *
get_scope (GoaOAuth2Provider *self)
{
    return MS_SCOPE;
}

/* -- build_object: token handoff to daemon -------------------------------- */

static GoaObject *
lnxdrive_build_object (GoaProvider  *provider,
                        GoaObjectSkeleton *object,
                        GKeyFile     *key_file,
                        const gchar  *group,
                        GDBusConnection *connection,
                        gboolean      just_added,
                        GError      **error)
{
    GoaObject *ret;

    /* Chain up to GoaOAuth2Provider's build_object first */
    ret = GOA_PROVIDER_CLASS (lnxdrive_goa_provider_parent_class)->build_object (
        provider, object, key_file, group, connection, just_added, error);

    if (ret == NULL)
        return NULL;

    /* Hand off tokens to the LNXDrive daemon when the account is first added */
    if (just_added) {
        GoaOAuth2Based *oauth2 = goa_object_peek_oauth2_based (ret);
        if (oauth2 != NULL) {
            g_autofree gchar *access_token = NULL;
            gint expires_in = 0;

            if (goa_oauth2_based_call_get_access_token_sync (oauth2,
                                                              &access_token,
                                                              &expires_in,
                                                              NULL, /* GCancellable */
                                                              error)) {
                /* Retrieve refresh token from the GOA key file */
                g_autofree gchar *refresh_token =
                    g_key_file_get_string (key_file, group,
                                           "refresh_token", NULL);

                if (refresh_token != NULL && access_token != NULL) {
                    gint64 expires_at_unix =
                        g_get_real_time () / G_USEC_PER_SEC + expires_in;

                    g_autoptr (GError) dbus_error = NULL;
                    if (!lnxdrive_goa_dbus_complete_auth_with_tokens (
                             access_token, refresh_token,
                             expires_at_unix, &dbus_error)) {
                        g_warning ("LNXDrive GOA: token handoff failed: %s",
                                   dbus_error ? dbus_error->message
                                              : "daemon rejected tokens");
                    }
                }
            }
        }
    }

    return ret;
}

/* -- GObject boilerplate -------------------------------------------------- */

static void
lnxdrive_goa_provider_init (LnxdriveGoaProvider *self)
{
}

static void
lnxdrive_goa_provider_class_init (LnxdriveGoaProviderClass *klass)
{
    GoaProviderClass *provider_class = GOA_PROVIDER_CLASS (klass);
    GoaOAuth2ProviderClass *oauth2_class = GOA_OAUTH2_PROVIDER_CLASS (klass);

    /* GoaProvider overrides */
    provider_class->get_provider_type = get_provider_type;
    provider_class->get_provider_name = get_provider_name;
    provider_class->get_provider_icon = get_provider_icon;
    provider_class->get_provider_group = get_provider_group;
    provider_class->build_object = lnxdrive_build_object;

    /* GoaOAuth2Provider overrides */
    oauth2_class->get_authorization_uri = get_authorization_uri;
    oauth2_class->get_token_uri = get_token_uri;
    oauth2_class->get_redirect_uri = get_redirect_uri;
    oauth2_class->get_client_id = get_client_id;
    oauth2_class->get_client_secret = get_client_secret;
    oauth2_class->get_scope = get_scope;
}

void
lnxdrive_goa_provider_register_type (GTypeModule *module)
{
    (void) lnxdrive_goa_provider_get_type ();
}
