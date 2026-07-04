---
id: ADR-2026-07-04-002
title: "Telemetría interna-only: se descarta todo export externo de datos"
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
tags: [telemetry, privacy, observability, product-promise, replanteo]
related: [ADR-2026-07-04-001, CHARTER-02-road-to-functional-v0-1, AILOG-2026-07-04-001]
supersedes: []
---

# ADR: Telemetría interna-only — se descarta todo export externo de datos

## Status

draft — **decisión D3 tomada por el operador el 2026-07-04**; este ADR la
formaliza y requiere su revisión/aprobación.

**Note**: This document was created by an AI agent and requires human review.

## Context

El diseño original de telemetría (`lnxdrive-guide/04-Componentes/13-telemetria.md`)
abarcaba **dos aspectos distintos** que nunca terminaron de definirse como uno:

1. **Auto-observación**: que el propio sistema determine su estado para emitir
   avisos o responder con acciones (salud del daemon, métricas locales, crash
   reports en disco).
2. **Informe anonimizado al exterior**: export OTLP/gRPC hacia un backend
   OpenTelemetry en Google Cloud (Cloud Run → BigQuery + Cloud Trace), con
   Anonymizer y CLI `lnxdrive report send`, opt-in y desactivado por defecto.

El aspecto (2) estaba de facto casi descartado desde el diseño: **uno de los
objetivos del producto es garantizar al adoptante que ningún dato sale de sus
PCs hacia sistemas del proyecto**. La propuesta de valor de LNXDrive frente a
clientes propietarios se apoya en esa garantía. Mantener el export "opt-in" en
la guía dejaba una ambigüedad de promesa: el diseño decía "puede salir" mientras
el discurso decía "nunca sale". Nada de esto se implementó
(`lnxdrive-telemetry` es un stub), así que el costo de decidir ahora es cero.

## Decision

**La telemetría de LNXDrive es interna-only. Se descarta formalmente todo
mecanismo de export de datos hacia el exterior.**

1. `lnxdrive-telemetry` se re-especifica como **agente de auto-observación**:
   el sistema conoce su propio estado (salud del daemon, progreso/errores de
   sync, presión de recursos), emite avisos al usuario y puede disparar acciones
   correctivas locales.
2. Se **elimina del diseño** (guía, plan B5): pipeline OTLP/gRPC, backend Google
   Cloud (Cloud Run/BigQuery/Cloud Trace/Error Reporting), Anonymizer, y el
   subcomando `lnxdrive report send`.
3. **Se conserva**: crash/error reports **locales** en
   `~/.local/share/lnxdrive/reports/` (el usuario decide qué hacer con ellos,
   p. ej. adjuntarlos manualmente a un issue), métricas Prometheus **locales-only**
   (`127.0.0.1`, como ya prescribía la guía), logs JSON locales con retención.
4. La garantía se vuelve **verificable y absoluta**: ningún componente de
   LNXDrive abre conexiones salvo hacia el proveedor de nube configurado por el
   usuario. Esta propiedad pasa a ser criterio de test de seguridad
   (`06-Testing/09`) y texto de promesa en la documentación pública.

## Alternatives Considered

### A. Mantener el export anonimizado opt-in (diseño original)
- **Pros**: datos de calidad para priorizar bugs; estándar en la industria.
- **Cons**: contradice la garantía central del producto incluso siendo opt-in
  (la promesa "nada sale" deja de ser absoluta y se vuelve "nada sale si no
  aceptas"); costo de infraestructura (GCloud) y de mantenimiento del pipeline;
  superficie de riesgo de privacidad y cumplimiento.
- **Por qué no**: decisión del operador — la confianza del adoptante vale más
  que los datos; el aspecto ya estaba "casi descartado" desde el diseño.

### B. Reports locales con envío manual explícito (`report send` bajo demanda)
- **Pros**: punto medio; el usuario controla cada envío.
- **Cons**: sigue exigiendo backend receptor + anonimización auditada; la
  promesa absoluta se diluye igual ("hay un botón que envía").
- **Por qué no**: el valor marginal sobre "adjunta el archivo al issue de
  GitHub" no justifica ni la infraestructura ni la ambigüedad de promesa.

### C. Interna-only con artefactos locales (elegida)
- **Pros**: promesa absoluta y verificable; cero infraestructura; el aspecto (1)
  —el único con valor de producto directo— se conserva íntegro; los reports
  locales dan al usuario el material para reportar bugs voluntariamente.
- **Cons**: el proyecto no recibe señal pasiva de calidad desde instalaciones
  reales; la priorización de bugs depende de reportes voluntarios.

## Consequences

**Positivas**: la garantía de privacidad es demostrable por inspección de red
(criterio de test); desaparece el costo GCloud; `lnxdrive-telemetry` queda con
un alcance implementable en v0.2; coherencia total entre discurso y diseño.

**Negativas**: sin telemetría pasiva de campo — mitigado por el canal de issues
público y los reports locales adjuntables.

**Neutras**: el CLI conserva `lnxdrive report list/view/delete` (gestión local);
pierde `send`.

## Affected Components

| Componente | Impacto |
|---|---|
| `lnxdrive-guide/04-Componentes/13-telemetria.md` | Reescritura (plan B5): interna-only, sin backend externo |
| `lnxdrive-engine/crates/lnxdrive-telemetry/` | Se implementa en v0.2 con el alcance redefinido |
| `lnxdrive-guide/06-Testing/09-testing-seguridad.md` | Nuevo criterio: ninguna conexión saliente salvo al proveedor configurado |
| CLI (`lnxdrive report …`) | `send` se retira del diseño; gestión local se conserva |

## Implementation Plan

1. Plan B5 (`new-guide/08-plan-actualizacion-guia.md`): reescribir el doc de
   telemetría de la guía.
2. v0.2.0-beta: implementar `lnxdrive-telemetry` interna-only (junto al epic
   estructural de ADR-2026-07-04-001).
3. Añadir el test de "cero conexiones salientes no-proveedor" a la estrategia
   de seguridad.

## Success Metrics

- Guía sin ninguna referencia a OTLP/Google Cloud/Anonymizer tras B5.
- Test de red en la suite de seguridad: bajo operación completa (sync + FoD +
  panel), las únicas conexiones salientes observadas apuntan al proveedor de
  nube configurado.
- Documentación pública (README/landing) enuncia la garantía en absoluto:
  "ningún dato sale de tu equipo hacia nosotros — no hay mecanismo para ello".

---

<!-- Template: StrayMark | https://strangedays.tech -->
