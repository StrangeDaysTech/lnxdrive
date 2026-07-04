# Estado de implementación del proveedor OneDrive / Microsoft Graph

Todo el proveedor de nube vive en el crate `lnxdrive-engine/crates/lnxdrive-graph/`. El puerto genérico está en `lnxdrive-core`.

**Conclusión adelantada:** OneDrive está implementado de verdad y casi completo a nivel de operaciones de archivos y OAuth PKCE. Lo que falta / está flojo son: refresco de token en runtime, detección de conflictos bidireccional, y no existe ningún proveedor "mock" de producción (el mock es solo de tests).

---

## 1. El puerto genérico `ICloudProvider`

**Archivo:** `lnxdrive-engine/crates/lnxdrive-core/src/ports/cloud_provider.rs`

- Trait `ICloudProvider` definido en `cloud_provider.rs:163-261` (`#[async_trait]`, `Send + Sync`). Es explícitamente provider-agnostic (doc `cloud_provider.rs:1-14`).
- Tipos DTO del puerto: `AuthFlow` (`:30-44`, solo variante `AuthorizationCodePKCE { app_id, redirect_uri, scopes }`), `Tokens` (`:54-75`, con `access_token`/`refresh_token`/`expires_at` y helpers `is_expired`/`expires_within`), `DeltaResponse`/`DeltaItem` (`:86-121`), `UserInfo` (`:131-143`).

Operaciones declaradas por el trait:

| Método | Línea |
|---|---|
| `authenticate(&AuthFlow) -> Tokens` | `cloud_provider.rs:172` |
| `refresh_tokens(&str) -> Tokens` | `:181` |
| `get_delta(Option<&DeltaToken>) -> DeltaResponse` | `:193` |
| `download_file(&RemoteId) -> Vec<u8>` | `:202` |
| `upload_file(parent, name, data) -> DeltaItem` (simple <4MB) | `:213` |
| `upload_file_session(parent, name, data, progress) -> DeltaItem` (resumable) | `:233` |
| `get_metadata(&RemoteId) -> DeltaItem` | `:248` |
| `get_user_info() -> UserInfo` | `:254` |
| `delete_item(&RemoteId) -> ()` | `:260` |

---

## 2. Implementación OneDrive/Graph — estado por operación

**Archivo principal:** `lnxdrive-engine/crates/lnxdrive-graph/src/provider.rs` — struct `GraphCloudProvider` (`:144-156`) que envuelve un `GraphClient` en `tokio::sync::Mutex`. Impl del trait en `:158-284`.

| Operación | Estado | Evidencia |
|---|---|---|
| `authenticate` | **(b) Stub intencional** — `anyhow::bail!("Use GraphAuthAdapter for authentication")` | `provider.rs:165-167` |
| `refresh_tokens` | **(b) Stub intencional** — `bail!("Use GraphAuthAdapter for token refresh")` | `provider.rs:172-174` |
| `get_delta` | **(a) Real** — delega a `delta::get_delta`, con paginación | `provider.rs:179-183` → `delta.rs:292-363` |
| `download_file` | **(a) Real** — `GET /me/drive/items/{id}/content` | `provider.rs:188-192` → `client.rs:272-291` |
| `upload_file` (simple) | **(a) Real** — `PUT .../content` | `provider.rs:197-211` → `upload.rs:208-237` |
| `upload_file_session` (grande) | **(a) Real** — sesión reanudable en chunks de 10 MiB | `provider.rs:216-231` → `upload.rs:385-447` |
| `get_metadata` | **(a) Real** — `GET /me/drive/items/{id}` + conversión a `DeltaItem` | `provider.rs:236-253` |
| `get_user_info` | **(a) Real** — `GET /me` + `GET /me/drive` (quota) | `provider.rs:258-262` → `client.rs:193-259` |
| `delete_item` | **(a) Real** — `DELETE /me/drive/items/{id}` (soft-delete a papelera) | `provider.rs:268-283` |

**Extra implementado (Files-on-Demand, fuera del trait)** en `provider.rs:290-453`: `get_download_url` (lee `@microsoft.graph.downloadUrl`), `download_file_to_disk` (streaming), `download_range` (HTTP Range para hidratación parcial). Usados por la capa FUSE (`crates/lnxdrive-fuse/src/hydration.rs:50,270`).

