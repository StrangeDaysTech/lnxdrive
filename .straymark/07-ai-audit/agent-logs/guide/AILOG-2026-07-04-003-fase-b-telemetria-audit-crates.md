---
id: AILOG-2026-07-04-003
title: "Fase B (B5+B6+B3): telemetría interna-only, aclaración puerto/crate de auditoría, lista canónica de crates"
status: accepted
created: 2026-07-04
agent: claude-code-v1.0
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: [6, 8]
lines_changed: 420
files_modified:
  - lnxdrive-guide/04-Componentes/13-telemetria.md
  - lnxdrive-guide/04-Componentes/12-auditoria.md
  - lnxdrive-guide/08-Distribucion/01-estructura-repositorios.md
  - lnxdrive-guide/05-Implementacion/03-convenciones-nomenclatura.md
  - lnxdrive-guide/07-Extensibilidad/04-artefactos-reutilizables.md
observability_scope: none
tags: [guide, telemetry, audit, crates, replanteo, fase-b]
related:
  - AILOG-2026-07-04-002
  - ADR-2026-07-04-001
  - ADR-2026-07-04-002
  - CHARTER-02-road-to-functional-v0-1
---

# AILOG: Fase B (B5+B6+B3) — telemetría, auditoría y lista canónica de crates

## Summary

Segundo bloque de la Fase B del plan (`new-guide/08-plan-actualizacion-guia.md`):
consolida las decisiones D1 (delimitación de crates) y D3 (telemetría
interna-only) en los documentos de componentes y de estructura de la guía
canónica. Tras este bloque, la guía deja de prescribir el export de telemetría a
Google Cloud, explica dónde vive realmente la auditoría, y tiene **una sola**
lista de crates (antes había dos divergentes y ninguna coincidía con el
workspace real).

## Context

Continuación de AILOG-2026-07-04-002 (B1+B4). Los ADRs que gobiernan este bloque
fueron aprobados formalmente por el operador (PR #63): ADR-2026-07-04-001
(restaurar delimitación de crates, incremental) y ADR-2026-07-04-002 (telemetría
interna-only, export externo descartado).

## Actions Performed

1. **B5 — `04-Componentes/13-telemetria.md` (reescritura completa)**:
   - Banner de decisión al frente (ADR-2026-07-04-002); el componente se
     re-especifica como **auto-observación interna**: catálogo inicial de
     señales de salud → umbral → aviso/acción local (fallos de sync repetidos,
     token por expirar, disco bajo, D-Bus degradado, backlog atascado, panics).
   - **Eliminado**: pipeline OTLP/gRPC, backend Google Cloud
     (Cloud Run/BigQuery/Cloud Trace), Anonymizer como componente de envío,
     `lnxdrive report send`, flujo de consentimiento opt-in (sin envío no hay
     nada que consentir), dependencia de `network-online.target` en la unidad
     systemd.
   - **Conservado**: crash/error reports locales (limpios de PII desde el
     origen — su destino voluntario típico es un issue público), CLI `report
     list/view/delete`, métricas Prometheus locales-only, propiedades de
     proceso de bajo impacto (`Nice=19`, `MemoryMax=50M`).
   - Riesgos actualizados: T1 (exfiltración) pasa a **eliminado por diseño**
     con tests de vigilancia (`test_no_external_connections`,
     `test_no_pii_in_reports`); T3 nuevo (fatiga de avisos).
2. **B6 — `04-Componentes/12-auditoria.md`**: nueva sección "Ubicación en el
   código: puerto vs crate" — tabla de tres planos (tipos de dominio en core /
   persistencia en SQLite compartida vía `IStateRepository` / motor en
   `lnxdrive-audit`, en migración v0.2) + nota de que la separación
   código/almacenamiento no contradice el diseño de puertos del propio doc.
