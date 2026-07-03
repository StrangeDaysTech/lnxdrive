---
id: AILOG-2026-06-04-002
title: "Fase 5 — Release infrastructure & public assets"
status: draft
created: 2026-06-04
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_integrity, value_chain]
iso_42001_clause: [8]
lines_changed: 656              # +550/-106 (git diff --shortstat del PR, sin contar el commit del Batch Ledger)
files_modified:
  - .github/workflows/release.yml
  - SECURITY.md
  - CHANGELOG.md
  - README.md
  - docs/screenshots/README.md
  - lnxdrive-packaging/README.md
  - lnxdrive-engine/Cargo.toml
  - lnxdrive-engine/Cargo.lock
  - lnxdrive-gnome/Cargo.toml
  - lnxdrive-gnome/Cargo.lock
  - lnxdrive-gnome/meson.build
  - lnxdrive-gnome/preferences/Cargo.toml
  - lnxdrive-gnome/preferences/Cargo.lock
  - lnxdrive.spdx
  - .straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md
  - .straymark/follow-ups-backlog.md
observability_scope: none
tags: [release, ci, packaging, security-policy, changelog, readme, versioning, charter-01, phase-5]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-06-04-001
  - AIDEC-2026-06-04-001
---

# AILOG: Fase 5 — Release infrastructure & public assets

## Summary

Fase 5 del Charter-01 (scope item 6): infraestructura de release y assets
públicos. Se crea el pipeline tag→bundle→Release (`release.yml`), `SECURITY.md`
(reporte privado + disclosure coordinado), `CHANGELOG.md` (Keep a Changelog,
entrada `0.1.0-alpha.1`), se unifica la versión a `0.1.0-alpha.1` en los 5
puntos de declaración + 3 Cargo.lock, y se actualiza el README raíz con
instalación real por Flatpak, galería de screenshots (6 nombres canónicos) y
tabla comparativa verificada contra el estado 2026 de jstaf/onedriver y
abraunegg/onedrive. Los 6 PNG los captura el operador en la VM Nivel-5
(decisión de esta sesión); el PR no se mergea sin ellos.

Adicionalmente se **backfillea el Batch Ledger** del Charter (process drift:
el Charter §Tasks mandaba `batch-complete` tras cada merge de fase, pero la
sección nunca se scaffoldeó en el AILOG de origen — Fases 0–4 registradas
retroactivamente desde el historial de merges con `--note`).

## Context

Con Fases 0–4 mergeadas, el proyecto tiene bundle construible pero cero
infraestructura de publicación: sin workflow de release, sin política de
seguridad, sin changelog, versiones `0.1.0` planas inconsistentes con el tag
objetivo `v0.1.0-alpha.1`, y un README raíz con secciones obsoletas
(instalación "Planned", quick-start de CLI con comandos inexistentes, roadmap
pre-Charter con todo "Planned").

## Actions Performed

1. **`.github/workflows/release.yml`** (nuevo): trigger en tags `v*`;
   construye con `flatpak-builder` nativo (apt, ubuntu-latest) + runtime 49 +
   rust-stable 25.08; verifica que el tag coincide con la versión del
   workspace (gate anti-desincronización); `flatpak build-bundle` →
   `lnxdrive.flatpak` + `SHA256SUMS`; publica GitHub Release con `gh` CLI
   (`--prerelease` automático si el tag lleva sufijo). **Sin actions de
   terceros** (solo `actions/checkout`) — postura supply-chain.
2. **`SECURITY.md`** (nuevo): reporte privado vía GitHub Security Advisories
   (primario) o `contact@strangedays.tech`; ack ≤7 días; disclosure
   coordinado ≤90 días; tabla de versiones soportadas (solo último alpha);
   resumen de postura (keyring, bus scoped, YAML hardening, cargo
   audit/deny).
3. **`CHANGELOG.md`** (nuevo): Keep a Changelog 1.1.0; entrada
   `0.1.0-alpha.1` con Added/Security/Known limitations; la fecha se fija al
   taggear (Fase 6).
