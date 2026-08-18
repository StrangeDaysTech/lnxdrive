---
id: AILOG-2026-08-17-002
title: "CHARTER-02 batch M1: login end-to-end — GOA primario + browser/PKCE loopback en el daemon"
status: draft
created: 2026-08-17
agent: qwen3.8-max
confidence: high
review_required: true
risk_level: high
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security, data_privacy]
iso_42001_clause: [6, 8, 9]
lines_changed: 1166
files_modified:
  - lnxdrive-engine/crates/lnxdrive-ipc/src/auth_backend.rs
  - lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs
  - lnxdrive-engine/crates/lnxdrive-daemon/src/goa_auth_backend.rs
  - lnxdrive-engine/crates/lnxdrive-daemon/src/main.rs
  - lnxdrive-engine/crates/lnxdrive-graph/src/auth.rs
  - lnxdrive-engine/crates/lnxdrive-graph/src/provider.rs
  - lnxdrive-engine/crates/lnxdrive-core/src/usecases/authenticate.rs
  - lnxdrive-engine/crates/lnxdrive-cli/src/commands/auth.rs
  - lnxdrive-gnome/preferences/src/dbus_client.rs
  - lnxdrive-gnome/preferences/src/onboarding/auth_page.rs
  - lnxdrive-gnome/meson_options.txt
observability_scope: none
tags: [charter-02, m1, auth, oauth, pkce, goa, keyring, dbus, security]
related:
  - CHARTER-02-road-to-functional-v0-1
  - AILOG-2026-07-04-004
  - AILOG-2026-05-29-002
  - ETH-2026-08-17-001
  - ETH-2026-05-29-001
  - AIDEC-2026-07-04-001
---

# AILOG: CHARTER-02 batch M1 — "Puedo entrar"

## Summary

