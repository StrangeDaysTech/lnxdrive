# Diagnóstico: error de login en la app de preferences

## Resumen ejecutivo (causa raíz)

El botón "Sign In" de la app de preferences NO construye la URL OAuth localmente: llama por D-Bus al método `StartAuth()` del daemon, y **el handler `StartAuth` del daemon real devuelve una URL de autorización literalmente vacía de parámetros**:

`lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs:853-855`
```rust
let auth_url = state.auth_url.clone().unwrap_or_else(|| {
    "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string()
});
```

Esa URL **no lleva `client_id`, ni `response_type`, ni `redirect_uri`, ni `scope`, ni `state`, ni `code_challenge`**. Microsoft rechaza la petición por el primer parámetro obligatorio ausente: **`client_id`** (error tipo AADSTS900144 "The request body must contain the following parameter: 'client_id'"). Ese es el "campo que falta".

El código que SÍ arma bien la URL (crate `oauth2`, con client_id/response_type/redirect_uri/scope/state/PKCE) existe pero **el daemon nunca lo invoca** — solo lo usa el CLI. `state.auth_url` siempre es `None` en el daemon (nada lo puebla), así que se usa el fallback placeholder.

---

## 1. Pantalla/página de login y manejo del clic

- **`lnxdrive-gnome/preferences/src/onboarding/auth_page.rs`** — página `AdwNavigationPage` "Sign in to OneDrive".
  - Botón "Sign In" creado en `auth_page.rs:101-107`; señal conectada en `auth_page.rs:205-207`.
  - Handler del clic: `on_sign_in_clicked()` en `auth_page.rs:218-335`. Su lógica clave:
    - `auth_page.rs:240` → `dbus_client.start_auth().await` (obtiene `(auth_url, state)` del daemon).
    - `auth_page.rs:243` → `gtk4::UriLauncher::new(&auth_url)` y `auth_page.rs:246` `launcher.launch_future(...)` abre el navegador.
    - `auth_page.rs:262-265` → se suscribe a la señal `AuthStateChanged` esperando `"authenticated"`.
  - El botón GOA SSO (feature `goa`) y su handler `on_goa_sign_in_clicked` están en `auth_page.rs:143-167` y `339-407` (ruta alternativa, no usa navegador).
- Proxy D-Bus: `lnxdrive-gnome/preferences/src/dbus_client.rs` (interface `LnxdriveAuthProxy`, método `start_auth`).

## 2. Construcción de la URL OAuth de Microsoft

Hay DOS constructores, y el que dispara la GUI es el equivocado:

- **(BUENO, pero NO usado por la GUI)** `lnxdrive-engine/crates/lnxdrive-graph/src/auth.rs`:
  - `PKCEFlow::generate_auth_url()` en `auth.rs:192-205` usa el crate `oauth2` (`authorize_url` + `add_scope` + `set_pkce_challenge`), que emite todos los params: `client_id`, `response_type=code`, `redirect_uri`, `scope`, `state`, `code_challenge`, `code_challenge_method=S256`.
  - Constantes: `AUTH_URL` = `https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize` (`auth.rs:26`), `REDIRECT_URI` = `http://127.0.0.1:8400/callback` (`auth.rs:32`), `DEFAULT_SCOPES` = `Files.ReadWrite.All`, `User.Read`, `offline_access` (`auth.rs:38`).
  - Este flujo solo lo llama el CLI: `lnxdrive-engine/crates/lnxdrive-cli/src/commands/auth.rs:78-79` (`GraphAuthAdapter::with_app_id(...).login()`).

- **(EL QUE USA LA GUI, roto)** `lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs:849-863` (`AuthInterface::start_auth`). Devuelve la URL fija sin query params. Params incluidos: **NINGUNO** (solo el endpoint base, tenant `common`). CSRF `state` fallback = `"pending"` (`service.rs:856-857`).

## 3. Campo que falta / vacío

Contra los requeridos por Microsoft identity v2.0 (authorization code + PKCE), en la URL que abre la GUI faltan **TODOS**:
- `client_id` → **AUSENTE** (primer error que reporta Microsoft).
- `response_type=code` → AUSENTE.
- `redirect_uri` → AUSENTE.
- `scope` → AUSENTE.
- `state` → AUSENTE (aunque el valor CSRF interno es el placeholder `"pending"`).
- `code_challenge` / `code_challenge_method` → AUSENTES (PKCE nunca se genera en esta ruta).

Sobre el `client_id` en sí (por si además estuviera mal configurado en la ruta buena):
- Default hardcodeado: `DEFAULT_APP_ID = "d50ca740-c83f-4d1b-b616-12c519384f0c"` en `lnxdrive-engine/crates/lnxdrive-core/src/usecases/authenticate.rs:26`.
- En config es opcional y **por defecto `None`**: `AuthConfig.app_id: Option<String>` en `lnxdrive-engine/crates/lnxdrive-core/src/config.rs:82` (comentario: "`None` until the user runs `lnxdrive auth login`"). YAML default `app_id: null` (`config.rs:1164`).
- El CLI exige app_id o falla: `commands/auth.rs:69-72` ("No app_id provided. Use --app-id flag or set auth.app_id in config.yaml").

