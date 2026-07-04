---
charter_id: CHARTER-02-road-to-functional-v0-1
status: in-progress
started_at: 2026-07-04
effort_estimate: L
trigger: "Replanteo aprobado el 2026-07-04 (new-guide/06-08, PR #61): CHARTER-01 cerrado por redefinición — su Fase 6 (tag) tenía un gate incorrecto (checklist de features sin criterio de capacidad). Decisión D4 del operador: v0.1.0-alpha.1 se re-gatea como 'M1–M6 demostrables contra OneDrive real'."
originating_ailogs: [AILOG-2026-07-04-001]
work_verb: implement
design_provenance: new
---

# Charter: Road to functional v0.1

> **Status (mirrored from frontmatter — source of truth is above):** in-progress (started 2026-07-04, batch M0). Effort: L (~6–8 semanas calendario).
>
> **Origin:** Follow-up of AILOG-2026-07-04-001 (gobernanza del replanteo) — sucesor directo de CHARTER-01, que cerró con Fases 0–5 ejecutadas y la Fase 6 (tag) sin ejecutar por gate incorrecto. Este Charter ejecuta la escalera de capacidades M0–M6 de `new-guide/07-milestones-capacidad.md` y termina cortando el tag `v0.1.0-alpha.1`.

## Context

CHARTER-01 entregó las piezas (motor Graph, FUSE, stack GNOME, packaging Flatpak,
infraestructura de release) pero el sistema no funciona end-to-end: el login GUI
está roto (`StartAuth` devuelve una URL OAuth sin parámetros), el daemon no
refresca el token en runtime (muere a la ~1h), y no existe detección de
conflictos (riesgo last-writer-wins). El replanteo del 2026-07-04
(`new-guide/06-catalogo-desviaciones.md`) diagnosticó tres desconexiones: hitos
sin criterio de capacidad, testing 100% contra mock, y risk backlog desconectado
de los milestones.

Este Charter corrige las tres: sus fases **son** milestones por capacidad
(cada una demostrable por el operador contra OneDrive real, con guion de
verificación), ninguna capacidad se cierra contra mock, y los issues del risk
backlog se asignan a la fase que bloquean (GitHub Milestones M0–M6). Aplican las
decisiones del operador D1 (funciones delimitadas por crate: la detección de
conflictos nace en `lnxdrive-conflict`), D2 (GOA como vía de login GNOME; PKCE
loopback como ruta universal) y D4 (el tag es consecuencia, no hito).

## Scope

**In scope:**

1. **M0 — "Sé qué es verdad"**: triage de re-verificación de los 23 issues
   abiertos (`from-risk-analysis`) contra el código actual. Sospechosos de estar
   ya resueltos: #9 (`sync_item.rs:768` permite `Error → *`), #14 (`engine.rs:442`
   maneja `410 Gone → full resync`), #21 (solapamiento parcial con ISSUE-002
   cerrado). Cada fantasma se cierra con nota de verificación; el resto se asigna
   al milestone de GitHub que bloquea.
2. **M1 — "Puedo entrar"**: login real con cuenta Microsoft desde la app de
   preferencias vía **GOA** (D2); tokens al keyring; la UI muestra la cuenta.
   Incluye: build shippeado con feature `goa` activa, arreglo del placeholder
   `StartAuth` (PKCE como ruta universal/CLI), consolidación del `redirect_uri`
   (`http://127.0.0.1:8400/callback` vs `localhost:8400`) y del origen del
   `app_id` (default vs config). Verifica de paso que los tokens GOA traen scopes
   utilizables para Graph.
3. **M2 — "Veo mis archivos"**: tras login, el delta inicial puebla placeholders
   del OneDrive real del operador; `lnxdrive status` refleja cuenta y conteo.
4. **M3 — "Abro un archivo"**: hidratación on-demand real desde Nautilus
   (placeholder → descarga → abre; xattr/estado correctos).
5. **M4 — "Mis cambios viajan"**: bidireccional demostrado — edición local
   aparece en OneDrive web; edición remota baja.
6. **M5 — "Sobrevive el tiempo"**: daemon 24h+ con sesión activa — cablear
   `refresh_if_needed` (hoy definido y jamás invocado) + manejo de 401 en el
   sync loop; sin reinicios manuales.
