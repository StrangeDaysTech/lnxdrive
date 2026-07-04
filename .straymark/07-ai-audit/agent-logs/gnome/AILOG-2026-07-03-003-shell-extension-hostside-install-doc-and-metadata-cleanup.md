---
id: AILOG-2026-07-03-003
title: "Extensión de Shell: doc de instalación host-side (FU-013) + limpieza de metadata muerto (FU-014)"
status: draft
created: 2026-07-03
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [8]
lines_changed: 42
files_modified:
  - lnxdrive-gnome/shell-extension/lnxdrive-indicator@strangedaystech.com/metadata.json
  - lnxdrive-packaging/README.md
observability_scope: none
tags: [gnome-shell, extension, packaging, host-side-install, meson, metadata, follow-ups]
related:
  - AILOG-2026-07-03-002-shell-indicator-manual-install-and-service-activation
---

# AILOG: Extensión de Shell — doc de instalación host-side (FU-013) + limpieza de metadata muerto (FU-014)

## Summary

Cierre de los dos follow-ups nacidos del fix del indicador (AILOG-2026-07-03-002 /
PR #55): FU-013 (instalación de la extensión) y FU-014 (`settings-schema` muerto).

**Corrección de reconocimiento importante**: la premisa registrada de FU-013 —
"la extensión no está en el build de Meson" — resultó **falsa**. El
`lnxdrive-gnome/shell-extension/meson.build` **ya existía** (desde 2026-05-28) y
ya instala los 7 archivos en `datadir/gnome-shell/extensions/<uuid>/`. Verificado
en campo: `meson setup -Denable_shell=true` configura limpio y `meson install`
deposita la extensión correctamente. El error se originó en un reconocimiento
inicial defectuoso (un `find` mal construido reportó ausente un archivo presente)
— el modo de fallo #210 (autoría contra código asumido, no leído).

La **deuda real** de FU-013 era más estrecha: el target de Meson existe pero
**ningún flujo de instalación lo ejecuta host-side**. El Flatpak (único paquete
shipeado) fuerza `-Denable_shell=false` porque la extensión carga en el
`gnome-shell` del host, fuera del sandbox; y no había documentación de cómo
instalarla aparte. Por eso el indicador exigió un hand-copy manual en la sesión
previa. Resuelto documentando la instalación host-side por Meson (prefix de
usuario y de sistema) en el README de packaging.

FU-014 sí era real (se leyó `prefs.js` directamente): el `metadata.json` declaraba
`settings-schema: com.strangedaystech.LNXDrive.Indicator` sin gschema, y `prefs.js`
no llama a `getSettings()` (solo lanza la app externa `lnxdrive-preferences`).
Resuelto eliminando la clave muerta — no se necesita gschema.

## Context

En la sesión previa se declaró un Charter-02 ("extensión de Shell como componente
distribuible de primera clase") para abordar ambos follow-ups con encuadre amplio.
Al ejecutar la primera tarea (escribir el `meson.build`) se descubrió que **ya
existía y funcionaba**, lo que vació la premisa principal del Charter (target
faltante / build roto / gate CI para prevenir esa rotura). Con la verificación en
mano, el operador optó por **degradar el Charter a un PR pequeño** — alineado con
la preferencia de minimum-viable. El Charter-02 (nunca commiteado, solo declarado)
se abandonó y se eliminó del árbol; este AILOG es el registro honesto de esa
decisión y del trabajo real.

## Actions Performed

1. **Verificación de campo del target de Meson** (que refutó FU-013 tal como estaba
   escrito): `meson setup /tmp/lnx-build-shell -Denable_shell=true …` → exit 0;
   `DESTDIR=/tmp/lnx-install meson install` → los 7 archivos aterrizan en
   `usr/local/share/gnome-shell/extensions/lnxdrive-indicator@strangedaystech.com/`.
2. **FU-014** — eliminada la clave `settings-schema` del `metadata.json` (y la coma
   colgante de la línea previa). Confirmado JSON válido, `shell-version` conserva la
   `50` (del PR #55) y `settings-schema` ausente.
3. **FU-013 (versión real)** — añadida la sección "Componentes host-side (extensión
   de GNOME Shell)" a `lnxdrive-packaging/README.md`: por qué queda fuera del
   sandbox, instalación por Meson a prefix de usuario (`--prefix ~/.local`) y de
   sistema (`sudo meson install`), habilitación (`gnome-extensions enable`) y
   reinicio de sesión (Wayland), compat GNOME Shell 45–50, y nota de que el
   indicador se conecta al daemon por D-Bus de sesión.
4. **Abandono de Charter-02**: archivo `.straymark/charters/02-shell-extension-distributable.md`
   (untracked, nunca commiteado) eliminado; rama renombrada de
   `feature/shell-extension-distributable` a `fix/shell-extension-hostside-and-metadata`.

## Risk

- **R3 (heredado del Charter) — eliminar `settings-schema` rompería un futuro
  `prefs.js` con settings propios**: hoy no hay settings (el panel delega en
  `lnxdrive-preferences`). Si en el futuro la extensión gana preferencias propias,
  deberá añadir y **compilar un gschema real** (la lectura amplia original de
  FU-014), no reponer la clave suelta. Sin impacto en el alpha.
- **Sin riesgo de build**: el `meson.build` de la extensión ya existía y funciona;
  este cambio no lo toca. El Flatpak sigue con `-Denable_shell=false` a propósito.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `lnxdrive-gnome/shell-extension/.../metadata.json` | -1 | Eliminada clave muerta `settings-schema` (FU-014) |
| `lnxdrive-packaging/README.md` | +41 | Sección de instalación host-side de la extensión (FU-013 real) |

## Decisions Made

- **Degradar Charter-02 a PR pequeño**: la verificación refutó la premisa central
  del Charter (target faltante / build roto). Mantener la ceremonia de Charter para
  un one-liner + una sección de docs sería inflar el proceso. Decisión del operador,
  alineada con minimum-viable.
- **No crear gschema para FU-014**: `prefs.js` no usa settings; la corrección
  correcta es eliminar la clave, no materializar un schema sin consumidor.
- **No añadir gate CI**: sería para prevenir la regresión de un build que ya
  funciona — gold-plating para el alpha. Si el build se vuelve frágil, se reevalúa.

## Impact

- **Functionality**: una instalación limpia ahora tiene una ruta documentada para
  obtener el indicador (antes requería hand-copy no reproducible). Sin cambios de
  comportamiento en runtime.
- **Performance**: N/A.
- **Security**: N/A.
- **Privacy**: N/A.
- **Environmental**: N/A.

## Verification

- [x] `meson setup -Denable_shell=true` + `meson install` con el metadata modificado:
  los 7 archivos aterrizan; metadata instalado sin `settings-schema` y con
  `shell-version` incluyendo `50`.
- [x] `metadata.json` es JSON válido (`python3 -c "json.load(...)"`).
- [x] Sintaxis de los 5 `*.js` de la extensión: `node --check` limpio en todos.
- [x] Validez runtime en GNOME Shell 50 ya confirmada en AILOG-2026-07-03-002
  (extensión `ACTIVE`, indicador visible).
- [x] `straymark validate`: sin errores nuevos (el único error es el pre-existente
  de Charter-01 por resolución de AILOGs en subdirectorios).

## Follow-ups

- **Limitación del validador de StrayMark con AILOGs en subdirectorios por
  componente**: `straymark validate --include-charters` reporta `CHARTER-AILOG-REF`
  "missing AILOG" para AILOGs que existen pero viven en `agent-logs/<componente>/`
  (el layout que exige el CLAUDE.md del monorepo). Afecta a Charter-01 hoy. Es deuda
  del tooling StrayMark, no de este repo; evaluar reporte upstream.

## Additional Notes

- La extensión de Nautilus se instala por el mismo mecanismo Meson host-side
  (`-Denable_nautilus=true`), ya documentado de refilón en la nueva sección.
- El `disable-extension-version-validation` que se activó temporalmente en la sesión
  previa ya fue revertido; con `shell-version` incluyendo `50`, no vuelve a hacer falta.

---

<!-- Template: StrayMark | https://strangedays.tech -->
