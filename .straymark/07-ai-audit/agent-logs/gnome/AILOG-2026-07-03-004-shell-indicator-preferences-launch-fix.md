---
id: AILOG-2026-07-03-004
title: "Fix: el ítem Preferences del indicador no abría nada (binario Flatpak fuera del host PATH)"
status: draft
created: 2026-07-03
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [8]
lines_changed: 95
files_modified:
  - lnxdrive-gnome/shell-extension/lnxdrive-indicator@strangedaystech.com/launcher.js
  - lnxdrive-gnome/shell-extension/lnxdrive-indicator@strangedaystech.com/menuItems.js
  - lnxdrive-gnome/shell-extension/lnxdrive-indicator@strangedaystech.com/prefs.js
  - lnxdrive-gnome/shell-extension/meson.build
observability_scope: none
tags: [gnome-shell, extension, bugfix, flatpak, host-side, preferences, launcher]
related:
  - AILOG-2026-07-03-002-shell-indicator-manual-install-and-service-activation
  - AILOG-2026-07-03-003-shell-extension-hostside-install-doc-and-metadata-cleanup
---

# AILOG: Fix — el ítem Preferences del indicador no abría nada

## Summary

Bug reportado por el operador: al abrir el menú del indicador del panel y hacer
clic en "Preferences", no se abría nada. Causa raíz: la extensión de Shell corre
**host-side** (en `gnome-shell`), pero lanzaba el panel con
`Gio.AppInfo.create_from_commandline('lnxdrive-preferences', …)` — un comando pelón
que **no está en el PATH del host**: el binario vive dentro del sandbox del Flatpak
(`/app/bin/lnxdrive-preferences`). El journal lo confirmó literal:
`[LNXDrive] Failed to launch preferences: Programa «lnxdrive-preferences» no
encontrado en $PATH`.

El mismo patrón roto estaba en **4 sitios**: el ítem Preferences, dos deep-links a
conflicts (`--page conflicts`) en `menuItems.js`, y el botón "Open" de las
preferencias de la extensión en `prefs.js`. Se corrigió con un módulo compartido
`launcher.js` (solo-Gio, importable desde el contexto Shell y el de prefs) que
resuelve el `.desktop` exportado por el Flatpak (cuyo `Exec` envuelve `flatpak
run …`) y lanza por ahí, con fallback al comando pelón para instalaciones
nativas/dev donde el binario sí está en el PATH.

## Context

Herencia directa de la integración host-side del indicador (AILOG-2026-07-03-002 /
-003). El binario `lnxdrive-preferences` se shipea **solo** dentro del Flatpak; el
`.desktop` exportado (`com.strangedaystech.LNXDrive.Preferences.desktop`) sí tiene
el `Exec` correcto: `/usr/bin/flatpak run --branch=master --arch=x86_64
--command=lnxdrive-preferences com.strangedaystech.LNXDrive`. La extensión no lo
usaba: invocaba el comando directo, que en el host no existe.

## Actions Performed

1. **Diagnóstico**: `which lnxdrive-preferences` → no está en el host PATH;
   journal muestra el `Failed to launch preferences` exacto al hacer clic;
   `--page` confirmado como opción real del panel (`preferences/src/app.rs:74`
   registra `--page`; `mod.rs:100` cambia a la página conflicts).
2. **`launcher.js`** (nuevo): módulo solo-Gio con `launchPreferences(page = null)`.
   Resuelve `Gio.DesktopAppInfo.new('com.strangedaystech.LNXDrive.Preferences.desktop')`;
   sin args, `.launch([], null)`; con args, reconstruye el comando desde el `Exec`
   del `.desktop` (ya `flatpak run …`), limpia field codes y añade `--page <page>`;
   fallback al comando pelón si no hay `.desktop` (instalación nativa/dev).
3. **`menuItems.js`**: los 3 sitios (`create_from_commandline`) reemplazados por
   `launchPreferences('conflicts')` (x2) y `launchPreferences()` (x1). Import
   `{launchPreferences}` añadido; `Gio` se conserva (sigue usándose 6×).
