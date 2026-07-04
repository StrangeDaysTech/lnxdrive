# Plan de actualización de la guía canónica `lnxdrive-guide/`

**Fecha:** 2026-07-04 · **Estado:** plan aprobado en dirección; se ejecuta doc a
doc tras cerrar la discusión de `06-catalogo-desviaciones.md` y
`07-milestones-capacidad.md`.

Principio rector: la guía vuelve a ser **verdadera** (describe el sistema real y
sus contratos reales) y **verificable** (sus hitos tienen criterios de
aceptación). Donde la implementación se desvió del plano arquitectónico
(decisión D1), la guía **conserva el plano** y añade el estado de migración.

---

## Fase B — cambios al canónico, doc por doc

| # | Documento | Cambio | Decisión ligada |
|---|---|---|---|
| B1 | `09-Referencia/02-hoja-de-ruta.md` | **Reescritura mayor**: fases-checkbox → milestones por capacidad (M0–M6 + v0.2 epics + v1.0), cada uno con criterios de aceptación y guion de verificación. Los "Hitos Principales" (MVP/1.0/2.0 por fases) se retiran. Fuente: `07-milestones-capacidad.md` | D4 |
| B2 | `08-Distribucion/02-comunicacion-dbus.md` | **Reescritura**: documentar el contrato D-Bus **real** (las 9 interfaces implementadas: Manager, Sync, Files, Status, Auth, Settings, Conflicts + legacy SyncController/Account), marcando cuáles son estables y cuáles legacy/por-deprecar. Especificar por fin el contrato de auth (hueco original: el XML canónico no tenía `.Auth`). Este doc es además el prerequisito declarado para reactivar las UIs de `experimental/` en v1.0 | — |
| B3 | `03-Arquitectura/*`, `05-Implementacion/03-convenciones-nomenclatura.md`, `07-Extensibilidad/04-artefactos-reutilizables.md` | **Una lista canónica de crates** (la de 12 de `08-Distribucion/01`); `lnxdrive-ratelimit`/`lnxdrive-state` pasan a "candidatos de extracción publicable (Fase 10)". Añadir tabla de **estado de migración**: qué crates están poblados, cuáles en migración (audit), cuáles nacen con su milestone (conflict en M6, telemetry en v0.2) | D1 |
| B4 | `06-Testing/01-estrategia-testing.md` | Añadir tier **E2E-real** (cuenta OneDrive de pruebas, `#[ignore]` + guiones de operador, patrón T101) y la regla "**mock no cierra hitos de capacidad**". Arreglar la referencia rota a `TRACE-risks-mitigations.md` | — |
| B5 | `04-Componentes/13-telemetria.md` | **Reescritura por redefinición**: el componente pasa a ser **auto-observación interna** (estado propio del sistema, avisos, reacciones). Se **elimina** todo el diseño de export externo (OTLP/gRPC → Google Cloud Run/BigQuery, Anonymizer, CLI `report send`). Se conserva: reports locales, métricas Prometheus **locales-only**, systemd unit de bajo impacto. Añadir la razón de la decisión: garantía al adoptante de que ningún dato sale de su PC | D3 |
| B6 | `04-Componentes/12-auditoria.md` | Aclarar la relación puerto/crate que la guía dejaba ambigua: el crate `lnxdrive-audit` delimita *código*; la persistencia sigue en la SQLite compartida vía `IStateRepository` (como el propio doc de adaptadores prescribe). Referenciar el epic de extracción (v0.2) | D1 |
| B7 | `04-Componentes/08-microsoft-graph.md` + `02-ui-gnome.md` | Documentar el **modelo de auth por plataforma**: GOA como vía GNOME (SSO nativo del escritorio), PKCE loopback daemon-owned como vía universal (CLI hoy; escritorios sin GOA en v1.0). Diagrama de secuencia del handshake UI→daemon→(GOA o navegador+callback)→keyring — pieza que la guía nunca tuvo. Consolidar redirect_uri canónico (`http://127.0.0.1:8400/callback`) | D2 |
| B8 | `07-Extensibilidad/02-puerto-icloudprovider.md` + `03-Arquitectura/02-capas-y-puertos.md` | Resolver la doble firma: el trait **implementado** se documenta como contrato v0.x; la firma rica (capabilities, provider_id, create_folder, move_item, get_quota) se marca como evolución objetivo para multi-proveedor (v1.0) | — |
| B9 | `MVP-CLOSURE-PLAN.md` | Archivar como histórico con nota: "cerró brechas de código, no capacidades; superado por milestones por capacidad" | D4 |
| B10 | Limpiezas menores | `lnxdrive-xfce` → `lnxdrive-gtk3`; residuos "NIMBUS"; extensión Nautilus = C (no "Python/C" ni "Python+GTK4"); revisar que ejemplos usen el namespace real | — |

### Orden de ejecución propuesto

1. **B1 + B4** (hoja de ruta + testing) — establecen el nuevo marco de avance.
2. **B5 + B6 + B3** (telemetría, auditoría, crates) — consolidan la decisión D1/D3.
3. **B7 + B2** (auth + contrato D-Bus) — pueden beneficiarse de lo aprendido al
   ejecutar M1 (el contrato real de auth se documenta mejor con el fix hecho).
4. **B8, B9, B10** — cierre.

Cada bloque = su propia rama `docs/` + PR, con AILOG correspondiente
(subdirectorio `guide/`).

---

## Fase C — gobernanza

| # | Acción | Nota |
|---|---|---|
| C1 | **Cerrar Charter-01 con honestidad** | Telemetría ex-post veraz: Fases 0–5 ejecutadas; Fase 6 (tag) **no ejecutada porque el gate era incorrecto** — el alpha se redefinió (D4). No es un fracaso de ejecución sino una corrección de criterio; así se registra |
| C2 | **Abrir Charter-02: "Road to functional v0.1"** | Fases = M0–M6. Cada fase = milestone de GitHub. Guiones de verificación como criterios de aceptación ex-ante |
| C3 | **ADRs/AIDECs para las decisiones** | D1 (delimitación de crates — restaurar plano), D2 (auth por plataforma: GOA/PKCE), D3 (telemetría interna-only, descarte de export externo). D3 en particular merece ADR formal: es una promesa de producto (privacidad) con implicaciones de arquitectura |
| C4 | **Crear los GitHub Milestones M0–M6** y asignar issues | Tras el triage M0 (los fantasmas se cierran, no se asignan) |
| C5 | **Registrar follow-ups nuevos** | Los huecos #2 (refresh token) y #4 (rutas de auth divergentes) del informe 02/05 aún no están en el registro de follow-ups — entran vía AILOG al abrir Charter-02, ligados a M5 y M1 |

---

## Qué NO cambia

- La arquitectura hexagonal, los puertos y el patrón daemon-común + UIs delgadas:
  **validados por la implementación**; la guía no necesita cirugía ahí.
- La visión multi-escritorio y multi-proveedor, y su calendario v1.0.
- El modelo de tracking (Issues como risk backlog público, sin Project board).
- El versionado SemVer como *mecánica* — solo se le añade el gate funcional que
  no tenía.

---

*new-guide · documento de trabajo — no canónico.*
