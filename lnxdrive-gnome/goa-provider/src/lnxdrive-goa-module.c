/* lnxdrive-goa-module.c — GIO module entry point for the GOA provider
 *
 * Copyright 2026 Strange Days Tech <https://strangedays.tech>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This file is loaded by goa-daemon when it scans the GOA provider
 * module directory.  It registers the LNXDrive Microsoft provider type
 * and advertises it via the GOA extension point system.
 */

#include "lnxdrive-goa-provider.h"

void
goa_provider_module_init (void)
{
    lnxdrive_goa_provider_register_type (NULL);

    goa_provider_ensure_extension_points_registered ();
    g_io_extension_point_implement (GOA_PROVIDER_EXTENSION_POINT_NAME,
                                    LNXDRIVE_TYPE_GOA_PROVIDER,
                                    "lnxdrive_microsoft",
                                    0);
}
