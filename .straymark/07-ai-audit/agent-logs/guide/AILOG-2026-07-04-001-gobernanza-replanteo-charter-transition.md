---
id: AILOG-2026-07-04-001
title: "Gobernanza del replanteo: cierre honesto de Charter-01, Charter-02 (M0–M6), ADRs D1/D3, AIDEC D2, GitHub Milestones"
status: accepted
created: 2026-07-04
agent: claude-code-v1.0
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [6, 8, 9]
lines_changed: 1100
files_modified:
  - .straymark/charters/01-road-to-v0-1-0-alpha-1.md
  - .straymark/charters/CHARTER-01.telemetry.yaml
  - .straymark/charters/02-road-to-functional-v0-1.md
  - .straymark/02-design/decisions/ADR-2026-07-04-001-restaurar-delimitacion-crates.md
  - .straymark/02-design/decisions/ADR-2026-07-04-002-telemetria-interna-only.md
  - .straymark/07-ai-audit/decisions/AIDEC-2026-07-04-001-auth-por-plataforma-goa-gnome.md
  - .straymark/follow-ups-backlog.md
observability_scope: none
tags: [governance, charter, replanteo, milestones, adr, aidec]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - CHARTER-02-road-to-functional-v0-1
  - ADR-2026-07-04-001
  - ADR-2026-07-04-002
  - AIDEC-2026-07-04-001
---

# AILOG: Gobernanza del replanteo — transición Charter-01 → Charter-02

## Summary

