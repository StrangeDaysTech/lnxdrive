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

## Estado en una línea

Proyecto en **v0.1.0-alpha.1, en la recta final** (Charter-01 `in-progress`, solo
falta cortar el tag en Fase 6). Arquitectura backend-común + UIs delgadas D-Bus
bien cimentada; OneDrive con transporte Graph casi completo, pero con 3 huecos que
impiden un uso end-to-end sin vigilancia: **login GUI no cableado**, **refresh de
token en runtime ausente**, y **detección de conflictos ausente**.