**No hay ningún `todo!()`, `unimplemented!()` ni `unreachable!()` en todo el crate `lnxdrive-graph`** (grep confirmado). Los únicos "not implemented" del engine están en FUSE (`filesystem.rs:435`, resume de descargas parciales), no en el proveedor.

Módulos de soporte, todos reales y con tests unitarios extensos:
- `client.rs` — `GraphClient` (base URL `https://graph.microsoft.com/v1.0` en `:30`), `execute_with_retry` con manejo de 429/`Retry-After` (`:315-391`).
- `delta.rs` — parser completo de DriveItem, normalización de paths `/drive/root:`, extracción de delta token, seguimiento de `@odata.nextLink`, y manejo de **410 Gone** (token expirado → resync) en `:311-313`.
- `upload.rs` — `upload_small` / `create_upload_session` / `upload_chunk` / `upload_large` (chunk 10 MiB, `CHUNK_SIZE` en `:27`).
- `rate_limit.rs` — `AdaptiveRateLimiter` (token bucket) real.
- `GraphError` enum en `lib.rs:29-68`.

---

## 3. Autenticación OAuth2 / PKCE

**Archivo:** `lnxdrive-engine/crates/lnxdrive-graph/src/auth.rs` — usa el crate `oauth2`.

- **Sí usa Authorization Code + PKCE (SHA256)**: `PkceCodeChallenge::new_random_sha256()` en `auth.rs:193`, `set_pkce_challenge` en `:201`.
- **Endpoints (tenant "consumers", hardcodeados)**: `AUTH_URL` = `https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize` (`:26`), `TOKEN_URL` = `.../consumers/.../token` (`:29`). El tenant NO es configurable (fijo a `consumers`).
- **redirect_uri**: `http://127.0.0.1:8400/callback` (`:32`). Nota: `authenticate.rs:20` define otra constante `http://localhost:8400` (inconsistencia menor entre las dos rutas de login).
- **scopes por defecto**: `["Files.ReadWrite.All", "User.Read", "offline_access"]` (`:38`) — incluye `offline_access` ⇒ pide refresh token.
- **Construcción de la URL de autorización**: `PKCEFlow::generate_auth_url` (`:192-205`) devuelve `(url, CsrfToken, PkceCodeVerifier)`.
- **state / CSRF**: se genera con `CsrfToken::new_random` (`:195`) y **se valida** en el callback (`:498-504`, `bail!` si no coincide).
- **Intercambio code→tokens**: `PKCEFlow::exchange_code` (`:215-244`), envía el `pkce_verifier`.
- **Refresh token**: `PKCEFlow::refresh_token` (`:253-280`) vía `exchange_refresh_token`. También existe `refresh_via_goa` (`:534-567`) que delega el refresh a GNOME Online Accounts por D-Bus.
- **Servidor de callback local**: `LocalCallbackServer::start` (`:308-388`), servidor `hyper` de una sola conexión en `127.0.0.1:8400`, parsea `code`/`state` (`:391-408`), y sirve HTML de éxito/error.
- **Orquestador**: `GraphAuthAdapter::login` (`:482-511`): genera URL → abre navegador (`webbrowser::open`, `:492`) → arranca callback server → valida state → intercambia code.
- **Almacenamiento de tokens**: `KeyringTokenStorage` (`:88-156`), guarda JSON en el keyring del SO (servicio `"lnxdrive"`, username = email).

### client_id: ¿real o placeholder?
- **Hay un client_id REAL por defecto (GUID):** `DEFAULT_APP_ID = "d50ca740-c83f-4d1b-b616-12c519384f0c"` en `lnxdrive-core/src/usecases/authenticate.rs:26`. Se usa como fallback en `AuthenticateUseCase::login` (`:80`).
- **En config es opcional y arranca vacío/`null`**: `AuthConfig.app_id: Option<String>` (`config.rs:80-84`, comentario "`None` until the user runs `lnxdrive auth login`"; YAML default `null` en `config.rs:1164`).
- **Dos rutas de login divergentes (inconsistencia a resolver):**
  1. **CLI directo** (`crates/lnxdrive-cli/src/commands/auth.rs:69-78`): exige app_id de `--app-id` o `config.auth.app_id`; **NO usa el default** — hace `.context("No app_id provided...")` y construye `GraphAuthAdapter::with_app_id`. No pasa por `AuthenticateUseCase`.
  2. **Use case** (`AuthenticateUseCase::login`): sí usa `DEFAULT_APP_ID` si no se pasa. Pero este use case **llama a `cloud_provider.authenticate()`, que en `GraphCloudProvider` es el stub que hace `bail!`** ⇒ ese camino no funciona end-to-end con el provider real (la autenticación real solo ocurre por el `GraphAuthAdapter` del CLI o por GOA).

