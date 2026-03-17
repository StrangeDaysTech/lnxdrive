/* lnxdrive-goa-provider.h — GOA OAuth2 provider for Microsoft/OneDrive
 *
 * Copyright 2026 Strange Days Tech <https://strangedays.tech>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Implements FR-019, FR-020: GOA provider registration and OAuth2 flow
 */

#pragma once

#define GOA_API_IS_SUBJECT_TO_CHANGE
#define GOA_BACKEND_API_IS_SUBJECT_TO_CHANGE

#include <goa/goa.h>
#include <goabackend/goabackend.h>

G_BEGIN_DECLS

#define LNXDRIVE_TYPE_GOA_PROVIDER (lnxdrive_goa_provider_get_type ())
G_DECLARE_FINAL_TYPE (LnxdriveGoaProvider, lnxdrive_goa_provider,
                      LNXDRIVE, GOA_PROVIDER, GoaOAuth2Provider)

void lnxdrive_goa_provider_register_type (GTypeModule *module);

G_END_DECLS
