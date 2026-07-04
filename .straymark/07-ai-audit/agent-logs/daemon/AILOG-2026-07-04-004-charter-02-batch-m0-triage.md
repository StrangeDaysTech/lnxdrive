---
id: AILOG-2026-07-04-004
title: "CHARTER-02 batch M0: triage de re-verificación de los 23 issues del risk-analysis"
status: accepted
created: 2026-07-04
agent: claude-code-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [6, 8, 9]
lines_changed: 160
files_modified:
  - .straymark/charters/02-road-to-functional-v0-1.md
  - new-guide/09-guiones-verificacion.md
  - .straymark/follow-ups-backlog.md
observability_scope: none
tags: [charter-02, m0, triage, issues, risk-analysis, milestones]
related:
  - CHARTER-02-road-to-functional-v0-1
  - AILOG-2026-07-04-001
---

# AILOG: CHARTER-02 batch M0 — "Sé qué es verdad"

## Summary

Primer batch de CHARTER-02. Los 23 issues abiertos del risk-analysis
(escritos ex-ante, antes de implementar) fueron re-verificados contra el
código actual por 4 verificadores paralelos agrupados por tema
(storage/data-integrity, sync engine, FUSE/conflictos/estados,
seguridad/D-Bus/config). Resultado ejecutado en GitHub con aprobación previa
del operador. **Titular: ningún P0 real sobrevivió al triage** — de los 5 P0,
2 estaban resueltos, 1 describía un subsistema inexistente y 2 estaban
sobredimensionados (degradados a P2).

## Context

M0 existe por la regla del proyecto "verificar la premisa antes de asignar
alcance" y por la Desconexión 3 del replanteo (risk backlog desconectado de
milestones). Sospechas previas confirmadas: #9 y #14 eran fantasmas; #21 se
solapaba parcialmente con ISSUE-002 ya cerrado.

## Actions Performed

1. **Cerrados como RESUELTOS (4)**, con evidencia file:line en el comentario:
   - #9 (P0) — `sync_item.rs:776-779`: `Error → *` + limpieza de `error_info`.
   - #12 (P0) — xattrs no persistidos (computados on-demand, `xattr.rs:82-97`;
     `setxattr` denegado `filesystem.rs:2751-2790`): única fuente de verdad.
   - #14 (P1) — `engine.rs:438-472`: `410 Gone` → full resync, con test.
   - #24 (P2) — `rate_limit.rs:124-135`: atómico bajo Mutex, test `:850`.
2. **Cerrados como NO-APLICA-AÚN (3)**, con condición de reapertura:
   - #11 (P0) — no existe sistema de plugins (extensibilidad = traits
     compile-time); el propio Proposed Fix del issue es el estado de facto.
   - #17 (P1) — no existe patrón observer (único callback siempre `None`).
   - #26 (P2) — no existe eviction/LRU de watches.
3. **Asignados a milestones de capacidad (6)**: #18→M3 (parcial: ventana
   TOCTOU + sin estado `Dehydrating`); #15, #27→M5; #13, #19, #25→M6
   (#19/#25 vigentes-por-ausencia: nacen con la detección de conflictos).
4. **Asignados a paraguas (7)**: #8, #16, #20, #21, #22, #31→v0.2.0-beta;
   #29→v1.0.0 (decidir versionado D-Bus antes de congelar API).
5. **Backlog deliberado sin milestone (3)**: #23, #28, #30 (comentados).
6. **Degradaciones de prioridad (3)**, justificadas en comentario:
   #8 P0→P2 (WAL+busy_timeout mitigan; falta writer único global),
   #13 P0→P2 (WAL es crash-safe; falta `synchronous` + `integrity_check` —
   pérdida de última tx, no corrupción), #20 P1→P2 (session bus =
   defensa-en-profundidad; vector de tokens cerrado por RISK-002).
7. **#21 estrechado**: retitulado a path-confinement de config; el núcleo DoS
   (billion-laughs) ya estaba resuelto por Charter-01 Fase 1.
8. **Issue #66 creado** (= FU-016): "Daemon never refreshes the OAuth access
   token at runtime" → M5, P1/auth/bug. Confirmado por dos análisis
   independientes.
9. **Charter-02 → `in-progress`** (started_at 2026-07-04); guion de
   verificación M0 escrito en `new-guide/09-guiones-verificacion.md` (con
   esqueleto del guion M1); FU-016 actualizado con el issue #66.

## Risk

- Sin riesgo de código (triage + gobernanza).
- **R1 (proceso)**: cerrar issues P0 podría ocultar riesgo real si la
  verificación fue superficial. Mitigación: cada cierre lleva la evidencia
  file:line exacta y condición de reapertura; los veredictos provienen de 4
  verificadores independientes con acceso al cuerpo del issue y al código; el
  operador aprobó la tabla completa antes de ejecutar.
- **Hallazgo transversal** (ya conocido, ahora con foto exacta): el engine
  hace last-writer-wins sin detección de conflicto (`engine.rs:838-882`,
  sobrescribe local modificado sin comprobar) — ese es el trabajo de M6 y el
  habilitador de #19/#25.

## Modified Files

| File | Change |
|------|--------|
| `.straymark/charters/02-road-to-functional-v0-1.md` | status declared → in-progress; espejo actualizado |
| `new-guide/09-guiones-verificacion.md` | New — guion M0 (con resultado) + esqueletos M1–M6 |
| `.straymark/follow-ups-backlog.md` | FU-016: destino actualizado a issue #66 |

## Decisions Made

- **Los NO-APLICA-AÚN se cierran (no quedan en backlog abierto)**: mantener
  abiertos issues sobre subsistemas inexistentes infla el backlog con diseño
  hipotético. La condición de reapertura queda en el comentario de cierre.
- **Los backlog deliberado (#23/#28/#30) quedan abiertos sin milestone**:
  son deuda real de baja prioridad, no fantasmas.

## Impact

- **Functionality**: N/A (sin código).
- **Process**: risk backlog y milestones quedan conectados por primera vez;
  el estado GitHub post-batch: 17 issues abiertos (13 heredados con destino +
  #66 nuevo + 3 backlog), 7 cerrados hoy. La Desconexión 3 del replanteo
  queda cerrada.

## Verification

- [x] `gh issue list --state open --label from-risk-analysis` sin milestone →
  solo #23, #28, #30 (backlog deliberado).
- [x] Los 7 cierres llevan comentario con evidencia file:line.
- [x] Charter-02 `in-progress`; `straymark validate` 0 errores;
  `followups drift` en sync.
- [x] Milestone M0 de GitHub: el triage está completo; se cierra el milestone
  al mergear el PR de este batch (`straymark charter batch-complete` 0).

## Follow-ups

Ninguno nuevo. FU-016 actualizado (no nuevo); los hallazgos del triage viven
en los issues asignados.

## Additional Notes

- Siguiente batch: **M1 — "Puedo entrar"** (issues/FUs: #66 no, ese es M5;
  FU-017 rutas de auth divergentes; feature `goa` en el build; verificación
  de scopes GOA↔Graph = riesgo R1 del Charter con fallback PKCE).

---

<!-- Template: StrayMark | https://strangedays.tech -->
