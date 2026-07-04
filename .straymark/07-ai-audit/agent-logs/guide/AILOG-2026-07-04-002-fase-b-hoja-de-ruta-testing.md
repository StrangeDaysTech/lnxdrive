---
id: AILOG-2026-07-04-002
title: "Fase B (B1+B4): hoja de ruta canónica por capacidades + tier E2E-real en la estrategia de testing"
status: accepted
created: 2026-07-04
agent: claude-code-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [6, 8]
lines_changed: 260
files_modified:
  - lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md
  - lnxdrive-guide/06-Testing/01-estrategia-testing.md
observability_scope: none
tags: [guide, roadmap, testing, e2e-real, replanteo, fase-b]
related:
  - AILOG-2026-07-04-001
  - CHARTER-02-road-to-functional-v0-1
  - ADR-2026-07-04-001
  - ADR-2026-07-04-002
  - AIDEC-2026-07-04-001
---

# AILOG: Fase B (B1+B4) — hoja de ruta por capacidades y tier E2E-real

## Summary

Primer bloque de la Fase B del plan de actualización de la guía canónica
(`new-guide/08-plan-actualizacion-guia.md`): B1 (reescritura mayor de la hoja
de ruta) y B4 (estrategia de testing). Es el bloque que vuelve **oficial** el
replanteo en `lnxdrive-guide/`: los hitos pasan de "fases completadas por
checkbox" a **milestones por capacidad demostrable** con criterios de
aceptación, y el testing gana el tier **E2E-real** con la regla anti-mock.

## Context

La gobernanza del replanteo cerró en AILOG-2026-07-04-001 (Charter-01 closed,
Charter-02 declared, ADRs D1/D3 aprobados por el operador, AIDEC D2, GitHub
Milestones M0–M6). Orden del operador: "gobernanza primero y luego reescribir
la hoja de ruta canónica y demás". Este AILOG ejecuta el primer bloque de la
reescritura según el orden propuesto en el plan (B1+B4 establecen el nuevo
marco de avance).

## Actions Performed

1. **B1 — `09-Referencia/02-hoja-de-ruta.md` (reescritura completa)**:
   - Principio rector: milestone = capacidad demostrable contra el proveedor
     real; el tag es consecuencia. Las tres reglas (anti-mock, guion ex-ante,
     1–3 semanas).
   - Escalera M0–M6 con criterio de aceptación por milestone (fuente:
     `new-guide/07-milestones-capacidad.md`); política de tag para
     `v0.1.0-alpha.1`.
   - v0.2.0-beta como dos epics (distribuible + restaurar delimitación de
     crates); v1.0.0 (multi-escritorio con principio de auth por plataforma,
     multi-proveedor, i18n, multi-cuenta); horizonte post-1.0 (OneDrive
     avanzado, más proveedores, crates.io).
   - El backend de telemetría en Google Cloud que listaba la Fase 4 original
     **no se arrastra**: el epic v0.2 referencia telemetría interna-only
     (ADR-2026-07-04-002).
   - Sección `## Historial` con el mapa fase-vieja → destino-nuevo (las 11
     fases originales quedan trazables; la historia completa en git).
   - Retirados: checkboxes de features, "Hitos Principales" (MVP/1.0/2.0 por
     fases), estimación 9–12 meses y diagrama de dependencias del esquema viejo.
2. **B4 — `06-Testing/01-estrategia-testing.md`**:
   - Nueva sección §3 "El Tier E2E-real (gate de milestones)": definición
     (cuenta real de pruebas, tests `#[ignore]` + guiones de operador, sin CI —
     patrón T101), la regla anti-mock, y el criterio de verificación de la
     promesa de privacidad (ninguna conexión saliente no-proveedor,
     ADR-2026-07-04-002).
   - Los mocks conservan su papel documentado (CI de componentes) — la regla
     solo les quita el poder de cerrar hitos.
   - Referencia rota corregida: `../.straymark/...` → `../../.straymark/...`
     (el archivo `TRACE-risks-mitigations.md` SÍ existe en la raíz del repo;
     el path relativo tenía un nivel de menos). Sección renumerada §3→§4.

## Risk

- Sin riesgo de código (solo documentación).
- **R1 (proceso)**: docs de la guía que referencian la hoja de ruta vieja
  (fases numeradas) podrían quedar incoherentes hasta completar B2–B10.
  Mitigado: la sección `## Historial` mapea cada fase vieja a su destino, y los
  bloques restantes del plan (B3, B5–B10) barren las referencias cruzadas.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md` | ~180/-290 | Reescritura completa por capacidades |
| `lnxdrive-guide/06-Testing/01-estrategia-testing.md` | +55/-2 | Tier E2E-real (§3) + fix de referencia + renumeración |

## Decisions Made

- **Conservar la trazabilidad del esquema viejo** vía `## Historial` en lugar de
  borrar sin rastro: el mapa fase→destino evita que las referencias cruzadas de
  otros docs de la guía queden huérfanas mientras avanza la Fase B.
- **El tier E2E-real se documenta en la estrategia (§3), no como doc nuevo**:
  es una extensión del espectro de aislamiento existente, no una metodología
  aparte; un doc separado invitaría a ignorarlo.

## Impact

- **Functionality**: N/A (sin código).
- **Process**: la guía canónica y la gobernanza (Charter-02, Milestones) quedan
  alineadas — un lector de `lnxdrive-guide/` ya no encuentra el esquema
  superado como si fuera vigente.
- **Security/Privacy**: el criterio de "cero conexiones salientes no-proveedor"
  entra a la estrategia de testing (§3.3).

## Verification

- [x] `straymark validate` → 0 errores tras los cambios.
- [x] Referencia `TRACE-risks-mitigations.md` verificada existente en
  `.straymark/02-design/risk-analysis/` (raíz del repo) antes de corregir el
  path relativo.
- [x] Links internos de ambos docs apuntan a archivos existentes
  (ADRs/AIDEC/Charter-02 mergeados en PRs #62–#63; docs de la guía).
- [x] La hoja de ruta no contiene ya referencias al backend Google Cloud de
  telemetría ni a los "Hitos Principales" por fases.

## Follow-ups

Ninguno nuevo. Los bloques restantes de la Fase B (B2, B3, B5–B10) ya están
planificados en `new-guide/08-plan-actualizacion-guia.md` y no son follow-ups
emergentes de este trabajo.

## Additional Notes

- Siguiente bloque sugerido del plan: B5+B6+B3 (telemetría, auditoría, lista
  canónica de crates) — consolidan D1/D3 en los docs de componentes.

---

<!-- Template: StrayMark | https://strangedays.tech -->
