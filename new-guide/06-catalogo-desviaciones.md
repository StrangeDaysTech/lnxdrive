# Catálogo de desviaciones — guía canónica vs implementación real

**Fecha:** 2026-07-04 · **Estado:** decisiones del operador registradas (sesión 2026-07-04)
**Fuentes:** informes 01-05 de esta carpeta + tres exploraciones de `lnxdrive-guide/`
(promesas arquitectónicas, hitos/testing, inventario de diferidos).

Este documento responde una pregunta por desviación: **¿qué lado debe moverse?**
Tres veredictos posibles: enderezar **código**, enderezar **guía**, o **decisión del
operador** (las cuatro decisiones tomadas se registran en §5).

---

## 1. Diagnóstico raíz: tres desconexiones

Por qué el proyecto acumuló avance real sin llegar a un sistema funcional:

### Desconexión 1 — hito ≠ capacidad
La guía define hitos como "fases completadas", y "completada" solo significa
"checkboxes de features marcados". En toda `lnxdrive-guide/` hay **cero**
ocurrencias de "criterio de aceptación", "definition of done" o "milestone"
(confirmado por barrido exhaustivo). La palabra "funcional" ("Sincronización
básica funcional", "Integración GNOME funcional") aparece **sin definición
operacional**. Cada pieza se marcó Done honestamente; el conjunto no funciona
porque nadie definió el conjunto como algo verificable.

### Desconexión 2 — el testing no puede falsificar "funciona"
Toda la pirámide (unit → integración → "E2E") corre contra `wiremock`/`mock-graph`.
En la guía, "entorno real" siempre significa *escritorio Linux real*, nunca *nube
real*. Ningún test del proyecto ha tocado la API productiva de Microsoft con una
cuenta real. El mock de VM (`mock-dbus-daemon.py`) fingía además el login completo
(devolvía URL falsa y auto-emitía `authenticated`) — por eso el onboarding se
validó como Done cuando el tramo daemon↔Microsoft nunca existió.

### Desconexión 3 — riesgo y roadmap corren sin tocarse
23 issues abiertos del risk-analysis (5 P0 de corrupción de datos, 8 P1) y
**ninguno asociado a milestone alguno**. Los P0 (divergencia SQLite↔FUSE,
corrupción WAL en corte de energía, races) definen "el sistema aguanta uso real",
y ningún tag los tenía como gate.

**Corolario:** el tag no funcionó como milestone porque mide lo que no importa
(checkboxes + SemVer de compatibilidad de API) y no mide lo que importa (¿puede un
usuario sincronizar un archivo real?).

---

## 2. Enderezar el CÓDIGO — funcionalidad faltante

| # | Desviación | Evidencia | Nota |
|---|---|---|---|
| C1 | **Login GUI roto** | `StartAuth` del daemon devuelve URL OAuth sin parámetros (`lnxdrive-ipc/src/service.rs:853`); el flujo PKCE completo existe pero solo cableado al CLI | Caso mixto: la guía prescribe OAuth PKCE + loopback + keyring (todo implementado), pero **nunca especificó el contrato D-Bus del login** — su XML canónico no tiene interfaz `.Auth`. Hay que arreglar el código Y escribir el contrato. Por decisión D2, la vía demostrable en GNOME es **GOA** |
| C2 | **Refresh de token en runtime** | `refresh_if_needed` existe (`authenticate.rs:201`) pero nadie lo llama; el daemon carga token una vez (`main.rs:232`) y muere a la ~1h | La guía prescribe `offline_access` — la intención siempre fue sesión persistente |
| C3 | **Detección de conflictos ausente** | Capítulo `11-conflictos.md` completo + Fase 5 de la hoja de ruta; en el código: nada (riesgo last-writer-wins, `engine.rs:1268` compara solo hash local vs almacenado) | Funcionalidad genuinamente faltante, con issues P1 asociados (#19, #25). Por decisión D1, se implementa **en `lnxdrive-conflict`** |
| C4 | **Comandos FoD del CLI son stubs** | `pin/unpin/hydrate/dehydrate` validan y reportan sin ejecutar (= FU-007 = los 4 únicos TODOs del engine) | Falta la IPC FUSE↔daemon; diferido a v0.2 |

## 3. Enderezar el CÓDIGO — estructura (decisión D1)

El plano arquitectónico original prescribe funciones **delimitadas por crate**
(`lnxdrive-conflict`, `lnxdrive-audit`, `lnxdrive-telemetry` — publicables o
proceso separado según el caso). Al implementar, esas funciones se incrustaron
en `lnxdrive-core` (dominio `conflict.rs`, `audit.rs`) y los crates quedaron como
stubs de 7 líneas. **Decisión del operador: se retoma el diseño original** — los
crates se conservan y las funciones migran/nacen en ellos.

| # | Crate | Estado actual | Plan (sin bloquear milestones funcionales) |
|---|---|---|---|
| E1 | `lnxdrive-conflict` | Stub vacío; entidad de dominio en core | **M6 implementa la detección/resolución directamente aquí** (la funcionalidad nueva nace en su casa), consumiendo los tipos de dominio de core |
| E2 | `lnxdrive-audit` | Stub vacío; lógica en core (`domain/audit.rs`), persistencia en SQLite compartida vía `IStateRepository` | Extracción como **epic estructural de v0.2** ("restaurar delimitación de crates"). Nota: no contradice el puerto — el crate delimita *código*, la persistencia puede seguir en la SQLite compartida como la propia guía prescribe |
| E3 | `lnxdrive-telemetry` | Stub vacío; nada implementado | Se implementa en v0.2 **bajo la redefinición D3**: solo auto-observación interna (ver §5) |

## 4. Enderezar la GUÍA

| # | Desviación | Veredicto |
|---|---|---|
| G1 | **Dos listas de crates divergentes** en la propia guía: `05-Implementacion/03` + `07-Extensibilidad/04` (7 crates, incl. `ratelimit`, `state`) vs `08-Distribucion/01` (12 crates, incl. `ipc/watch/sync/cache/telemetry`) | Consolidar a **una** lista canónica. Propuesta: la de `08-Distribucion/01` (12 crates — coincide 11/12 con lo implementado). `lnxdrive-ratelimit` y `lnxdrive-state` quedan solo como candidatos de extracción publicable en Fase 10 (hoy viven razonablemente dentro de `graph` y `cache`) |
| G2 | **`ICloudProvider` con dos firmas distintas** en la guía (`03-Arquitectura/02` abreviada vs `07-Extensibilidad/02` rica); el código implementa una tercera más simple | Documentar el trait real como **contrato v0.x**; la firma rica (capabilities, provider_id, create_folder, move_item, get_quota) queda como objetivo de evolución para multi-proveedor (v1.0) |
| G3 | **Contrato D-Bus desactualizado**: la guía define 5 interfaces (`Manager/Sync/Files/Status/Accounts`); el código implementa 9 (incl. `Auth`, `Settings`, 2 legacy). Ninguna descripción es verdad completa | Escribir el **doc de contrato D-Bus real** — además es requisito previo declarado para reactivar las UIs de `experimental/` en v1.0 |
| G4 | **Testing sin tier de nube real** — la guía nunca lo exigió | Añadir tier **E2E-real** (cuenta OneDrive de pruebas, tests `#[ignore]` + guiones manuales, patrón ya existente en el proyecto con FUSE/T101) + regla "mock no cierra hitos" |
| G5 | **Hoja de ruta por fases-checkbox** sin criterios de aceptación | Reescribir con milestones por capacidad (ver `07-milestones-capacidad.md`) |
| G6 | **`MVP-CLOSURE-PLAN.md`** cierra brechas de código y valida con `cargo check`, delegando la verificación funcional a un "después" nunca especificado | Archivar como histórico con nota |
| G7 | Menores: naming `lnxdrive-xfce` vs `lnxdrive-gtk3`; residuos del rebrand "NIMBUS"; extensión Nautilus descrita como "Python/C" vs "Python+GTK4" (es C en la realidad); referencia rota a `TRACE-risks-mitigations.md` en la estrategia de testing | Limpiezas puntuales |

## 5. Decisiones del operador (2026-07-04)

| ID | Decisión | Implicación |
|---|---|---|
| **D1** | **Los crates stub se conservan y se retoma el diseño de funciones delimitadas.** La implementación ignoró el plano arquitectónico e incrustó funciones en los crates principales; eso rompe la idea de diseño original y se corrige | §3 completo. M6 nace en `lnxdrive-conflict`; extracción de audit + implementación de telemetry = epic estructural v0.2 |
| **D2** | **GNOME usa GOA como vía de login.** Cada escritorio tendrá artefactos ejecutables/de configuración específicos de su plataforma | PKCE loopback queda como ruta universal (CLI hoy; escritorios sin GOA en v1.0). M1 se demuestra vía GOA en GNOME. Riesgo a verificar en M1: que los tokens GOA traigan scopes utilizables para Graph (`Files.ReadWrite.All`) — exactamente lo que el gate de cuenta real detectará |
| **D3** | **Telemetría redefinida — solo auto-observación interna.** El componente original abarcaba (a) que el sistema determine su propio estado para emitir avisos o reaccionar, y (b) informe anonimizado al exterior. **(b) se descarta**: contradice la garantía al adoptante de que ningún dato sale de su PC | `lnxdrive-telemetry` se re-especifica como agente de salud/auto-observación local. El diseño OTLP→Google Cloud (Cloud Run/BigQuery) de `13-telemetria.md` **se elimina de la guía**. Métricas Prometheus siguen siendo locales-only como ya prescribía la guía |
| **D4** | **`v0.1.0-alpha.1` se redefine como "M1–M6 demostrables".** El tag se aleja unas semanas; no se corta un alpha que no puede hacer login | Política de tags en `07-milestones-capacidad.md`. FU-010/FU-011 siguen siendo tag-time y se resuelven al cortar |

## 6. Issues fantasma — insumo del milestone M0

El risk-analysis se escribió **antes** de implementar; al menos dos issues abiertos
parecen ya resueltos en el código actual:

- **#9** "State machine has no exit from error state" — pero `sync_item.rs:768`
  permite `Error → cualquier estado` (retry).
- **#14** "Delta token expiration not handled gracefully" — pero `engine.rs:442`
  maneja `410 Gone → full resync`.
- **#21** "YAML injection" — posible solapamiento parcial con ISSUE-002
  (billion-laughs) ya cerrado en Charter-01 Fase 1; verificar el alcance restante.

Regla del proyecto (aprendida en Charter-02 abandonado): *verificar la premisa
antes de asignar alcance*. De ahí que M0 sea un triage de re-verificación de los
23 issues contra el código actual, antes de agendar nada.

---

*new-guide · documento de trabajo — no canónico. Se traslada a `lnxdrive-guide/`
según `08-plan-actualizacion-guia.md`.*