Es decir: incluso si el daemon llamara al constructor correcto, dependería de que `auth.app_id` esté configurado; hoy además ni siquiera lo llama.

## 4. Cómo se dispara el navegador y el callback

- En la GUI: `gtk4::UriLauncher` (`Gio` bajo el capó, no `xdg-open` directo) — `auth_page.rs:243-246`.
- En el CLI/flujo real: `webbrowser::open(&auth_url)` — `auth.rs:492`.
- Servidor de callback loopback: `LocalCallbackServer::start()` en `auth.rs:303-388`, bind a `127.0.0.1:8400`, ruta `/callback`, parsea `code`/`state` (`parse_callback_params` `auth.rs:391-408`) e intercambia el code (`exchange_code` `auth.rs:215-244`). **Está implementado**, pero pertenece al `GraphAuthAdapter`/CLI, NO al daemon que atiende la GUI.
- Discrepancia adicional de redirect_uri: `auth.rs:32` usa `http://127.0.0.1:8400/callback`, mientras `usecases/authenticate.rs:20` define `DEFAULT_REDIRECT_URI = "http://localhost:8400"` (sin `/callback`). Inconsistente, relevante si se llega a arreglar la ruta.
- El daemon real completa auth por otra vía (GOA, sin code de navegador): `complete_auth`/`complete_auth_via_goa` en `service.rs:873-891` y `893+`, con `GoaAuthBackend` cableado en `lnxdrive-engine/crates/lnxdrive-daemon/src/main.rs:131-132` (`DbusService::new(...).with_auth_backend(GoaAuthBackend::new())`). El daemon **no** cablea nada que genere/pueble `state.auth_url`.

## 5. El mock anterior (por qué "funcionaba" en la VM)

- **`lnxdrive-gnome/tests/mock-dbus-daemon.py`** — daemon D-Bus mock en Python lanzado manualmente para tests de integración GNOME (arranca con `python3 mock-dbus-daemon.py [--authenticated] ...`, ver cabecera `:15` y `test-nautilus-extension.py:44` `MOCK_DAEMON`).
  - `StartAuth` mock: `mock-dbus-daemon.py:570-574` devuelve `https://login.microsoftonline.com/mock-auth?state=mock123` (URL falsa que no valida params reales).
  - `CompleteAuth` mock: `:576-582` marca autenticado y emite `AuthStateChanged("authenticated")` de inmediato → por eso el login parecía "automático" en la VM: la señal llegaba sin login real de Microsoft.
- **Activación**: no hay feature flag ni env var tipo `LNXDRIVE_MOCK`. El mock se "activa" simplemente registrando ese proceso Python en el bus D-Bus de sesión con el nombre `com.strangedaystech.LNXDrive` en lugar del daemon real; la app de preferences se conecta a quien tenga ese nombre en el bus.
- **¿Sigue existiendo?** Sí, el script sigue presente en `lnxdrive-gnome/tests/`. Pero al arrancar el daemon real en vez del mock, `StartAuth` cae en `service.rs:853` (URL vacía) → error de Microsoft.

---

## Conclusión / punto exacto a corregir

El parámetro que Microsoft reporta como faltante es **`client_id`** (y en realidad faltan todos los query params). Causa: `AuthInterface::start_auth` en **`lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs:849-863`** devuelve un placeholder `https://login.microsoftonline.com/common/oauth2/v2.0/authorize` sin parámetros, porque el daemon nunca invoca el generador real `PKCEFlow::generate_auth_url()` (`lnxdrive-engine/crates/lnxdrive-graph/src/auth.rs:192`) ni puebla `state.auth_url`. El flujo PKCE correcto solo está cableado en el CLI (`lnxdrive-cli/src/commands/auth.rs:78`), no en el daemon que sirve a la GUI. Antes "funcionaba" porque en VM corría el mock (`mock-dbus-daemon.py:570`) que devolvía una URL falsa y auto-emitía `authenticated`.

### Qué se necesita para arreglarlo
Cablear en el daemon la ruta PKCE real: que `StartAuth` genere la URL con `PKCEFlow::generate_auth_url()` (poblando `state.auth_url`, el verifier PKCE y el CSRF `state`), levante el callback server loopback en `127.0.0.1:8400`, y complete el intercambio `code → tokens` para almacenarlos en el keyring. De paso, resolver la inconsistencia de `redirect_uri` (`127.0.0.1:8400/callback` vs `localhost:8400`) y decidir el origen del `app_id` (usar `DEFAULT_APP_ID` como fallback también en el daemon).