---

## 4. Sistema MOCK vs real

- **NO existe un proveedor mock de producción** (ni feature de cargo `mock`/`fake`, ni env var, ni selección en config). No hay un `MockCloudProvider` que implemente `ICloudProvider`. El grep de `mock`/`fake` en código no-test solo devuelve comentarios.
- **El único "mock" es de tests:**
  - `MockAuthBackend` está **dentro del módulo de tests** de `crates/lnxdrive-ipc/src/service.rs:1278-1312` (helpers `::ok(email)` / `::err(...)`). Implementa el trait `AuthBackend`, no `ICloudProvider`. Se inyecta en tests (`service.rs:2011,2034,2064`) para no tocar GOA ni el keyring.
  - Los tests de integración de Graph usan **`wiremock`** (servidor HTTP falso), no un provider mock: `crates/lnxdrive-graph/Cargo.toml` (dev-dep `wiremock`), y `crates/lnxdrive-graph/tests/integration/common.rs:19 setup_graph_mock()` que monta endpoints `/me`, `/me/drive`, etc., y devuelve un `GraphClient` apuntando al mock (`GraphClient::with_base_url`, `client.rs:121`).
- **Selección real de backend de auth en producción:** el daemon inyecta `GoaAuthBackend` (GNOME Online Accounts vía D-Bus) — `crates/lnxdrive-daemon/src/goa_auth_backend.rs`. El trait `AuthBackend` está en `crates/lnxdrive-ipc/src/auth_backend.rs:73-80`. Es decir, hay **dos mecanismos de auth reales** (PKCE-navegador vía CLI, y GOA vía daemon/D-Bus), pero **ningún login automático "mock" para pruebas de VM** en el código de producción actual.

> Sobre "login automático en pruebas de VM": no se encontró nada de eso en producción. Lo más cercano es el `MockAuthBackend` de tests unitarios y los helpers wiremock. El atajo histórico de auto-login probablemente fue retirado (nota: `RISK-002` en `goa_auth_backend.rs:5` documenta el endurecimiento de la superficie D-Bus para no aceptar tokens crudos). El "auto-login" que se veía en VM venía del mock D-Bus en Python (ver informe de diagnóstico de login).

---

## 5. Sincronización con Graph

**Motor:** `lnxdrive-engine/crates/lnxdrive-sync/src/engine.rs` — `SyncEngine` (`:216-257`) recibe `Arc<dyn ICloudProvider>`.

- **Delta sync: IMPLEMENTADO (real).**
  - Use case `QueryDeltaUseCase::execute` (`lnxdrive-core/src/usecases/query_delta.rs:66-106`): lee delta token de la cuenta, llama `get_delta`, persiste el nuevo token (`delta_link`), y `handle_delta_item` (`:127-217`) mapea `DeltaItem`→`SyncItem` (creación/modificación/borrado).
  - Motor: `engine.rs:433` (`get_delta(token)`) con fallback a full resync ante token expirado — `engine.rs:442` "Delta token expired, performing full resync", `:451` re-llama `get_delta(None)`.
  - Persistencia de delta token en la cuenta: `query_delta.rs:81-90`.
- **Descarga: IMPLEMENTADA (real).** `engine.rs:759` y `:873` llaman `download_file`. Además la hidratación FUSE usa `download_range`/`download_file_to_disk`.
- **Subida: IMPLEMENTADA (real).** `handle_local_create` (`engine.rs:~1176-1208`) y `handle_local_update` (`:1293-1313`) eligen entre `upload_file` (simple) y `upload_file_session` (grande) según `self.large_file_threshold` (`engine.rs:1176`, `:1293`). El `progress` se pasa como `None` (subida grande sin callback de progreso conectado).
- **Borrado: IMPLEMENTADO.** `handle_local_delete` (`engine.rs:1351-1377`) llama `delete_item`.
- **Retries:** helper `with_retry(...)` envuelve todas las llamadas al provider (`engine.rs:1182,1200,1294,1306,1364`).