4. **Unificación de versión** `0.1.0` → `0.1.0-alpha.1`: workspace del engine,
   `lnxdrive-gnome/Cargo.toml`, `preferences/Cargo.toml`, `meson.build`,
   `lnxdrive.spdx`; los tres `Cargo.lock` regenerados (`cargo update
   --workspace --offline`) — crítico porque el build Flatpak usa `--locked`.
   El lock del crate stub raíz (`lnxdrive-gnome`) se ajustó a mano: su
   git-dependency apunta a un remoto que no resuelve y nada lo construye
   (entrada idéntica a la que escribiría cargo). Verificado: `cargo check -p
   lnxdrive-daemon -p lnxdrive-cli` limpio; sin asserts de versión en
   tests/CI.
5. **`README.md` raíz**: badges (alpha + release), instalación real
   (`flatpak install --user <URL del bundle>` + verificación SHA256SUMS),
   galería de 6 screenshots con nombres canónicos, first-steps reales
   (wizard → GOA/browser → selective sync → daemon), quick-start del CLI con
   **comandos reales** (status/auth/mount/pin/dehydrate/explain — los
   anteriores `account add`/`ls`/`log` no existen; verificado contra el
   binario del Flatpak instalado), tabla comparativa vs jstaf/onedriver
   <!-- ERRATUM 2026-07-03 (auditoría Fases 4–5, H1): "comandos reales" es
        inexacto para `pin`/`dehydrate` — ambos parsean y corren pero son
        stubs que reportan la acción sin ejecutarla (pin.rs:51-99,
        hydrate.rs:172-236). El README se corrigió para marcarlos como no
        funcionales en el alpha; el wiring real (FUSE IPC) es v0.2. Ver
        review.md §4 P0 y el TDE de wiring. -->
   (v0.15.0, ene-2026, activo) y abraunegg/onedrive (v2.5.10, ene-2026,
   activo) con maturity honesta (alpha vs stable), limitaciones del alpha
   (componentes host-side), roadmap actualizado a milestones reales,
   diferenciador multi-provider re-calificado como *(roadmap)*.
6. **`docs/screenshots/README.md`** (nuevo): los 6 nombres canónicos +
   instrucciones de captura para el operador (VM Nivel-5, cuenta de prueba).
7. **`lnxdrive-packaging/README.md`**: realineado con la realidad del alpha
   (Flatpak only; rpm/debian/aur/appimage diferidos a v0.2.0-beta; comandos
   de build desde la raíz del monorepo; nota org.flatpak.Builder). Cierra
   **FU-004** del registro (recount en el mismo commit).
8. **Batch Ledger backfill** (commit previo de esta rama): sección añadida al
   AILOG de origen del Charter con Batches 1–7 (= Fases 0–6); Batches 1–5
   completados retroactivamente vía `straymark charter batch-complete
   --note` con PRs y fechas del historial.

## Risk

- **R9 (process drift, new, not in Charter)**: el mecanismo de Batch Ledger
  declarado en Charter §Tasks nunca se materializó en Fases 0–4 (la sección
  no existía en el AILOG de origen). Corregido con backfill retroactivo desde
  el historial de merges. Causa raíz: el scaffold del AILOG de origen
  (formato pre-v4) no incluía la sección y ningún gate lo detectó hasta el
  primer `batch-complete` real. El gate de cierre (`straymark charter drift`
  rechaza `(pending)`) protegerá las Fases 5–6.
- **R10 (scope note)**: el README raíz necesitó más que la "install section +
  comparison" declarada — quick-start con comandos inexistentes y roadmap
  pre-Charter eran contenido público falso a fecha de release; se corrigieron
  en esta fase como parte del espíritu "public assets". El movimiento del
  contenido previo no fue necesario (el README raíz ya era de producto; el
  del engine ya existía por separado).
- Los screenshots son **bloqueo de merge**: el PR queda en draft hasta que el
  operador deposite los 6 PNG (FU-005 se cierra entonces).