4. **`prefs.js`**: botón "Open" → `launchPreferences()`; import `Gio` eliminado
   (ya no se usaba tras el cambio).
5. **`shell-extension/meson.build`**: `launcher.js` añadido a la lista de `files()`
   para que se instale con el resto.

## Risk

- **Sin riesgo de regresión en instalación nativa**: el fallback conserva el
  comportamiento previo (comando pelón) cuando el binario está en el PATH.
- **Warning de deprecación benigno**: `Gio.DesktopAppInfo` emite un aviso en
  GNOME 50 (GLib 2.80+ movió la clase a `GioUnix.DesktopAppInfo`). Se mantiene
  `Gio.DesktopAppInfo` a propósito: `GioUnix.DesktopAppInfo` no existía en GNOME
  45/46, y la extensión declara compat 45–50. El warning no afecta la función.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `.../launcher.js` | +78 (nuevo) | Helper compartido de lanzamiento vía `.desktop` exportado |
| `.../menuItems.js` | +3/-27 | 3 sitios → `launchPreferences()`; import del helper |
| `.../prefs.js` | +3/-12 | Botón → `launchPreferences()`; import `Gio` eliminado |
| `.../meson.build` | +1 | `launcher.js` en la lista de instalación |

## Decisions Made

- **Módulo compartido solo-Gio** en vez de duplicar la lógica en 2 contextos:
  `menuItems.js` (Shell) y `prefs.js` (extension-prefs) no pueden compartir un
  módulo que importe `resource:///…/shell/…` ni Gtk/Adw, pero sí uno solo-Gio.
- **Resolver el `.desktop` en vez de hardcodear `flatpak run`**: packaging-agnostic
  — funciona tanto para el Flatpak (Exec envuelto) como para una instalación
  nativa futura, sin ramas condicionales por formato.
- **Preservar `--page conflicts`**: es una opción real de GApplication del panel;
  se reconstruye el comando para no perder el deep-link a la página de conflictos.

## Impact

- **Functionality**: "Preferences" y los deep-links a conflicts ahora abren el
  panel GTK4. Verificado end-to-end: lanzar vía el `.desktop` arranca el proceso
  `lnxdrive-preferences` en el sandbox.
- **Performance**: N/A.
- **Security**: N/A (mismo binario, mismo sandbox; solo cambia el mecanismo de
  invocación).
- **Privacy**: N/A.
- **Environmental**: N/A.

## Verification

- [x] `Gio.DesktopAppInfo.new('com.strangedaystech.LNXDrive.Preferences.desktop')`
  resuelve desde el host (probado con `gjs`); `get_commandline()` devuelve el
  `flatpak run …`; el comando reconstruido con `--page conflicts` es correcto.
- [x] Lanzamiento end-to-end vía `gtk-launch com.strangedaystech.LNXDrive.Preferences`
  → proceso `lnxdrive-preferences` arranca en el sandbox (ventana abre).
- [x] `node --check` limpio en `launcher.js`, `menuItems.js`, `prefs.js`.
- [x] `meson install` incluye `launcher.js` en el layout de la extensión.
- [x] `Gio` sigue importado en `menuItems.js` (6 usos) y eliminado de `prefs.js`
  (0 usos) — sin imports muertos.
- [ ] Confirmación en runtime por el operador tras recargar la sesión GNOME
  (Wayland: logout/login; el Shell solo relee el JS al iniciar sesión).

## Follow-ups

- Ninguno nuevo. La ruta de lanzamiento host-side queda cubierta por el `.desktop`
  exportado; documentación de instalación host-side ya añadida en
  AILOG-2026-07-03-003.

## Additional Notes

- La copia instalada del operador (`~/.local/share/gnome-shell/extensions/…`) se
  actualizó con los 3 archivos + `launcher.js` para prueba inmediata tras recargar
  sesión; el fix reproducible va en este PR.

---

<!-- Template: StrayMark | https://strangedays.tech -->
