---
id: AILOG-2026-06-01-001
title: Upgrade StrayMark framework fw-4.19.0 → fw-4.20.0 (POLISH-CHARTER-PATTERN v1, N=2)
status: accepted
created: 2026-06-01
agent: claude-opus-4-8-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
tags: [straymark, framework-upgrade, governance, polish-charter-pattern]
related:
  - POLISH-CHARTER-PATTERN
  - "straymark#209"
  - "straymark#210"
---

# AILOG: Upgrade StrayMark framework fw-4.19.0 → fw-4.20.0

## Summary

Versionado de la actualización del framework StrayMark vendored de `fw-4.19.0` a
`fw-4.20.0`, aplicada en disco con `straymark update-framework` (CLI `cli-3.18.0`).
El bump no es solo de versión: **crystalliza el `POLISH-CHARTER-PATTERN` de v0 (N=1)
a v1 (N=2)** y este repositorio (LNXDrive) es el **segundo dominio independiente** que
valida el patrón —un daemon de cloud-sync en Rust + escritorio GTK, frente al backend
Go de Sentinel que lo originó.

La graduación a N=2 provino de **dos findings que LNXDrive publicó upstream** en
`StrangeDaysTech/straymark`:

- **[straymark#209](https://github.com/StrangeDaysTech/straymark/issues/209)** — aportó la
  **sub-clase 5** del anti-patrón ("shipped-mitigation regression vía un consumidor
  downstream no actualizado") y cruzó el gate N=2 para el subcomando
  `straymark analyze declared-vs-wired`.
- **[straymark#210](https://github.com/StrangeDaysTech/straymark/issues/210)** — aportó la
  disciplina de reconocimiento de `charter new` ("READ antes de declarar") y la regla de
  validación `CHARTER-FILES-EXIST`.

## Context

LNXDrive surfaceó la sub-clase 5 durante Charter-01 Fase 3: el daemon D-Bus había cerrado
un riesgo de seguridad eliminando un método que entregaba tokens y publicando un reemplazo
token-safe, pero un componente separado (un cliente de preferencias GTK, compilado con un
build system distinto —Meson vs Cargo) **seguía llamando al método eliminado** tras un
`#[cfg(feature = "goa")]` cuyo feature `Cargo.toml` nunca definió. El código compilaba
*fuera* por completo —código muerto que esquiva tanto CI como code review—. El contrato solo
se unía en runtime sobre el bus, así que ninguna suite de tests abarcaba ambos lados.

Esa experiencia, generalizada, es lo que se reportó upstream y motivó los cambios de
`fw-4.20.0`. Este AILOG deja el registro de governance del bump en el lado del adoptante.

## Actions Performed

1. Aplicado `straymark update-framework` (fw-4.19.0 → fw-4.20.0); cambios ya presentes en el
   working tree.
2. Creada rama `chore/straymark-fw-4.20.0` desde `main`.
3. Autorado este AILOG bajo un subdir nuevo `agent-logs/governance/` (el cambio es meta de
   framework, repo-wide; no encaja en `daemon/`/`gnome/`/`guide/`).
4. Verificación read-only (`straymark status`, `straymark validate`) antes de commit.
5. Commit `chore(straymark): ...` + PR a `main` enlazando #209/#210 como origen (sin `Closes`,
   porque son issues del repo upstream, no de este repo).

## Modified Files

Cambios traídos por `update-framework` (resumen por categoría — no se enumeran las 40 líneas):

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `.straymark/00-governance/POLISH-CHARTER-PATTERN.md` | +~30/-~14 | v0→v1 (N=2), sub-clase 5, créditos #209/#210 |
| `.straymark/templates/charter/charter-template.md` (+ i18n es, zh-CN) | +~80 | Guía de reconocimiento + listar todos los consumidores cross-component |
| `.{claude,codex,gemini}/skills/straymark-charter-new/SKILL.md` | +6 | Recordatorio de reconocimiento "READ antes de declarar" |
| `.straymark/config.yml` | +6 | Comentarios `complexity.threshold` y scope `china` |
| `.straymark/QUICK-REFERENCE.md` | New | One-pager simplificado |
| `.straymark/dist-manifest.yml` | +1/-1 | `version: 4.19.0` → `4.20.0` |
| Docs de governance + i18n (es, zh-CN) | +1/-1 c/u | Bump de footer `fw-4.19.0` → `fw-4.20.0` |
| `.straymark/.checksums.json` | regenerado | Checksums de distribución |
| `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursorrules`, `.cursor/rules/straymark.md`, `.github/copilot-instructions.md` | -1 c/u | Limpieza de whitespace tras `<!-- straymark:end -->` |
| `.straymark/07-ai-audit/agent-logs/governance/AILOG-2026-06-01-001-...md` | New | Este documento |

## Impact

- **Functionality**: Solo governance/plantillas/tooling-docs. Sin cambios de código de runtime.
- **Performance**: N/A.
- **Security**: N/A (el bump documenta una lección de seguridad ya mitigada en Fase 3; no
  introduce ni altera superficie de seguridad en este repo).
- **Privacy**: N/A.
- **Environmental**: N/A.

## Verification

- [x] `straymark status` → Framework `fw-4.20.0`, estructura 17/17 OK
- [x] `straymark validate` → sin errores con el AILOG nuevo incluido
- [x] Manual review performed
- [ ] Tests pass (N/A — sin cambios de código)

## Additional Notes

#209 y #210 son issues del repositorio **upstream** `StrangeDaysTech/straymark`, no de
este repo, por lo que el PR los referencia como enlaces sin `Closes #N`.

---

<!-- Template: StrayMark | https://strangedays.tech -->
