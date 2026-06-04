---
id: AILOG-2026-06-04-001
title: "Fase 4 — Flatpak packaging: manifest rewrite, SPDX fix, metainfo completion"
status: draft
created: 2026-06-04
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security, value_chain]
iso_42001_clause: [8]
lines_changed: 510              # +447/-63 (git diff --shortstat del PR)
files_modified:
  - lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
  - lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in
  - lnxdrive.spdx
  - .straymark/charters/01-road-to-v0-1-0-alpha-1.md
observability_scope: none
tags: [packaging, flatpak, spdx, metainfo, appstream, charter-01, phase-4]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AIDEC-2026-06-04-001
  - AILOG-2026-05-31-002
---

# AILOG: Fase 4 — Flatpak packaging: manifest rewrite, SPDX fix, metainfo completion

## Summary

Fase 4 del Charter-01 (scope item 5): el manifiesto Flatpak pasa de un esqueleto
roto (repos git inexistentes, runtime 45 EOL, `command` apuntando a un stub, sin
install stages) a un manifiesto funcional que construye daemon + CLI + panel
GTK4 desde el monorepo, con sandbox scoped según la postura RISK-002.
`lnxdrive.spdx` deja de describir StrayMark (proyecto equivocado, licencia MIT
equivocada) y describe LNXDrive bajo GPL-3.0-or-later. El metainfo de
Preferences se completa con descripción extensa, screenshots y release
`0.1.0-alpha.1` (type development). **Verificado**: el bundle construye e
instala limpio con `org.flatpak.Builder` y los tres binarios + assets quedan
correctamente exportados en el sandbox.

## Context

