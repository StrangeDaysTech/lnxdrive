// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Strange Days Tech <https://strangedays.tech>

/**
 * Preferences launcher — shared by the Shell indicator (menuItems.js) and the
 * extension preferences window (prefs.js).
 *
 * Both callers run host-side (in gnome-shell and in the extension-prefs process),
 * where the `lnxdrive-preferences` binary is NOT on $PATH: it lives inside the
 * Flatpak sandbox (/app/bin). Launching a bare `lnxdrive-preferences` commandline
 * therefore fails with "not found in $PATH" and nothing opens.
 *
 * This module resolves the exported desktop entry
 * (com.strangedaystech.LNXDrive.Preferences.desktop) — whose Exec wraps
 * `flatpak run …` when packaged — and launches through it. For native/dev installs
 * where the binary IS on $PATH, it falls back to a bare commandline.
 *
 * Gio/GLib only — safe to import from both the Shell and the prefs contexts
 * (no `resource:///org/gnome/shell/...` or Gtk/Adw imports).
 */

import Gio from 'gi://Gio';

const PREFERENCES_DESKTOP_ID = 'com.strangedaystech.LNXDrive.Preferences.desktop';
const PREFERENCES_COMMAND = 'lnxdrive-preferences';

/**
 * Launch the LNXDrive preferences application, optionally on a specific page.
 *
 * @param {string|null} page - Page id to open (e.g. 'conflicts'), or null for the
 *     default page. Passed through as `--page <page>`, a known GApplication option
 *     of the preferences app.
 */
export function launchPreferences(page = null) {
    const extraArgv = page ? ['--page', page] : [];

    try {
        // Gio.DesktopAppInfo (not GioUnix.DesktopAppInfo): GLib 2.80+ emits a
        // deprecation warning on GNOME 50, but GioUnix.DesktopAppInfo did not exist
        // on GNOME 45/46 (GLib < 2.80). This extension targets Shell 45–50, so the
        // in-Gio accessor is the cross-version-safe choice; the warning is benign.
        const desktopApp = Gio.DesktopAppInfo.new(PREFERENCES_DESKTOP_ID);

        if (desktopApp) {
            // No extra args → launch the desktop entry directly (resolves the
            // `flatpak run` wrapper when packaged).
            if (extraArgv.length === 0) {
                desktopApp.launch([], null);
                return;
            }

            // DesktopAppInfo.launch() cannot append arbitrary args, so rebuild the
            // commandline from the entry's Exec (already `flatpak run …` when
            // packaged), strip desktop field codes, and append the page arg.
            const baseExec = desktopApp.get_commandline();
            if (baseExec) {
                const cleaned = baseExec.replace(/ *%[fFuUdDnNickvm]/g, '').trim();
                const cmd = `${cleaned} ${extraArgv.join(' ')}`;
                Gio.AppInfo.create_from_commandline(
                    cmd, 'LNXDrive Preferences', Gio.AppInfoCreateFlags.NONE,
                ).launch([], null);
                return;
            }
        }

        // Fallback: native/dev install with the binary on $PATH.
        const argv = [PREFERENCES_COMMAND, ...extraArgv].join(' ');
        Gio.AppInfo.create_from_commandline(
            argv, 'LNXDrive Preferences', Gio.AppInfoCreateFlags.NONE,
        ).launch([], null);
    } catch (e) {
        console.error(`[LNXDrive] Failed to launch preferences: ${e.message}`);
    }
}