3. **B3 — lista canónica de crates** (tres fuentes reconciliadas):
   - `08-Distribucion/01-estructura-repositorios.md` = **canónica** (11 crates,
     verificados contra el workspace real): se elimina `lnxdrive-watch` (nunca
     existió como crate; el watching vive en `lnxdrive-sync`), se anotan los
     folds (`ratelimit`→graph, `state`→cache) y el estado de
     conflict/audit/telemetry.
   - `05-Implementacion/03-convenciones-nomenclatura.md`: el diagrama y la
     tabla propios (7 crates + `apps/` inexistente + `lnxdrive-kde` con nombre
     viejo) se reemplazan por la tabla canónica de 11 + nota que remite a la
     fuente canónica.
   - `07-Extensibilidad/04-artefactos-reutilizables.md`: nota de que el
     catálogo es **aspiracional** (post-1.0) y su diagrama 14.5 pasa a "estado
     objetivo post-1.0" en lugar de fingir estado actual.

## Risk

- Sin riesgo de código (solo documentación).
- **R1 (proceso)**: el doc de telemetría reescrito describe funcionalidad aún
  no implementada (el crate es stub hasta v0.2). Mitigado: el doc lo declara
  ("se implementa en v0.2") y la hoja de ruta lo agenda en el epic estructural
  — es diseño ex-ante legítimo, ya no promesa desalineada.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `lnxdrive-guide/04-Componentes/13-telemetria.md` | ~330/-450 | Reescritura: interna-only (ADR-002) |
| `lnxdrive-guide/04-Componentes/12-auditoria.md` | +30/-1 | Sección puerto vs crate (ADR-001) |
| `lnxdrive-guide/08-Distribucion/01-estructura-repositorios.md` | +13/-14 | Lista canónica: sin `watch`, folds y estados anotados |
| `lnxdrive-guide/05-Implementacion/03-convenciones-nomenclatura.md` | +45/-45 | Diagrama/tabla propios → canónica de 11 + nota |
| `lnxdrive-guide/07-Extensibilidad/04-artefactos-reutilizables.md` | +22/-17 | Catálogo marcado aspiracional; 14.5 = objetivo post-1.0 |

## Decisions Made

- **Una sola fuente canónica de la lista de crates** (estructura-repositorios);
  los demás docs remiten a ella en vez de duplicarla — es la corrección
  estructural del defecto que originó la divergencia.
- **El catálogo de artefactos no se poda**: sigue siendo la visión de
  extracción publicable; solo se le quita la ambigüedad de tiempo (aspiracional
  post-1.0, no presente).

## Impact

- **Functionality**: N/A (sin código).
- **Process**: D1 y D3 quedan consolidadas en los docs de componentes; la
  promesa de privacidad es ahora texto normativo de la guía.
- **Security/Privacy**: el diseño sin ruta de export queda documentado como
  propiedad estructural verificable.

## Verification

- [x] `straymark validate` → 0 errores tras los cambios.
- [x] `grep -in "opentelemetry\|otlp\|google cloud\|bigquery" lnxdrive-guide/04-Componentes/13-telemetria.md`
  → 3 líneas, todas nombrando lo **descartado**: 2 en el banner de decisión
  (describe qué diseño muere) y 1 en "no incluye cliente OTLP". Cero menciones
  como diseño vigente — ni en arquitectura, ni en código de ejemplo, ni en
  configuración.
- [x] Lista canónica contrastada contra el workspace real
  (`lnxdrive-engine/Cargo.toml`, 11 miembros — informes new-guide/01).
- [x] Links a ADRs verificados (mergeados en `main` vía PRs #62–#63).

## Follow-ups

Ninguno nuevo. Los bloques restantes de la Fase B (B2, B7–B10) siguen
planificados en `new-guide/08-plan-actualizacion-guia.md`.

## Additional Notes

- Siguiente bloque sugerido: B7+B2 (auth por plataforma + contrato D-Bus real).
  El plan recomienda ejecutarlos tras (o junto a) el batch M1 de CHARTER-02,
  porque el contrato de auth se documenta mejor con el fix hecho.

---

<!-- Template: StrayMark | https://strangedays.tech -->
