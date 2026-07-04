# LNXDrive — Informes de estado (2026-07-03)

Análisis del estado real del código, verificado contra el repositorio
(`/home/montfort/StrangeDaysTech/lnxdrive`). Generado a partir de cuatro
exploraciones paralelas del código.

> **Propósito de `new-guide/`**: espacio de trabajo "en papel". Aquí se hará el
> replanteamiento del roadmap **antes** de alterar la guía canónica en
> `lnxdrive-guide/`. Estos archivos NO son documentación canónica todavía; son el
> borrador de análisis sobre el que se decidirán los cambios que luego se
> trasladarán a `lnxdrive-guide/`.

## Índice

| # | Archivo | Contenido |
|---|---------|-----------|
| 1 | [01-arquitectura-engine.md](01-arquitectura-engine.md) | Mapa de arquitectura del backend común (11 crates, hexagonal, D-Bus, puertos/adaptadores) |
| 2 | [02-estado-onedrive-msgraph.md](02-estado-onedrive-msgraph.md) | Estado de implementación del proveedor OneDrive / Microsoft Graph — hecho vs falta |
| 3 | [03-diagnostico-login.md](03-diagnostico-login.md) | Causa raíz del error de login en la app de preferences (URL OAuth sin `client_id`) |
| 4 | [04-hoja-de-ruta.md](04-hoja-de-ruta.md) | Roadmap alpha → beta → 1.0 y estado del Charter activo |
| 5 | [05-resumen-consolidado.md](05-resumen-consolidado.md) | Síntesis ejecutiva de los cuatro informes + próximos pasos |
| 6 | [06-catalogo-desviaciones.md](06-catalogo-desviaciones.md) | Catálogo de desviaciones guía↔código con veredictos + decisiones del operador D1–D4 (2026-07-04) |
| 7 | [07-milestones-capacidad.md](07-milestones-capacidad.md) | Replanteo de milestones: capacidades demostrables M0–M6, política de tags, tier E2E-real |
| 8 | [08-plan-actualizacion-guia.md](08-plan-actualizacion-guia.md) | Plan doc-por-doc (B1–B10) para actualizar el canónico `lnxdrive-guide/` + gobernanza (C1–C5) |
| 9 | [09-guiones-verificacion.md](09-guiones-verificacion.md) | **Documento vivo de CHARTER-02**: guiones de verificación por milestone (M0–M6), escritos al abrir cada batch |

## Ciclo de vida de esta carpeta (acordado 2026-07-04)

`new-guide/` cumple hoy tres papeles y **se retira al cerrar CHARTER-02** (tag
`v0.1.0-alpha.1` cortado):

1. **Artefacto activo del Charter**: `09-guiones-verificacion.md` es el registro
   vivo de los gates M0–M6 (declarado en el `Files to modify` del Charter y
   referenciado por la hoja de ruta canónica). Se actualiza en cada batch.
2. **Tracking del remanente de Fase B**: `08-plan-actualizacion-guia.md` es
   donde consta qué bloques de la actualización de `lnxdrive-guide/` faltan.
   Estado al 2026-07-04: B1, B3, B4, B5, B6 ✅ (PRs #64–#65) · B2+B7 diferidos
   deliberadamente a después del batch M1 (el contrato de auth se documenta con
   el fix hecho) · B8–B10 pendientes, sin dependencias.
3. **Registro histórico del replanteo**: los informes 00–05 (estado verificado
   del código al 2026-07-03) y 06–07 (catálogo de desviaciones + decisiones
   D1–D4) son la fuente de las decisiones. D1–D3 ya viven formalmente en
   ADR-2026-07-04-001/002 y AIDEC-2026-07-04-001; el catálogo completo solo
   vive aquí.

**Plan de retiro (último ítem del cierre de CHARTER-02)**: los guiones de
verificación se formalizan en `lnxdrive-guide/06-Testing/` (o
`lnxdrive-testing/`); los informes 00–08 se archivan como histórico — la
gobernanza ya vive en Charters/ADRs/AILOGs. Esta carpeta no debe sobrevivir al
Charter que la usa: un "espacio de trabajo" permanente se convierte en una
segunda guía divergente, que es exactamente la enfermedad que el replanteo curó.

## Estado en una línea

Proyecto en **v0.1.0-alpha.1, en la recta final** (Charter-01 `in-progress`, solo
falta cortar el tag en Fase 6). Arquitectura backend-común + UIs delgadas D-Bus
bien cimentada; OneDrive con transporte Graph casi completo, pero con 3 huecos que
impiden un uso end-to-end sin vigilancia: **login GUI no cableado**, **refresh de
token en runtime ausente**, y **detección de conflictos ausente**.