El Charter-01 declara la Fase 4 como "Flatpak packaging + `lnxdrive.spdx` fix +
metainfo completion". La exploración previa (agente Explore, mapeo del estado
real antes de planificar) reveló que el manifiesto heredado no era completable
de forma incremental: sus sources apuntaban a dos repos git separados con tag
`v0.1.0` que nunca existieron (el proyecto es un monorepo sin tags), y su
segundo módulo construía el binario stub `lnxdrive-gnome` ("Not yet
implemented") en lugar del panel real `lnxdrive-preferences`. Se reescribió
completo — decisiones de arquitectura en [[AIDEC-2026-06-04-001]].

## Actions Performed

1. **Manifiesto** (`lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml`):
   - Runtime `org.gnome.Platform 45 → 49` (el "47" del Charter estaba EOL desde
     2025 — drift R8, ver AIDEC).
   - Sources `type: dir` relativos al manifiesto (monorepo local) con
     `skip: [target]`, en lugar de repos git inexistentes.
   - Módulo engine: `cargo build --release --locked -p lnxdrive-daemon -p
     lnxdrive-cli` → instala `lnxdrived` y `lnxdrive`.
   - Módulo gnome: buildsystem **meson** con `-Denable_nautilus=false
     -Denable_shell=false -Denable_preferences=true -Denable_goa=false` — el
     meson existente ya instala panel, iconos, `.desktop`, metainfo y schema
     GSettings (los "install stages" del Charter sin duplicación manual). Las
     extensiones Nautilus/Shell y el provider GOA son host-side y no pueden
     vivir en el sandbox.
   - `command: lnxdrive-preferences` (la GUI real; el daemon se lanza con
     `flatpak run --command=lnxdrived`).
   - Sandbox: se elimina `--socket=session-bus` en favor de
     `--own-name=com.strangedaystech.LNXDrive` +
     `--talk-name=org.freedesktop.secrets` +
     `--talk-name=org.gnome.OnlineAccounts` (superficie D-Bus mínima,
     coherente con RISK-002). `--filesystem=home` literal del Charter.
     `--device=all` para `/dev/fuse` (files-on-demand; sin clase más fina).
   - `build-args: --share=network` para cargo (ver Follow-ups: vendoring para
     Flathub).
2. **SPDX** (`lnxdrive.spdx`): reemplazo completo — describe LNXDrive (daemon,
   FUSE, CLI, integración GNOME), `PackageLicenseConcluded/Declared:
   GPL-3.0-or-later` (antes MIT, contradiciendo `LICENSE` y todos los crates),
   `PackageVersion: 0.1.0` (antes 1.0.0; la unificación a `0.1.0-alpha.1` es
   Fase 5), copyright alineado con `LICENSE`.
3. **Metainfo**
   (`lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in`):
   descripción de 3 párrafos + lista de features por página del panel, URLs
   `homepage`/`bugtracker`/`vcs-browser` apuntando al monorepo (antes:
   `lnxdrive-gnome` repo separado), 3 screenshots con nombres canónicos que la
   Fase 5 debe producir (`preferences-window.png`, `onboarding-wizard.png`,
   `conflict-dialog.png` bajo `docs/screenshots/`), release `0.1.0-alpha.1`
   `type="development"` con changelog (sustituye la entrada `0.1.0 /
   2026-02-05` que nunca correspondió a un release publicado).
4. **Atomic update del Charter** (formato v4): filas Fase 4 de la tabla
   `## Files to modify` — runtime 47→49 y ruta real del metainfo (la tabla lo
   ubicaba bajo `lnxdrive-packaging/flatpak/`; vive en el árbol meson de
   preferences). Scope item 5 anotado con el resultado de la fase.

## Risk

- **R8 (drift, documentado aquí y en el Charter)**: dos desviaciones de la
  declaración ex-ante — (a) runtime objetivo `org.gnome.Platform 47 → 49`
  porque 47 alcanzó EOL en 2025, antes incluso de la firma del Charter
  (2026-05-29); (b) la ruta del metainfo en la tabla Files-to-modify era
  incorrecta. Ambas corregidas atómicamente en este mismo PR
  ([[AIDEC-2026-06-04-001]]).
- El riesgo R2 del Charter (comportamiento del bundle bajo sandbox ≠ `cargo
  run`, en particular el mount FUSE) **sigue abierto por diseño**: su
  mitigación es el smoke-test en VM Fedora/Ubuntu previo al release (Fase 6),
  no esta fase.
- **Salida del drift check** (`check-charter-drift.sh`, rango
  `origin/main..HEAD`): los 26 archivos "declared but NOT modified" pertenecen
  a las demás fases del Charter (el script compara la tabla completa contra un
  PR de fase — esperado, no accionable). Los 3 "modified but NOT declared" son
  falsos positivos: el metainfo **sí** está declarado (fila corregida por el
  propio drift R8), `lnxdrive.spdx` **sí** está declarado (fila intacta de la
  tabla; límite del parser heurístico), y `.straymark/follow-ups-backlog.md`
  es el registro de governanza que debe viajar en el mismo commit que el AILOG
  (AGENT-RULES §13), no scope de producto.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` | reescritura completa | Runtime 49, sources dir monorepo, módulo meson, sandbox scoped, command real |
| `lnxdrive.spdx` | reescritura completa | Describe LNXDrive (antes StrayMark), GPL-3.0-or-later (antes MIT), versión 0.1.0 |
| `lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in` | +60/-6 aprox | Descripción completa, URLs monorepo, screenshots, release 0.1.0-alpha.1 |
| `.straymark/charters/01-road-to-v0-1-0-alpha-1.md` | ~6 líneas | Atomic update: filas Fase 4 + anotación scope item 5 (drift R8) |

## Decisions Made

Ver [[AIDEC-2026-06-04-001]] (runtime 49, sources dir, command, módulos
host-side excluidos, bus scoped, red en build). Decisión menor sin AIDEC: la
entrada de release `0.1.0 / 2026-02-05` del metainfo se sustituyó en lugar de
conservarse — nunca hubo release público con esa versión y AppStream la
mostraría como historial falso.

## Impact

- **Functionality**: primer artefacto de distribución construible del proyecto;
  base directa para `release.yml` (Fase 5).
- **Performance**: N/A.
- **Security**: superficie D-Bus del sandbox reducida (own-name/talk-name en
  vez de session-bus sin restricción); SPDX ahora declara la licencia real
  (GPL-3.0-or-later) — relevante para compliance de distribución; tokens
  siguen en keyring vía `--talk-name=org.freedesktop.secrets` (RISK-002).
- **Privacy**: N/A (sin cambios en manejo de datos).
- **Environmental**: N/A.

## Verification

- [x] `desktop-file-validate` sobre el `.desktop`: limpio (exit 0).
- [x] `appstreamcli validate --no-net` sobre el metainfo: **pass** (1 aviso
  pedante por mayúsculas en el app-id heredado, no accionable). Con red:
  3 warnings esperados `screenshot-image-not-found` — los PNG llegan en
  Fase 5 (nombres canónicos ya fijados, ver Follow-ups).
- [x] Build de verificación del Charter: `flatpak run org.flatpak.Builder
  --user --install --force-clean build-dir
  lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` — **construye
  e instala limpio** (commit `31106ca4`, 33.9 MB instalado, runtime
  org.gnome.Platform/x86_64/49).
- [x] Post-install en sandbox: `/app/bin/{lnxdrived,lnxdrive,
  lnxdrive-preferences}` presentes; `flatpak run --command=lnxdrive … --version`
  → `lnxdrive 0.1.0`; `.desktop`, `gschemas.compiled`, icono SVG y metainfo
  exportados en `/app/share/`.
- [x] `straymark validate` + `.straymark/scripts/check-charter-drift.sh`
  pre-commit (resultados en la descripción del PR).

## Follow-ups

- **Vendoring de crates para Flathub**: el manifiesto usa `build-args:
  --share=network` para que cargo descargue crates.io; una submission a
  Flathub exige sources vendorizados (`flatpak-cargo-generator`). Aplica
  cuando se decida publicar en Flathub (post-alpha, candidato v0.2).
- **`lnxdrive-packaging/README.md` desactualizado**: promete subdirectorios
  `rpm/`, `debian/`, `aur/`, `appimage/` que no existen (diferidos a
  v0.2.0-beta por el Charter). Alinear el README con la realidad del alpha
  (Flatpak only) — candidato a resolverse de paso en Fase 5 junto con el
  README raíz.
- **Nombres canónicos de screenshots para Fase 5**: el metainfo referencia
  `docs/screenshots/{preferences-window,onboarding-wizard,conflict-dialog}.png`;
  la Fase 5 debe producir exactamente esos nombres (más los 3 restantes del
  Charter para el README).

## Additional Notes

- El bundle del alpha **no incluye** la extensión de Nautilus, la extensión de
  Shell ni el provider GOA (componentes host-side). Documentarlo como
  limitación conocida en README/release notes es trabajo de Fase 5.
- Verificación de build ejecutada con `org.flatpak.Builder` (Flathub, user
  install) — el binario `flatpak-builder` nativo no está empaquetado en la
  máquina de verificación; el comando del Charter §Verification sigue siendo
  válido sustituyendo el prefijo por `flatpak run org.flatpak.Builder`.

---

<!-- Template: StrayMark | https://strangedays.tech -->