7. **M6 — "No destruye datos"**: detección de conflicto simultáneo local+remoto,
   materializado sin pérdida silenciosa (resolución mínima: keep-both).
   **Implementado en `lnxdrive-conflict`** (D1) consumiendo los tipos de dominio
   de core. Los P0 de data-integrity (#8, #12, #13) llegan aquí verificados o
   resueltos.
8. **Cierre = tag**: con M1–M6 demostrados, fecha real en `CHANGELOG.md` y
   metainfo (FU-010/FU-011, transferidos de CHARTER-01), tag firmado
   `v0.1.0-alpha.1`, pre-release de GitHub con bundle + SHA256SUMS, anuncios.

**Out of scope:**

- Todo el epic "distribuible" de v0.2.0-beta (Flathub/vendoring FU-003/FU-008,
  RPM/DEB/AUR/AppImage, grupo System, Unix-socket fallback, i18n, tarpaulin
  formal) — hereda los diferimientos de CHARTER-01 sin cambios.
- Epic estructural "restaurar delimitación de crates" para lo YA implementado
  (extracción de audit desde core; implementación de telemetry redefinida por
  D3) — v0.2.0-beta. M6 no lo adelanta: solo la funcionalidad NUEVA nace en su
  crate.
- FU-007 (IPC FoD para `pin/hydrate/…` del CLI) — v0.2.0-beta; no bloquea
  ninguna capacidad M1–M6 (la hidratación de M3 va por FUSE, no por CLI).
- UIs de `experimental/`, multi-proveedor, 5+ idiomas — v1.0.0.
- Resolución de conflictos avanzada (reglas YAML, UI de resolución, Meld) — M6
  entrega detección + keep-both; lo demás es post-v0.1.

## Files to modify

Charter multi-batch (7 milestones); la tabla nombra las superficies load-bearing
verificadas por lectura en los informes `new-guide/01-05` (2026-07-03/04). Los
milestones M2–M4 son mayormente de *verificación* sobre código existente: su
superficie de fix exacta se descubre al ejecutar y se documenta por el patrón de
actualización atómica (convención 5).

| File | Change |
|---|---|
| `lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs` | M1: `AuthInterface::start_auth` deja de devolver el placeholder sin parámetros (`:849-863`); ruta GOA verificada end-to-end; PKCE poblando `state.auth_url` como vía universal |
| `lnxdrive-engine/crates/lnxdrive-graph/src/auth.rs` | M1: consolidar `REDIRECT_URI` (`:32`) con `authenticate.rs:20`; M5: reuso de `PKCEFlow::refresh_token` / `refresh_via_goa` desde el daemon |
| `lnxdrive-engine/crates/lnxdrive-core/src/usecases/authenticate.rs` | M1: resolver la divergencia de rutas (el use case llama `cloud_provider.authenticate()` que es `bail!` en el provider real); M5: `refresh_if_needed` (`:201`) por fin invocado |
| `lnxdrive-engine/crates/lnxdrive-daemon/src/main.rs` | M5: el `GraphClient` deja de crearse una sola vez con token fijo (`:232`); refresh/re-auth en el loop |
| `lnxdrive-gnome/preferences/src/onboarding/auth_page.rs` + `lnxdrive-gnome/preferences/Cargo.toml` | M1: camino GOA (`#[cfg(feature = "goa")]`) activo en el build shippeado; UX de error si GOA ausente |
| `lnxdrive-engine/crates/lnxdrive-sync/src/engine.rs` | M4: fixes que surjan de la verificación real; M6: hook de detección de conflictos antes de upload (`:1268` hoy compara solo hash local vs almacenado) |
| `lnxdrive-engine/crates/lnxdrive-conflict/src/lib.rs` | M6: New (deja de ser stub) — detección + materialización keep-both, consumiendo tipos de dominio de core (D1) |
| `lnxdrive-engine/crates/lnxdrive-cache/src/repository.rs` | M6: persistencia de conflictos detectados (métodos ya existentes en `IStateRepository`) |
| `CHANGELOG.md` + `lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in` | Cierre: fecha real del tag (FU-010/FU-011) |
| `new-guide/09-guiones-verificacion.md` | New: guion de verificación por milestone, escrito al ABRIR cada batch (regla 2 del replanteo) |
| `.straymark/07-ai-audit/agent-logs/{daemon,gnome}/AILOG-*.md` | New: un AILOG por milestone/batch, `risk_level` según fase (medium M1/M5/M6; low M0/M2–M4) |

## Verification

### Local checks

```bash
# Build & test — engine y preferences compilan y pasan en checkout limpio
cd lnxdrive-engine && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
cd ../lnxdrive-gnome/preferences && cargo build --features goa

# Gobernanza — Charter válido, sin drift no documentado
straymark validate
straymark charter drift CHARTER-02-road-to-functional-v0-1 --range origin/main..HEAD

# M0 — cada issue cerrado como fantasma referencia file:line del código que lo resuelve
gh issue list --state closed --search "verificado" --limit 30
```

### Capability gates (operador, OneDrive real — no ejecutables sin cuenta)

Guiones completos en `new-guide/09-guiones-verificacion.md` (se escriben al abrir
cada batch). Resumen del gate por milestone:

```bash
# M1: app de preferencias → Sign In (GOA) → cuenta visible; tokens SOLO en keyring
secret-tool search service lnxdrive        # token presente
dbus-monitor --session | grep -cE "Bearer|eyJ"   # 0 apariciones

# M2: tras login, placeholders reales
lnxdrive status --json | jq '.account, .items_total'   # cuenta + conteo > 0

# M3: hidratar desde Nautilus un placeholder y verificar estado
getfattr -n user.lnxdrive.state <archivo>  # hydrated

# M4: tocar archivo local → aparece en OneDrive web; editar remoto → baja
# M5: daemon 24h+ (journalctl --user -u lnxdrive) sin reinicio, token renovado
# M6: editar el MISMO archivo local y remoto en <1 ciclo de sync →
#     ambos contenidos sobreviven (keep-both), estado Conflicted visible
```

## Risks

- **R1 — Los tokens GOA no traen scopes utilizables para Graph** (`Files.ReadWrite.All`).
  Probabilidad media, severidad alta (invalida la vía D2 para M1).
  Mitigación: es lo PRIMERO que M1 verifica contra cuenta real (gate temprano,
  primera semana). Si falla: fallback a PKCE loopback daemon-owned como vía GUI
  (los dos caminos ya existen en código), GOA se re-evalúa como SSO de solo-identidad,
  y la decisión D2 se re-abre con el operador antes de continuar.
- **R2 — M6 (conflictos) explota el presupuesto** — es la única capacidad sin base
  implementada. Probabilidad media, severidad media.
  Mitigación: alcance mínimo inflexible (detección + keep-both; nada de reglas/UI);
  si al cabo de 2 semanas de batch no hay gate M6, se divide: M6a detección-only
  cierra el Charter con keep-both manual documentado, M6b pasa a v0.2. El tag no
  se corta sin al menos M6a.
- **R3 — Los P0 de data-integrity (#8/#12/#13) exigen cirugía de engine mayor a lo
  presupuestado.** Probabilidad media, severidad alta.
  Mitigación: M0 los re-verifica y dimensiona ANTES de agendar (primera semana);
  si alguno requiere >1 semana, se presenta al operador con opciones (bloquear
  tag vs documentar como known-limitation del alpha) — decisión explícita, no
  deriva silenciosa.
- **R4 — Drift declarado-vs-tocado** (constante en Charters largos). Probabilidad
  alta, severidad baja.
  Mitigación: `straymark charter drift` pre-commit por batch (Tasks #8);
  desviaciones al AILOG como `R<N+1> (new, not in Charter)` + actualización
  atómica de `## Files to modify` en el mismo PR.
- **R5 — Throttling/limitaciones de la API real de Microsoft durante verificación
  intensiva.** Probabilidad media, severidad baja.
  Mitigación: cuenta de pruebas dedicada; el `AdaptiveRateLimiter` ya implementado
  gestiona 429; los guiones de verificación espacian operaciones. Si un gate se
  vuelve no-determinista por throttling, se documenta el patrón y se repite en
  ventana distinta — nunca se marca pasado sin observarlo.

## Tasks

Ejecución **multi-batch** (7 batches = M0…M6). Tras el merge del PR de cada batch:
`straymark charter batch-complete CHARTER-02-road-to-functional-v0-1 <N>`.

1. Sync `main`, branch por batch (`fix/m1-…`, `feat/m6-…` según el caso).
2. Al ABRIR cada batch: escribir el guion de verificación del milestone en
   `new-guide/09-guiones-verificacion.md` (ex-ante, regla 2).
3. M0: triage de 23 issues → cerrar fantasmas con evidencia file:line, asignar
   el resto a GitHub Milestones M1–M6 (o v0.2/backlog).
4. M1…M6: implementar/verificar según Scope; el batch NO se cierra hasta que el
   operador ejecuta el gate de capacidad contra OneDrive real y lo confirma.
5. AILOG por batch (`risk_level` según tabla) con `## Batch Ledger` mantenido.
6. Pre-commit por batch: `straymark charter drift CHARTER-02-road-to-functional-v0-1`.
7. Registrar en el registro de follow-ups los emergentes de cada batch
   (`straymark followups drift --apply` antes de commitear el AILOG).
8. Cierre: FU-010/FU-011 (fechas) → tag firmado `v0.1.0-alpha.1` → pre-release
   GitHub → anuncios (r/linux, r/gnome, r/onedrive, Mastodon) →
   `straymark charter close` con telemetría.

## Charter Closure

When closing this Charter:

1. **Atomic update (format v4)**: si el drift check reportó desviaciones no
   capturadas en los AILOGs de batch, editar `## Files to modify` y/o añadir
   `## Closing notes` en el mismo commit/PR del cierre.
2. **Post-merge drift check**: `straymark charter drift CHARTER-02-road-to-functional-v0-1 --range origin/main..HEAD`.
3. **Status frontmatter** → `closed` + `closed_at`; telemetría con el comando
   `straymark charter close` (comparar estimación L vs real; resultados R1–R5;
   conteo de emergentes).
4. **El tag es el gate**: este Charter no cierra sin `v0.1.0-alpha.1` publicado
   — o sin una decisión explícita del operador que re-defina el cierre (como le
   ocurrió a CHARTER-01; si sucede dos veces, el formato de milestones se
   re-examina en la retro).
5. **No borrar** este archivo.

<!--
Format conventions — see the template footer preserved in CHARTER-01 and the
framework template; conventions 1-8 apply unchanged to this Charter.
-->
