---
id: ADR-2026-07-04-001
title: "Restaurar la delimitación de funciones por crate (conflict, audit, telemetry)"
status: draft
created: 2026-07-04
updated: 2026-07-04
agent: claude-code-v1.0
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
iso_42001_clause: [6, 8]
alternatives_documented: []
api_changes: []
tags: [architecture, crates, hexagonal, technical-debt, replanteo]
related: [CHARTER-02-road-to-functional-v0-1, AILOG-2026-07-04-001]
supersedes: []
---

# ADR: Restaurar la delimitación de funciones por crate

## Status

draft — **decisión D1 tomada por el operador el 2026-07-04**; este ADR la
formaliza y requiere su revisión/aprobación.

**Note**: This document was created by an AI agent and requires human review.

## Context

El diseño original (`lnxdrive-guide/`) prescribe funciones **delimitadas por
crate**: `lnxdrive-conflict` y `lnxdrive-audit` como crates independientes
publicables (`07-Extensibilidad/04-artefactos-reutilizables.md`,
`05-Implementacion/03`, `08-Distribucion/01`) y `lnxdrive-telemetry` como
crate/proceso separado (`04-Componentes/13-telemetria.md`).

Durante la implementación **se ignoró ese plano**: la lógica de conflictos y
auditoría se incrustó como entidades de dominio dentro de `lnxdrive-core`
(`domain/conflict.rs`, `domain/audit.rs`) y los tres crates quedaron como stubs
de 7-8 líneas sin código ni consumidores. El catálogo de desviaciones del
replanteo (`new-guide/06-catalogo-desviaciones.md` §3) lo documenta con
evidencia. Fuerzas en juego: (a) la delimitación por crate hace las fronteras
de responsabilidad verificables y habilita la publicación futura (Fase 10);
(b) un crate vacío miente sobre la arquitectura; (c) mover código funcionando
es riesgo de regresión sin beneficio funcional inmediato; (d) los milestones
funcionales M1–M6 (CHARTER-02) son la prioridad del proyecto.

## Decision

**Se restaura el diseño de funciones delimitadas por crate, de forma
incremental y sin bloquear los milestones funcionales:**

1. Los crates `lnxdrive-conflict`, `lnxdrive-audit` y `lnxdrive-telemetry`
   **se conservan** en el workspace.
2. **La funcionalidad nueva nace en su crate**: la detección/resolución de
   conflictos (milestone M6 de CHARTER-02) se implementa en `lnxdrive-conflict`,
   consumiendo los tipos de dominio de `lnxdrive-core`.
3. **La funcionalidad existente migra como epic estructural de v0.2.0-beta**
   ("restaurar delimitación de crates"): extracción de la lógica de audit de
   core a `lnxdrive-audit`. La persistencia puede seguir en la SQLite compartida
   vía `IStateRepository` — el crate delimita *código*, no *almacenamiento*
   (coherente con el propio diseño de puertos de la guía).
4. `lnxdrive-telemetry` se implementa en v0.2.0-beta bajo la redefinición de
   ADR-2026-07-04-002 (interna-only).
5. Los tipos de *dominio* (entidad `Conflict`, `AuditEntry`, transiciones de
   estado) permanecen en `lnxdrive-core` — la arquitectura hexagonal no cambia;
   lo que se delimita son los *servicios/motores* sobre esos tipos.

## Alternatives Considered

### A. Borrar los stubs y actualizar la guía hacia el código
- **Pros**: honestidad estructural inmediata; cero refactor; git conserva la historia.
- **Cons**: abandona la intención de diseño original (fronteras verificables,
  publicación futura); consolida la deriva como hecho consumado.
- **Por qué no**: decisión explícita del operador — la desviación fue un
  accidente de implementación, no una evolución deliberada del diseño.

### B. Extraer todo inmediatamente (audit + conflict antes de M1)
- **Pros**: coherencia estructural total desde ya.
- **Cons**: semanas de refactor de código funcionando, con riesgo de regresión,
  antes de tener un sistema que haga login. Invierte las prioridades.
- **Por qué no**: el proyecto no tiene sistema funcional; la estructura no puede
  ir antes que la capacidad.

### C. Restauración incremental (elegida)
- **Pros**: lo nuevo nace bien ubicado (costo marginal ~0); lo existente migra
  cuando hay presupuesto; los milestones funcionales no se bloquean.
- **Cons**: ventana de incoherencia (audit en core hasta v0.2) — mitigada
  documentándola explícitamente en la guía (plan B3/B6).

## Consequences

**Positivas**: M6 entrega `lnxdrive-conflict` real; las fronteras de
responsabilidad vuelven a ser las del diseño; la publicación de crates (Fase 10)
sigue siendo alcanzable; la guía y el código convergen con un plan datado.

**Negativas**: hasta v0.2, `lnxdrive-audit`/`lnxdrive-telemetry` siguen siendo
stubs — la guía debe declarar el estado de migración (plan B3) para no mentir.

**Neutras**: la tabla de crates canónica pasa a ser la de 12 miembros de
`08-Distribucion/01`; `lnxdrive-ratelimit`/`lnxdrive-state` quedan como
candidatos de extracción de Fase 10, no deuda actual.

## Affected Components

| Componente | Impacto |
|---|---|
| `lnxdrive-engine/crates/lnxdrive-conflict/` | Deja de ser stub en M6 (CHARTER-02) |
| `lnxdrive-engine/crates/lnxdrive-audit/` | Recibe la lógica de `core/domain/audit.rs` en v0.2 |
| `lnxdrive-engine/crates/lnxdrive-telemetry/` | Se implementa en v0.2 según ADR-2026-07-04-002 |
| `lnxdrive-engine/crates/lnxdrive-core/` | Conserva tipos de dominio; cede servicios/motores |
| `lnxdrive-guide/` (docs B3/B6 del plan) | Documenta lista canónica + estado de migración |

## Implementation Plan

1. CHARTER-02 M6: detección de conflictos nace en `lnxdrive-conflict` (2026, v0.1).
2. v0.2.0-beta, epic estructural: extracción de audit; implementación de telemetry.
3. Guía actualizada según `new-guide/08-plan-actualizacion-guia.md` B3/B6.

## Success Metrics

- M6 cerrado con `lnxdrive-conflict` como dependencia real de `lnxdrive-sync`
  (ya no stub sin consumidores).
- Al cierre de v0.2: `grep -c "pub" crates/lnxdrive-audit/src/lib.rs` > 0 y
  `core/domain/audit.rs` reducido a tipos de dominio.
- `straymark analyze --contracts` (declared-vs-wired) sin hallazgos sobre los
  tres crates al cierre de v0.2.

---

<!-- Template: StrayMark | https://strangedays.tech -->
