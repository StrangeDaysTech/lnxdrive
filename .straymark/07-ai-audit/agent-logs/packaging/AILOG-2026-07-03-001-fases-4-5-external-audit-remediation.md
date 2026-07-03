---
id: AILOG-2026-07-03-001
title: "Fases 4–5 — external audit round + truthfulness remediation"
status: draft
created: 2026-07-03
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_integrity]
iso_42001_clause: [8, 9]
lines_changed: 39              # +25/-14 (copy + metadata remediation; audit artifacts committed separately)
files_modified:
  - README.md
  - SECURITY.md
  - CHANGELOG.md
  - lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in
  - lnxdrive-gnome/Cargo.toml
  - .straymark/07-ai-audit/agent-logs/packaging/AILOG-2026-06-04-002-fase-5-release-infrastructure-public-assets.md
  - .straymark/audits/CHARTER-01/review.md
  - .straymark/audits/CHARTER-01/external-audit-pending.yaml
  - .straymark/follow-ups-backlog.md
observability_scope: none
tags: [audit, external-audit, remediation, release-truthfulness, packaging, readme, security-policy, charter-01, phase-4, phase-5]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-06-04-001
  - AILOG-2026-06-04-002
---

# AILOG: Fases 4–5 — external audit round + truthfulness remediation

## Summary

Tercera ronda de auditoría externa del Charter-01 (fases-4-5: empaquetado
Flatpak + infraestructura de release), rango `31482c7..ae5a27d` (PRs #48+#49).
Tres auditores heterogéneos (gemini-3.1-pro-high, gpt-5-codex, qwen3.7-plus)
produjeron conjuntos de hallazgos **disjuntos** (sin contaminación). El
calibrador (claude-opus-4-8) verificó los **7 hallazgos VÁLIDOS** (0 falsos
positivos, 0 misatribuciones) y encontró **1 que los tres pasaron por alto** →
8 consolidados. Análisis completo en `.straymark/audits/CHARTER-01/review.md`.

El hallazgo bloqueante (H1, High) y dos Medium son de **veracidad de release**,
no de código: la copy pública (README, docs de seguridad, metainfo AppStream)
describía comportamiento/features que el alpha no entrega. Esta remediación
corrige la copy (fixes triviales, alto valor) y difiere el trabajo real de
código a v0.2.

## Changes

### Remediado en este PR (copy + metadata)

1. **H1 — README quick-start (`README.md`)**: `pin`/`dehydrate` se retiran del
   bloque ejecutable y se marcan explícitamente como *no funcionales en el
   alpha* (son stubs que reportan la acción sin ejecutarla —
   `lnxdrive-cli/src/commands/pin.rs:51-99`, `hydrate.rs:172-236`). El wiring
   real (FUSE IPC) es v0.2 → follow-up abajo.
2. **M1 — copy de seguridad (`SECURITY.md`, `CHANGELOG.md`)**: se elimina la
   afirmación de que "el API D-Bus expone session handles opacos" (diseño
   original que derivó y **no** se shippeó). El API real es
   `CompleteAuthViaGOA(goa_account_path) → bool/email` (Charter §Files:60); la
   única "session handle" del daemon es la del mount FUSE (`main.rs:54,265,300`).
   Reescrito a "recibe solo un object path de GOA no sensible y devuelve
   éxito/estado de cuenta".
3. **Missed-by-all — descripción `<release>` del metainfo**
   (`…Preferences.metainfo.xml.in:69-72`): anunciaba "GNOME Shell indicator,
   Nautilus overlays and GOA single sign-on" como contenido del bundle, que el
   manifiesto Flatpak (`…yaml:9-12`), README y CHANGELOG dicen que **excluye**.
   Reescrito a lo que el bundle realmente trae (daemon+FUSE, CLI, panel GTK4 con
   GOA sign-in; Shell/Nautilus host-side desde fuente).
4. **Erratum AILOG-2026-06-04-002**: el claim "comandos reales
   (…pin/dehydrate…)" se anota como inexacto (comentario ERRATUM inline),
   remitiendo a review.md §4 P0.
5. **RD-1 — `lnxdrive-gnome/Cargo.toml`**: `repository` corregido de
   `…/lnxdrive-gnome` (repo inexistente) a `https://github.com/StrangeDaysTech/lnxdrive`
   (monorepo; también corrige el casing).

### Diferido (follow-ups)

M2 (alineación de ids), RD-2 (dep git), RD-3/RD-4 (fechas/links de tag-time) y
el wiring de comandos FoD → ver §Follow-ups.

## Risk

- **R1 (new, not in Charter) — copy overstatement en assets de release.**
  Probabilidad media (assets escritos antes de tener el binario final a mano),
  severidad media (un alpha que afirma falsedades sobre lo que hace erosiona
  credibilidad). Mitigado: los tres defectos de veracidad se corrigieron a copy
  fiel antes del tag; el patrón "declared but not wired" ya está trackeado
  upstream (straymark#209).

## Follow-ups

- **Wiring de comandos files-on-demand (`pin`/`unpin`/`hydrate`/`dehydrate`) al
  motor FUSE vía IPC**: hoy son stubs que validan paths y reportan la acción sin
  ejecutarla (`lnxdrive-engine/crates/lnxdrive-cli/src/commands/pin.rs:51-99`,
  `hydrate.rs:55-60,172-236`). Necesita la IPC FUSE↔daemon que no existe en el
  alpha. Candidato a TDE; trigger v0.2.0-beta. Raíz de H1.
- **M2 — alinear el app-id del Flatpak (`com.strangedaystech.LNXDrive`) con el
  id de AppStream/desktop/GSettings-schema (`…Preferences`)**: el mismatch impide
  que GNOME Software asocie el metainfo con el ref instalado, y el cid dispara
  `cid-contains-uppercase-letter`. No es copy-only: toca el id del schema
  GSettings (runtime) + desktop + metainfo + launchable, y requiere validación
  con un build real de Flatpak. Trigger v0.2.0-beta / Flathub.
- **RD-2 — `lnxdrive-gnome` depende de `lnxdrive-ipc` vía git remoto**
  (`lnxdrive-gnome/Cargo.toml:21`) en lugar de path local. Bajo impacto (crate
  stub, no se construye en el bundle). Trigger ready; destino chore.
- **RD-3 — actualizar la fecha y el link `[0.1.0-alpha.1]` del `CHANGELOG.md`**
  (`:8`, `:59`) a la fecha real del tag. Trigger Fase 6 (tag-time).
- **RD-4 — actualizar `<release … date="2026-06-04">` del metainfo**
  (`…Preferences.metainfo.xml.in:66`) a la fecha real del tag. Trigger Fase 6
  (tag-time).

## Additional Notes

Round file layout: los archivos de esta ronda (`report-*.md`, `review.md`,
`external-audit-pending.yaml`) viven planos en `.straymark/audits/CHARTER-01/`;
las rondas cerradas (fase-1, fase-3) están en subcarpetas. Ver el `README.md` de
ese directorio y straymark#341. El bloque `external_audit:` para la telemetría
de cierre está en `external-audit-pending.yaml`. Ratings: gpt-5-codex 9.2 /
qwen3.7-plus 7.0 / gemini-3.1-pro-high 4.6.