Segundo batch de CHARTER-02 (issue #70 = FU-017). Se implementa el login
end-to-end desde la app de preferencias con las dos rutas decididas en
AIDEC-2026-07-04-001: **GOA** (primaria en GNOME, decisión D2) y
**browser/PKCE con loopback dentro del daemon** (fallback universal). Los
tokens SOLO tocan el keyring; ni los tokens ni el authorization code cruzan
D-Bus (extensión del invariante RISK-002). Se consolida FU-017
(`redirect_uri`, stub `AuthenticateUseCase::login`, origen del `app_id`) y
se añade el botón de onboarding que lanza GNOME Online Accounts. **Cambio de
código de autenticación ⇒ `risk_level: high`, requiere ETH y revisión humana.**

## Context

M1 existe porque el sistema "no funciona end-to-end": el botón Sign In de la
app de preferencias llamaba a `Auth.StartAuth`, que devolvía una URL OAuth
**sin ningún parámetro** (`service.rs:853`, `03-diagnostico-login.md` §3);
Microsoft la rechazaba por falta de `client_id`. El flujo PKCE correcto ya
existía en `lnxdrive-graph` pero solo estaba cableado al CLI. Antes
"funcionaba" en la VM porque corría el mock (`mock-dbus-daemon.py:570`) que
auto-emitía `authenticated`. Además, las rutas de auth divergían (FU-017):
`redirect_uri` inconsistente, `AuthenticateUseCase::login` llamando a un
`authenticate()` que era `bail!` en el único provider, y `app_id` sin origen
fijo.

## Actions Performed

1. **Ruta browser/PKCE en el daemon** (issue #70, loopback-en-daemon):
   - `AuthBackend` gana `start_browser_auth()` → `BrowserAuthStart` y
     `complete_browser_auth(csrf, verifier)` → `AuthenticatedAccount`
     (`auth_backend.rs`).
   - `AuthInterface::start_auth` arma el flujo real vía el backend, guarda
     `csrf_state`/`pkce_verifier` en `DaemonState` y **lanza una tarea** que
     espera el redirect loopback, actualiza el estado y emite
     `AuthStateChanged` (`service.rs`).
   - `GoaAuthBackend::complete_browser_auth` posee la captura loopback
     (`LocalCallbackServer`), valida CSRF, intercambia el code, resuelve el
     perfil Graph y persiste tokens + cuenta. **El code no cruza D-Bus.**
   - **Retirado** el método `Auth.CompleteAuth(code, state)` del surface
     D-Bus (y del proxy de preferences) — era el vector que llevaba el code
     por el bus.
2. **Ruta GOA**: `complete_auth_via_goa` ahora enriquece best-effort vía
   Graph (prueba empírica del riesgo R1), persiste la cuenta y emite
   `AuthStateChanged("authenticated")` — antes no lo hacía (issue #70).
3. **Emisión de señales desde tareas en segundo plano**: `ConnectionSlot`
   publicado por `DbusService::start()` (y re-publicado en cada reconnect del
   health-monitor, RISK-001) + helper `emit_auth_state_changed` que resuelve
   el `InterfaceRef` y emite. Sobrevive a caídas del bus.
4. **FU-017 — consolidación**:
   - `redirect_uri` unificado a `http://127.0.0.1:8400/callback`
     (`lnxdrive-graph::auth::REDIRECT_URI` = única fuente de verdad; el
     `DEFAULT_REDIRECT_URI` de core alineado).
   - `GraphCloudProvider::authenticate`/`refresh_tokens` ahora componen
     `GraphAuthAdapter` detrás del puerto — dejan de ser stubs `bail!`, y
     `AuthenticateUseCase::login` funciona end-to-end.
   - Origen del `app_id`: **flag > config > default** (`DEFAULT_APP_ID`
     hecho `pub`). El CLI ya no falla duro sin config explícita.
5. **Persistencia de cuenta en el login** (ambas rutas): SQLite + audit
   entry, replicando los pasos 5-6 del `auth login` del CLI. Es lo que
   permite al `wait_for_auth_loop` del daemon retomar sin reinicio.
6. **Onboarding GNOME** (issue #70): botón "Add a Microsoft account in GNOME
   Online Accounts…" visible cuando no hay cuenta `lnxdrive_microsoft`;
   abre `gnome-control-center online-accounts` y un polling (~5 min) lo
   sustituye por el botón GOA en cuanto la cuenta existe.
7. **Meson**: `enable_goa` por defecto `false → true` (instalación
   host-side del proveedor). El manifiesto Flatpak mantiene
   `-Denable_goa=false` a propósito (el proveedor debe vivir en el host, no
   dentro del sandbox).
8. **Tests**: `service.rs` (armado real del flujo, idempotencia, completion
   con éxito/fallo, estado resultante) y `goa_auth_backend.rs` (rechazo de
   path no-GOA, URL PKCE real con `client_id`/`code_challenge`).

## Risk

- **`risk_level: high`** — código de autenticación/credenciales. Requiere
  **ETH** (`ETH-2026-08-17-001`, draft) y aprobación humana antes del merge,
  según `AGENT-RULES.md`.
- **R1 (riesgo del Charter, materializable)**: el token GOA podría no traer
  scopes utilizables para Graph. El backend lo prueba empíricamente con
  `GET /me`+`/me/drive` en el login GOA; si falla, degrada a una cuenta
  mínima y la capacidad puede cerrarse por la ruta PKCE (fallback ya
  implementado). Confirmación final = guion del operador contra OneDrive real.
- **R2 (nuevo, acotado)**: la tarea loopback queda armada hasta 300 s si el
  usuario abandona el flujo; el listener se libera por timeout. Idempotencia
  de `StartAuth` evita dobles bind.
- **R3 (entorno)**: este checkout compila y pasa tests, pero la verificación
  de capacidad es contra OneDrive real (guion `new-guide/09` §M1). No se
  cierra M1 contra mock (regla 1 del replanteo).

## Modified Files

| File | Change |
|------|--------|
| `lnxdrive-ipc/src/auth_backend.rs` | `AuthenticatedAccount`, `BrowserAuthStart`, variantes de error, 2 métodos nuevos del trait |
| `lnxdrive-ipc/src/service.rs` | `start_auth` real + tarea de completion, retiro de `complete_auth`, emisión de señal, `ConnectionSlot`, `pkce_verifier` en estado |
| `lnxdrive-daemon/src/goa_auth_backend.rs` | Rutas GOA+PKCE completas: loopback, CSRF, exchange, Graph, keyring, cuenta |
| `lnxdrive-daemon/src/main.rs` | Cablea `GoaAuthBackend` con `app_id`/repo/sync_root |
| `lnxdrive-graph/src/auth.rs` | `REDIRECT_URI` `pub`, wrappers `arm_pkce_flow`/`exchange_pkce_code` |
| `lnxdrive-graph/src/provider.rs` | `authenticate`/`refresh_tokens` reales (fin del stub) |
| `lnxdrive-core/src/usecases/authenticate.rs` | `DEFAULT_APP_ID` `pub`, `redirect_uri` consolidado |
| `lnxdrive-cli/src/commands/auth.rs` | `app_id` flag>config>default |
| `lnxdrive-gnome/preferences/src/dbus_client.rs` | Retiro de `complete_auth` del proxy |
| `lnxdrive-gnome/preferences/src/onboarding/auth_page.rs` | Botón onboarding GOA + polling |
| `lnxdrive-gnome/meson_options.txt` | `enable_goa=true` por defecto |

## Decisions Made

- **Loopback + exchange viven en el backend, no en ipc**: mantiene
  `lnxdrive-ipc` agnóstico de Graph y garantiza que el authorization code no
  cruce D-Bus (espíritu de RISK-002 aplicado al code).
- **`ConnectionSlot` para señales**: la tarea de completion emite vía la
  conexión viva publicada por `DbusService::start()`; transparente a los
  reconnects del health-monitor (RISK-001).
- **Cuenta mínima si Graph falla en la ruta GOA**: el login es real; el
  daemon arranca y expone el fallo de Graph desde sync en vez de quedar
  esperando una auth que ya ocurrió.
- **Flatpak mantiene `enable_goa=false`**: el proveedor GOA debe cargarlo el
  `goa-daemon` del host; dentro del sandbox no es visible. Se documenta como
  requisito host-side.

## Impact

- **Functionality**: el login GUI pasa de "URL vacía rechazada por
  Microsoft" a dos rutas funcionales con tokens solo en keyring. El daemon
  retoma tras el login sin reinicio.
- **Security**: se retira el último método del surface D-Bus que aceptaba un
  secreto (el code en `CompleteAuth`). Tokens y code permanecen en el
  proceso del daemon + keyring.
- **Process**: M1 queda abierto con guion escrito (regla 2 del replanteo);
  el cierre lo dicta el operador ejecutando el guion contra OneDrive real.

## Verification

- [x] `cargo test --workspace` — 0 fallos (24 suites ok), incluye los tests
      nuevos de armad o/completion de flujo y URL PKCE real.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` — limpio.
- [x] `cargo build --features goa` (preferences) — OK; clippy de preferences
      limpio.
- [x] `meson setup` (con `-Denable_goa=false` aquí por ausencia de las libs
      de desarrollo GOA en este entorno) — el build system queda intacto.
- [ ] **Guion del operador contra OneDrive real** (`new-guide/09` §M1):
      gate de capacidad, incluye el gate R1. Pendiente.

## Follow-ups

Ninguno nuevo. El pendiente de este batch es el **gate de capacidad del
operador** (guion `new-guide/09` §M1 contra OneDrive real, incluye el gate
R1), que se trackea por el mecanismo de batch/charter — no como follow-up.
FU-017 se resuelve con este batch (ver `Decisions Made` y el issue #70); su
cierre formal en el registry lo hace el operador al validar el gate.

## Additional Notes

- El ETH correspondiente es `ETH-2026-08-17-001` (draft). El PR de este
  batch no debe mergear sin aprobación humana del ETH (código de
  autenticación).
- **Mock divergente** (hallazgo, no blocking): `mock-dbus-daemon.py` aún
  devuelve URL falsa en `StartAuth` y conserva `CompleteAuth`; aceptable
  para tests de UI, no sirve de gate (regla 1 del replanteo). Considerar
  alinear/retirar en M2+.
- Siguiente batch: **M2 — "Veo mis archivos"** (delta inicial real tras el
  login; `lnxdrive status` refleja cuenta y conteo).

---

<!-- Template: StrayMark | https://strangedays.tech -->