Ejecución de las acciones de gobernanza C1–C5 del plan de replanteo
(`new-guide/08-plan-actualizacion-guia.md`, aprobado en PR #61). El proyecto
transita del esquema "hito = fases completadas + tag SemVer" (que dejó a
CHARTER-01 sin cierre natural: Fase 6 inejectuable con el login roto) al esquema
de **milestones por capacidad demostrable** (M0–M6), con las decisiones del
operador D1–D4 formalizadas en ADRs/AIDEC.

## Context

El replanteo del 2026-07-04 (`new-guide/06-catalogo-desviaciones.md`) diagnosticó
tres desconexiones: hitos sin criterio de capacidad, testing 100% mock, y risk
backlog desconectado de milestones. El operador tomó cuatro decisiones (D1–D4)
y ordenó "gobernanza primero, luego reescritura de la guía canónica".

## Actions Performed

1. **C1 — Cierre honesto de CHARTER-01**: `straymark charter close` con
   telemetría validada contra schema. `completed_as_planned: false`,
   `scope_changes: mayor` — Fases 0–5 ejecutadas con calidad auditada; **Fase 6
   (tag) NO ejecutada porque el gate era incorrecto** (checklist de features sin
   criterio de capacidad). Bloque `## Closing notes (2026-07-04)` añadido al
   Charter con la trazabilidad de la redefinición (D4). FU-010/FU-011 (tag-time)
   transferidos al cierre de CHARTER-02.
2. **C2 — CHARTER-02 declarado** (`02-road-to-functional-v0-1.md`, effort L,
   `status: declared`): fases = milestones M0–M6; superficie load-bearing
   declarada desde código LEÍDO (informes new-guide/01-05); riesgos R1–R5 con
   mitigaciones de umbral concreto; cierre gateado por el tag `v0.1.0-alpha.1`.
3. **C3 — Decisiones formalizadas**:
   - `ADR-2026-07-04-001` (D1): restaurar delimitación de crates —
     incremental, M6 nace en `lnxdrive-conflict`, extracción de audit en v0.2.
   - `ADR-2026-07-04-002` (D3): telemetría interna-only; export externo
     (OTLP→Google Cloud) formalmente descartado — promesa de privacidad absoluta
     y verificable.
   - `AIDEC-2026-07-04-001` (D2): GOA como vía de login GNOME; PKCE loopback
     como ruta universal; fallback definido si los scopes GOA no sirven (R1).
   - Ambos ADRs quedan `draft` + `review_required: true` para aprobación del
     operador.
4. **C4 — GitHub Milestones M0–M6 creados** (#4–#10), conservando los
   milestones-paraguas por versión (#1–#3). La asignación de issues a M-hitos es
   trabajo del batch M0 (los fantasmas se cierran, no se asignan).
5. **C5 — Follow-ups nuevos registrados** (sección §Follow-ups de este AILOG →
   registro vía `straymark followups drift --apply` en el mismo commit).

## Risk

- Sin riesgo de código: cambio 100% de gobernanza/documentación.
- **R1 (proceso)**: cerrar un Charter con `completed_as_planned: false` podría
  leerse como fracaso. Mitigado: las Closing notes y la telemetría explican que
  es corrección de criterio, con las Fases 0–5 auditadas como entregadas.

## Modified Files

| File | Change |
|---|---|
| `.straymark/charters/01-road-to-v0-1-0-alpha-1.md` | status → closed; espejo actualizado; `## Closing notes (2026-07-04)` |
| `.straymark/charters/CHARTER-01.telemetry.yaml` | New — telemetría de cierre validada (3 rondas de auditoría externa, 4 emergentes, 15 follow-ups) |
| `.straymark/charters/02-road-to-functional-v0-1.md` | New — Charter-02 declarado con fases M0–M6 |
| `.straymark/02-design/decisions/ADR-2026-07-04-00{1,2}-*.md` | New — D1 y D3 formalizadas (draft, review_required) |
| `.straymark/07-ai-audit/decisions/AIDEC-2026-07-04-001-*.md` | New — D2 formalizada |
| `.straymark/follow-ups-backlog.md` | +2 entradas (drift --apply) |

## Decisions Made

- Los milestones por versión de GitHub (#1–#3) **se conservan** como paraguas:
  M0–M6 miden capacidad; v0.2/v1.0 agrupan los epics diferidos. Un issue vive en
  el M-hito que bloquea o en el paraguas de versión si está diferido.
- Los ADRs se emiten como `draft` — la aprobación es del operador
  (`straymark approve`), consistente con los límites de autonomía de
  AGENT-RULES §3.

## Impact

- **Functionality**: N/A (sin código).
- **Process**: el avance del proyecto pasa a medirse por capacidades
  verificables contra OneDrive real; el risk backlog queda conectable a hitos.
- **Security/Privacy**: D3 convierte la promesa de privacidad en propiedad de
  diseño verificable (criterio de test futuro, plan B5).

## Verification

- [x] `straymark charter status CHARTER-01-road-to-v0-1-0-alpha-1` → closed,
  telemetría validada contra schema (`charter close` finalizado sin errores).
- [x] `straymark charter status CHARTER-02-road-to-functional-v0-1` → declared,
  originating AILOG = este documento.
- [x] `gh api repos/StrangeDaysTech/lnxdrive/milestones` → 10 milestones
  (3 versión + 7 capacidad M0–M6).
- [x] `straymark validate` limpio (0 errores) tras todos los cambios.
- [x] `straymark followups drift` en sync tras `--apply`.

## Follow-ups

- **Cablear el refresh de token en runtime del daemon**: `refresh_if_needed`
  (`lnxdrive-engine/crates/lnxdrive-core/src/usecases/authenticate.rs:201-252`)
  existe y nunca se invoca; el daemon crea el `GraphClient` una sola vez al
  arrancar (`lnxdrive-daemon/src/main.rs:232`) y no maneja 401 en el loop — la
  sesión muere a la ~1h. Trigger: CHARTER-02 batch M5. Destino: milestone M5
  (GitHub #9).
- **Unificar las rutas de autenticación divergentes**: el CLI usa
  `GraphAuthAdapter` directo exigiendo `app_id` de config
  (`lnxdrive-cli/src/commands/auth.rs:69-78`), mientras
  `AuthenticateUseCase::login` usa `DEFAULT_APP_ID` pero llama a
  `cloud_provider.authenticate()` que es un stub `bail!` en el provider real
  (`lnxdrive-graph/src/provider.rs:165`) — ese use case no autentica de verdad;
  además `redirect_uri` difiere entre rutas (`127.0.0.1:8400/callback` vs
  `localhost:8400`). Trigger: CHARTER-02 batch M1. Destino: milestone M1
  (GitHub #5); diseño según AIDEC-2026-07-04-001.

## Additional Notes

- Este AILOG es el `originating_ailog` de CHARTER-02.
- La reescritura de la guía canónica (Fase B, B1–B10) es el siguiente bloque de
  trabajo tras la aprobación de este PR; no forma parte de este AILOG.

---

<!-- Template: StrayMark | https://strangedays.tech -->