### Lo que FALTA / está flojo en sync
- **Detección/resolución de conflictos: NO IMPLEMENTADA como servicio.**
  - El crate `lnxdrive-conflict` es un **stub vacío**: `crates/lnxdrive-conflict/src/lib.rs` tiene **8 líneas, solo doc-comments**, cero código. No es dependencia de ningún otro crate.
  - En `SyncEngine` no hay lógica de conflicto bidireccional. Solo existe estado de dominio: `SyncItem::resolve_conflict()` (`lnxdrive-core/src/domain/sync_item.rs:927`) y transiciones de estado, pero **nada detecta "cambió local Y remoto a la vez"** contra Graph. `handle_local_update` compara hash local vs hash almacenado (`engine.rs:1268-1271`), no contra el estado remoto actual ⇒ riesgo de last-writer-wins sin detección de conflicto.
- **Refresco de token en runtime: NO CABLEADO.**
  - `AuthenticateUseCase::refresh_if_needed` (`authenticate.rs:201-252`) existe pero **no se llama desde ningún sitio** (grep: única aparición es su definición).
  - El daemon crea el `GraphClient` **una sola vez al arrancar** con el `access_token` del keyring (`crates/lnxdrive-daemon/src/main.rs:232-233`) y entra al `sync_loop`. No hay refresco periódico ni recuperación ante 401 dentro del loop. ⇒ **Cuando el access token expire (~1h), el daemon fallará hasta reiniciar.** `GraphClient::set_access_token` (`client.rs:161`) existe pero no se usa en runtime.

---

## Resumen: hecho / falta

**HECHO (real, con tests):**
- Puerto genérico `ICloudProvider` completo y agnóstico.
- OAuth2 **Authorization Code + PKCE (SHA256)** con validación de state, callback server local, apertura de navegador, y almacenamiento en keyring (`auth.rs`).
- Ruta alternativa de auth vía **GNOME Online Accounts (D-Bus)** en el daemon (`goa_auth_backend.rs`), endurecida para no exponer tokens.
- **client_id real por defecto** (`d50ca740-...`) en `authenticate.rs:26`.
- Todas las operaciones de archivo del trait: delta (con paginación y 410-Gone→resync), download (+ Files-on-Demand con Range), upload simple y resumable en chunks, metadata, user_info/quota, delete.
- Rate limiting adaptativo + retry 429.
- Motor de sync cableado end-to-end (delta→SyncItem, create/update/delete) e integrado en daemon, CLI y FUSE.

**FALTA / INCOMPLETO:**
- **Refresco de token en runtime**: `refresh_if_needed` definido pero nunca invocado; el daemon usa un token fijo cargado al inicio (`main.rs:232`). No hay recuperación ante 401/expiración. (Gap más crítico para uso prolongado.)
- **Detección/resolución de conflictos**: crate `lnxdrive-conflict` **vacío (8 líneas)**, sin usar por nadie; el engine no detecta cambios simultáneos local+remoto. Solo hay transiciones de estado de dominio.
- **Tenant fijo a `consumers`** (no soporta cuentas de organización/`common`/tenant específico); redirect_uri inconsistente entre las dos rutas (`127.0.0.1:8400/callback` vs `localhost:8400`).
- **Dos rutas de login divergentes**: el CLI no usa `DEFAULT_APP_ID` (exige config), y `AuthenticateUseCase::login` llama a `authenticate()` que en el provider real es un `bail!` stub ⇒ ese use case no autentica de verdad con Graph.
- **`upload_file_session` sin progreso**: el engine pasa `progress = None`.
- **No existe proveedor mock/fake de producción** ni selección mock-vs-real por feature/env/config; los mocks son solo de tests (`wiremock` para Graph, `MockAuthBackend` para D-Bus).
- No hay un segundo proveedor todavía (el puerto está listo para ello, pero OneDrive/Graph es el único).
