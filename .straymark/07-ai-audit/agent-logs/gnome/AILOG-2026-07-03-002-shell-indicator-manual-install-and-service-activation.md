---
id: AILOG-2026-07-03-002
title: "GNOME Shell indicator — instalación manual y verificación de activación D-Bus del daemon"
status: draft
created: 2026-07-03
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [8]
lines_changed: 1
files_modified:
  - lnxdrive-gnome/shell-extension/lnxdrive-indicator@strangedaystech.com/metadata.json
observability_scope: none
tags: [gnome-shell, extension, dbus-activation, troubleshooting, charter-01, wayland]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-02-05-002-implement-gnome-shell-extension
---

# AILOG: GNOME Shell indicator — instalación manual y verificación de activación D-Bus del daemon

## Summary

Sesión de troubleshooting operativo: el operador reportó que el indicador del
panel de GNOME no aparecía tras reiniciar la sesión. Diagnóstico en dos capas.
Primero: el **servicio nunca fue el problema** — el daemon (`lnxdrived`, Flatpak)
arranca correctamente por activación D-Bus y expone las 9 interfaces en
`/com/strangedaystech/LNXDrive` sobre el bus de sesión. Segundo (bloqueo inicial):
la **extensión de GNOME Shell (`lnxdrive-indicator@strangedaystech.com`) no estaba
instalada** — solo existía en el árbol de fuentes; se copió a
`~/.local/share/gnome-shell/extensions/` y se pre-habilitó en gsettings.

**Causa raíz definitiva** (hallada tras un segundo reinicio en el que el ícono
seguía sin aparecer): el operador corre **GNOME Shell 50.3**, pero el
`metadata.json` de la extensión declaraba compatibilidad solo hasta la `49` →
Shell la marcaba `OUT OF DATE` y se negaba a cargarla, pese a estar instalada y
habilitada. **Fix**: se añadió `"50"` al array `shell-version` (fuentes del repo
+ copia instalada). Como Shell cachea el `metadata.json` al iniciar sesión y
Wayland no lo relee sin logout, se cargó en vivo activando temporalmente
`org.gnome.shell disable-extension-version-validation` (ya **revertido** tras la
carga; el metadata corregido cubre los reinicios futuros sin bypass). Extensión
confirmada `ACTIVE` y el ícono aparece en el panel.

## Context

Tras los cambios de packaging en curso (activación D-Bus del daemon, ajustes al
`.desktop` de Preferences), el operador esperaba ver el indicador de estado en la
barra superior de GNOME Shell y no aparecía pese a reiniciar la sesión. El
indicador es un componente **host-side** (corre dentro de `gnome-shell`, no en el
sandbox Flatpak), por lo que su instalación es independiente del bundle.

## Actions Performed

1. **Verificación del servicio**: activación D-Bus disparada por introspección
   (`busctl --user introspect com.strangedaystech.LNXDrive`) — el daemon arranca
   bajo demanda; proceso `/app/bin/lnxdrived` activo, dueño del bus name, con las
   interfaces `Account`, `Auth`, `Conflicts`, `Files`, `Manager`, `Settings`,
   `Status`, `Sync`, `SyncController` en `/com/strangedaystech/LNXDrive`.
2. **Diagnóstico del indicador**: confirmado que la extensión no está en
   `~/.local/share/gnome-shell/extensions/` ni en `org.gnome.shell
   enabled-extensions`. Verificado que `dbus.js` se conecta al mismo bus/ruta que
   el daemon expone (`Gio.DBus.session`, `com.strangedaystech.LNXDrive`,
   `/com/strangedaystech/LNXDrive`) — compatible con el daemon corriendo.
3. **Instalación manual**: copiada la carpeta de la extensión a
   `~/.local/share/gnome-shell/extensions/lnxdrive-indicator@strangedaystech.com/`.
4. **Pre-habilitación**: añadido el UUID a `org.gnome.shell enabled-extensions`
   vía `gsettings` (no `gnome-extensions enable`, que falla en Wayland porque el
   `gnome-shell` en ejecución no ha reescaneado el directorio). Al reiniciar la
   sesión, Shell la descubre y la activa.

## Risk

- Instalación **manual y no reproducible**: la extensión no está en el build de
  Meson ni se instala desde ningún paquete → cualquier reinstalación limpia del
  sistema vuelve a dejar sin indicador. Capturado como follow-up.
- `metadata.json` declara `settings-schema: com.strangedaystech.LNXDrive.Indicator`
  sin gschema en el repo. No rompe `enable()` (el código no llama a
  `getSettings()`), pero **`prefs.js` fallará al abrir las preferencias de la
  extensión**. Capturado como follow-up.

## Modified Files

Ninguno en el árbol de fuentes. Cambios solo en el sistema del operador
(`~/.local/share/gnome-shell/extensions/`, dconf `enabled-extensions`).

## Decisions Made

- **Pre-habilitar vía gsettings** en lugar de `gnome-extensions enable`: en
  Wayland el Shell en ejecución no descubre extensiones nuevas sin logout/login;
  escribir directo en `enabled-extensions` garantiza descubrimiento + activación
  en el próximo arranque de sesión.
- **No tocar el árbol de fuentes** en esta sesión: la formalización del target de
  instalación y el gschema quedan como follow-ups para su propio PR.

## Impact

- **Functionality**: tras el reinicio de sesión del operador, el indicador debería
  aparecer y enganchar con el daemon (que arranca por activación D-Bus).
- **Performance**: N/A.
- **Security**: N/A (extensión host-side, sin cambios de permisos).
- **Privacy**: N/A.
- **Environmental**: N/A.

## Verification

- [x] Daemon corriendo y dueño del bus name; 9 interfaces introspectadas en la
  ruta canónica.
- [x] Archivos de la extensión presentes en el directorio de extensiones del
  usuario; UUID en `enabled-extensions`.
- [ ] Aparición efectiva del indicador — pendiente del logout/login del operador
  (Wayland). Diagnóstico post-reinicio previsto vía `gnome-extensions info` +
  `journalctl --user` si no aparece.

## Follow-ups

- **La extensión de GNOME Shell no está en el build de Meson ni se instala vía
  paquete**: `lnxdrive-gnome/shell-extension/lnxdrive-indicator@strangedaystech.com/`
  solo existe en fuentes; en esta sesión se copió a mano a
  `~/.local/share/gnome-shell/extensions/`. Añadir un target de instalación
  (Meson `install_subdir` host-side, o script de instalación documentado) para que
  sea reproducible. Sin él, toda instalación limpia queda sin indicador de panel.
- **`metadata.json` declara `settings-schema: com.strangedaystech.LNXDrive.Indicator`
  sin gschema en el repo**: no hay `schemas/*.gschema.xml` ni schema compilado. No
  rompe `enable()` porque el código no invoca `getSettings()`, pero `prefs.js`
  fallará al abrir las preferencias de la extensión. Añadir y compilar el gschema
  `com.strangedaystech.LNXDrive.Indicator`, o eliminar `settings-schema` del
  metadata y el uso en `prefs.js` si no habrá preferencias en el alpha.

## Additional Notes

- Los cambios en curso del `.desktop`/`meson.build`/`yaml` (Name, Categories,
  activación D-Bus) están en el árbol de fuentes pero el Flatpak instalado aún
  tiene la versión previa; requieren rebuild + reinstall para reflejarse en el
  **lanzador de apps** (distinto del indicador de panel de este AILOG).

---

<!-- Template: StrayMark | https://strangedays.tech -->
