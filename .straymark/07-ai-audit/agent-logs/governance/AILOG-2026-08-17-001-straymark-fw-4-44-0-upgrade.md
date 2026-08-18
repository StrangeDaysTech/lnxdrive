---
id: AILOG-2026-08-17-001
title: Upgrade StrayMark framework fw-4.32.0 → fw-4.44.0 + Qoder agent registration (skills project-scoped + user-level)
status: accepted
created: 2026-08-17
agent: qoder-cli-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 8288
files_modified: []
observability_scope: none
tags: [straymark, framework-upgrade, governance, qoder, skills]
related:
  - AILOG-2026-06-01-001-straymark-fw-4-20-0-upgrade.md
---

# AILOG: Upgrade StrayMark framework fw-4.32.0 → fw-4.44.0 + Qoder agent registration

## Summary

Versionado de la actualización del framework StrayMark vendored de `fw-4.32.0` a
`fw-4.44.0`, aplicada en disco con `straymark update-framework` (CLI `cli-3.48.0`,
ya actualizado previamente en esta máquina). El bump trae 12 versiones de
framework intermedias; el cambio más relevante para este repo es el **soporte de
primera clase para Qoder CLI** —el agente desde cuyo frontend se operó esta
actualización—: el manifiesto de distribución ahora materializa `.qoder/skills/`
(project-scoped) y `AGENTS.md` recibe los marcadores de inyección
`<!-- straymark:begin/end -->`.

Como segundo paso se registraron los skills a nivel de usuario con la nueva
característica del CLI `straymark install-skills --agent qoder` (15 skills
copiados a `~/.qoder/skills/`), de modo que estén disponibles también fuera de
este proyecto.

## Context

El repo estaba en `fw-4.32.0`. Su `dist-manifest.yml` no incluía `.qoder/skills/`
ni la directiva para Qoder, y ni el directorio project-scoped ni el user-level
(`~/.qoder/skills/`) existían: los skills de StrayMark no estaban registrados para
el agente en uso. El CLI 3.48.0 ya soporta `install-skills --agent qoder`, pero
ese soporte solo es útil con un framework que materialice el origen
(`.qoder/skills/`), de ahí la necesidad del bump conjunto framework + registro.

## Actions Performed

1. Ejecutado `straymark update-framework` (fw-4.32.0 → fw-4.44.0): 192 archivos
   actualizados, 48 añadidos.
2. Verificado el nuevo `dist-manifest.yml` (v4.44.0): añade `.qoder/skills/`,
   `.qwen/skills/`, `.agent/skills/` a `files:` y `QWEN.md` a `injections:`;
   declara `retired:` para `.gemini/skills/` y `.agent/workflows/` (conservados
   en el repo, ya no instalados por StrayMark).
3. Verificada la materialización de `.qoder/skills/` con 15 skills
   (straymark-adr, aidec, ailog, architecture, architecture-sync, audit-execute,
   audit-prompt, audit-review, charter-new, followups, loom, mcard, new, sec,
   status); mismo set en `.qwen/skills/` y `.agent/skills/`.
4. `straymark install-skills --agent qoder --dry-run` (15 skills, 0 reemplazos)
   y luego la instalación real: 15 skills copiados a `~/.qoder/skills/`.
5. Confirmado que los skills se detectan en la sesión activa de Qoder (aparecen
   en el listado de skills disponibles sin reiniciar el repo).
6. Autorado este AILOG bajo `agent-logs/governance/` (cambio meta de framework,
   repo-wide; precedentes: AILOG-2026-06-01-001).

## Modified Files

Cambios traídos por `update-framework` (resumen por categoría — 59 archivos
tracked modificados, +859/-415, más ~7,015 líneas nuevas en untracked):

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `.straymark/dist-manifest.yml` | rewrite | v4.32.0 → v4.44.0: entradas `.qoder/.qwen/.agent` skills, `QWEN.md`, bloque `retired:` |
| `.qoder/skills/*/SKILL.md` (×15) | New (~2,200) | Skills project-scoped para Qoder — primera materialización |
| `.qwen/skills/*/SKILL.md` (×15) | New (~2,200) | Skills project-scoped para Qwen Code |
| `.agent/skills/*/SKILL.md` (×15) | New (~2,200) | Skills project-scoped para Antigravity |
| `QWEN.md` | New (+54) | Directiva de agente para Qwen Code |
| `.straymark/00-governance/AUDIT-ROUNDS-PATTERN.md` (+ i18n es, zh-CN) | New (+255) | Nuevo patrón de governance de rondas de auditoría |
| `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, `.cursor/rules/straymark.md`, `.github/copilot-instructions.md` | +marcadores | Bloque gestionado `<!-- straymark:begin/end -->` |
| `.straymark/00-governance/*` (+ i18n) | +~300 | Contenido de governance fw-4.44.0 (AGENT-RULES, DOCUMENTATION-POLICY, QUICK-REFERENCE, etc.) |
| `.straymark/templates/*` (+ i18n), `.straymark/schemas/*`, `STRAYMARK.md` | +~550 | Plantillas, schemas y reglas actualizadas |
| `.claude/skills/*`, `.codex/skills/*` | mod. | Refresh de SKILL.md a fw-4.44.0 |
| `.straymark/.checksums.json` | regenerado | Checksums de distribución |
| `.straymark/07-ai-audit/agent-logs/governance/AILOG-2026-08-17-001-...md` | New | Este documento |

## Decisions Made

- Se instaló el set de skills **también** a nivel de usuario (`~/.qoder/skills/`)
  además del project-scoped que ya lee Qoder: el help de `install-skills` aclara
  que en Qoder la instalación user-level solo aporta disponibilidad fuera de este
  proyecto, y se quiere esa portabilidad.
- No se eliminaron los artefactos retired (`.gemini/skills/`, `.agent/workflows/`):
  el propio update los conserva ("retired upstream, kept") y la decisión de
  borrarlos corresponde al operador, no al agente.

## Impact

- **Functionality**: Solo governance/plantillas/tooling-docs. Sin cambios de código de runtime.
- **Performance**: N/A.
- **Security**: N/A. La nueva superficie son archivos de instrucciones para agentes,
  gestionados por los marcadores de inyección del CLI.
- **Privacy**: N/A.
- **Environmental**: N/A.

## Verification

- [x] `straymark status` → Framework `fw-4.44.0`, CLI `cli-3.48.0`, estructura OK
- [x] `straymark install-skills --agent qoder --dry-run` inspeccionado antes de instalar
- [x] `~/.qoder/skills/` contiene 15 skills tras la instalación
- [x] Skills `straymark-*` visibles en la sesión activa de Qoder CLI
- [x] Manual review performed
- [ ] Tests pass (N/A — sin cambios de código)

## Additional Notes

La instrucción-set para Qoder queda registrado vía `AGENTS.md` (estándar abierto
que Qoder lee), ahora bajo gestión explícita de los marcadores
`<!-- straymark:begin/end -->`; no existe directiva `QODER.md` separada en
fw-4.44.0. Pendiente de decisión del operador: commit + PR de estos cambios
(regla §5 de STRAYMARK.md — nunca commit directo a `main`).

---

<!-- Template: StrayMark | https://strangedays.tech -->