- **Salida del drift check** (rango `origin/main..HEAD`): los 21 "declared but
  NOT modified" pertenecen a otras fases (esperado en PR de fase). Los 9
  "modified but NOT declared" están todos cubiertos arriba: 3 `Cargo.lock` +
  `meson.build` + `preferences/Cargo.toml` + `lnxdrive.spdx` = unificación de
  versión (la fila del Charter "Every Cargo.toml with version=, all manifests,
  metainfo XML" los declara en agregado); `docs/screenshots/README.md` =
  soporte de la fila screenshots; `lnxdrive-packaging/README.md` = FU-004;
  `follow-ups-backlog.md` = registro §13 en el mismo commit.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `.github/workflows/release.yml` | +78 (nuevo) | Pipeline tag → bundle → Release + SHA256SUMS |
| `SECURITY.md` | +55 (nuevo) | Política de seguridad y disclosure |
| `CHANGELOG.md` | +60 (nuevo) | Entrada 0.1.0-alpha.1 |
| `README.md` | ~150 modificadas | Install real, screenshots, comparativa, roadmap, CLI real |
| `docs/screenshots/README.md` | +20 (nuevo) | Nombres canónicos para el operador |
| `lnxdrive-packaging/README.md` | reescritura | Flatpak only, formatos diferidos (FU-004) |
| 5 archivos de versión + 3 Cargo.lock | ~30 | Unificación 0.1.0-alpha.1 |
| AILOG origen Charter | +40 | Batch Ledger backfill (R9) |
| `.straymark/follow-ups-backlog.md` | ~10 | FU-004 closed + recount |

## Decisions Made

- **`gh` CLI + actions oficiales únicamente** en `release.yml` (sin
  `softprops/action-gh-release` ni equivalentes): menor superficie
  supply-chain; el runner ya trae `gh` autenticado con `github.token`.
- **Gate tag↔versión** en el workflow: aborta si `v<tag>` ≠ versión del
  workspace — previene releases con artefactos mal versionados.
- **Comparativa honesta**: maturity "Alpha" vs "Stable" explícita y
  recomendación directa de las alternativas para Business/SharePoint. Estado
  de competidores verificado por web search (2026-06), no entrenamiento.
- Screenshots: captura por el operador en VM Nivel-5 (opción elegida en
  sesión frente a captura automatizada o PR aparte).

## Impact

- **Functionality**: con esto, `git tag v0.1.0-alpha.1 && git push --tags`
  produce el release completo — Fase 6 queda reducida a tag + smoke + anuncio.
- **Performance**: N/A.
- **Security**: canal de reporte privado publicado; política de versiones
  soportadas explícita; pipeline sin actions de terceros.
- **Privacy**: screenshots con cuenta de prueba (instrucción explícita).
- **Environmental**: N/A.

## Verification

- [x] `cargo check -p lnxdrive-daemon -p lnxdrive-cli --offline` limpio tras
  el bump de versión + locks regenerados.
- [x] Comandos del README quick-start contrastados contra `lnxdrive --help`
  real (binario del Flatpak instalado en Fase 4).
- [x] Estado de jstaf/onedriver y abraunegg/onedrive verificado por web
  search (releases ene-2026, ambos activos).
- [x] `straymark validate` 0 errores; `followups recount` tras cierre de
  FU-004; drift check documentado en la descripción del PR.
- [ ] `release.yml` se ejercita end-to-end recién en Fase 6 (primer tag) —
  riesgo aceptado: el workflow replica los pasos ya verificados localmente
  en Fase 4 (mismo manifiesto, mismo runtime).
- [ ] 6 PNG del operador en `docs/screenshots/` (bloqueo de merge).

## Follow-ups

- **`lnxdrive-engine/config/lnxdrive-autostart.desktop` apunta a
  `/usr/bin/lnxdrive-daemon`**: el binario real se llama `lnxdrived` y en
  Flatpak vive en `/app/bin`. Sin efecto en el alpha (el autostart no se
  instala desde el bundle), pero corregir antes de empaquetar formatos
  nativos en v0.2.0-beta.

## Additional Notes

- FU-005 (nombres canónicos de screenshots) queda `open` hasta que los PNG
  estén en el árbol; se cierra en el triage de esta fase.
- La fecha del `CHANGELOG.md` y del `<release>` del metainfo se fijan al
  taggear (Fase 6).

---

<!-- Template: StrayMark | https://strangedays.tech -->
