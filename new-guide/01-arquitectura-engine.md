# Mapa de arquitectura: backend común de LNXDrive (engine Rust)

## Resumen ejecutivo

El monorepo sigue el patrón **"un daemon común (`lnxdrived`) + UIs delgadas por escritorio que hablan D-Bus"**. El backend es un workspace Cargo en `lnxdrive-engine/` con **11 crates** organizados en arquitectura hexagonal: un núcleo de dominio puro (`lnxdrive-core`) con puertos (traits) y adaptadores concretos en crates satélite. Tres crates (`conflict`, `audit`, `telemetry`) son **stubs de 7 líneas sin implementar**. Todas las UIs de escritorio salvo GNOME/preferences son stubs "Not yet implemented".

---

## 1. Estructura de crates

Workspace definido en `lnxdrive-engine/Cargo.toml:1-16` (11 miembros).

| Crate | Rol | Qué hace | Estado |
|---|---|---|---|
| **lnxdrive-core** | **CORE / dominio** | Dominio puro + puertos + casos de uso. Sin dependencias externas de infraestructura. `lnxdrive-engine/crates/lnxdrive-core/src/lib.rs:1-19` | Implementado (~8.5k líneas) |
| **lnxdrive-daemon** | **Binario daemon (`lnxdrived`)** | Ensambla adaptadores, arranca D-Bus, corre el bucle de sync. `crates/lnxdrive-daemon/src/main.rs` | Implementado |
| **lnxdrive-ipc** | **IPC (D-Bus)** | Define e implementa las 9 interfaces D-Bus del daemon. `crates/lnxdrive-ipc/src/lib.rs:1-41` | Implementado |
| **lnxdrive-cli** | Cliente CLI | Comando `lnxdrive` (auth, sync, status, pin, hydrate, mount, conflicts…). `crates/lnxdrive-cli/src/commands/` | Implementado |
| **lnxdrive-fuse** | Adaptador FUSE | Files-on-demand: hidratación/deshidratación, inodes, xattr. `crates/lnxdrive-fuse/` | Implementado (~10.5k líneas) |
| **lnxdrive-sync** | Motor de sync | `SyncEngine` (delta bidireccional) + watcher + scheduler + adaptador FS local. `crates/lnxdrive-sync/src/engine.rs` | Implementado |
| **lnxdrive-graph** | Adaptador nube (OneDrive) | Implementa `ICloudProvider` vía Microsoft Graph (auth, delta, upload, rate-limit). `crates/lnxdrive-graph/src/provider.rs:159` | Implementado |
| **lnxdrive-cache** | Adaptador persistencia | `SqliteStateRepository` (implementa `IStateRepository`) + pool sqlx. `crates/lnxdrive-cache/src/repository.rs:546` | Implementado |
| **lnxdrive-conflict** | (adaptador conflictos) | **STUB** — sólo doc-comment. `crates/lnxdrive-conflict/src/lib.rs:1-8` | **No implementado** |
| **lnxdrive-audit** | (audit trail) | **STUB** — sólo doc-comment. `crates/lnxdrive-audit/src/lib.rs:1-8` | **No implementado** |
| **lnxdrive-telemetry** | (métricas) | **STUB** — sólo doc-comment. `crates/lnxdrive-telemetry/src/lib.rs:1-8` | **No implementado** |

Nota: la lógica de conflictos y audit **sí existe pero en `lnxdrive-core`** (dominio: `domain/conflict.rs`, `domain/audit.rs`), no en sus crates homónimos. Los crates dedicados están vacíos.

---

## 2. Arquitectura hexagonal: puertos y adaptadores

**Puertos** (traits secundarios/driven) definidos en `lnxdrive-core/src/ports/mod.rs:14-22`:

| Puerto (trait) | Definición | Adaptador concreto |
|---|---|---|
| **`ICloudProvider`** | `ports/cloud_provider.rs:164` | `GraphCloudProvider` en `lnxdrive-graph/src/provider.rs:159` |
| **`IStateRepository`** | `ports/state_repository.rs:119` | `SqliteStateRepository` en `lnxdrive-cache/src/repository.rs:546` |
| **`ILocalFileSystem`** | `ports/local_filesystem.rs:172` | `LocalFileSystemAdapter` en `lnxdrive-sync/src/filesystem.rs` |
| **`INotificationService`** | `ports/notification.rs:132` | Sin adaptador localizado (probablemente pendiente) |
| `IFileObserver` (callback) | `ports/local_filesystem.rs:81` | — |

**Casos de uso** (orquestan dominio a través de puertos): `usecases/authenticate.rs`, `sync_file.rs`, `query_delta.rs`, `explain_failure.rs`.

### Trait `ICloudProvider` (puerto de proveedor de nube)

Definido en `lnxdrive-core/src/ports/cloud_provider.rs:163-261`. Es `Send + Sync`, `#[async_trait]`, **provider-agnóstico** (diseñado para multi-proveedor, hoy sólo OneDrive). Métodos:

- `authenticate(&AuthFlow) -> Tokens` (cloud_provider.rs:172)
- `refresh_tokens(&str) -> Tokens` (:181)
- `get_delta(Option<&DeltaToken>) -> DeltaResponse` (:193) — sync incremental
- `download_file(&RemoteId) -> Vec<u8>` (:202)
- `upload_file(&RemotePath, &str, &[u8]) -> DeltaItem` (:213) — small file (<4MB)
- `upload_file_session(..., progress) -> DeltaItem` (:233) — resumable/grande
- `get_metadata(&RemoteId) -> DeltaItem` (:248)
- `get_user_info() -> UserInfo` (:254)
- `delete_item(&RemoteId)` (:260)

DTOs de puerto asociados: `AuthFlow` (:31), `Tokens` (:55), `DeltaResponse`/`DeltaItem` (:87/:102), `UserInfo` (:132). Los errores usan `anyhow::Result` deliberadamente (comentario :10-11).

---

## 3. Backend común vs UIs de escritorio (D-Bus)

**Confirmado el patrón "daemon común + UIs delgadas".** La comunicación es vía **D-Bus session bus**, bus name `com.strangedaystech.LNXDrive`, path `/com/strangedaystech/LNXDrive` (constantes en `lnxdrive-ipc/src/service.rs:28` y `:31`).

**9 interfaces D-Bus** (macro `#[zbus::interface]` en `lnxdrive-ipc/src/service.rs`):
- `SyncController` (:194, legacy), `Account` (:289, legacy), `Conflicts` (:334), `Files` (:496), `Sync` (:613), `Status` (:726), `Auth` (:843), `Settings` (:1010), `Manager` (:1090).

El daemon las registra al arrancar `DbusService::start()` (main.rs:130-152) y adquiere el well-known name (instancia única).

**Consumo desde las UIs:**
- **GNOME preferences** (GTK4/libadwaita) — cliente real: usa `zbus::proxy` contra las interfaces `Auth`, `Settings`, etc. en `lnxdrive-gnome/preferences/src/dbus_client.rs:72-110`. Este subcrate está implementado (~797 líneas). Añade la dependencia `lnxdrive-ipc` por path en `lnxdrive-gnome/Cargo.toml`.
- **GNOME Shell extension** (JS) — define proxies vía XML introspection en `lnxdrive-gnome/shell-extension/.../dbus.js:17-30`.
- **Nautilus extension** (C) — `lnxdrive-gnome/nautilus-extension/src/lnxdrive-dbus-client.c`.

**Detalle arquitectónico importante:** las interfaces D-Bus del daemon **no invocan el `SyncEngine` directamente**; mutan un `DaemonState` compartido (`Arc<Mutex<DaemonState>>`) con flags como `sync_requested`, y el bucle del daemon los consulta en cada tick (main.rs:336-341). Es un desacople por estado compartido, no llamadas directas.

**UIs stub / no implementadas:**
- `lnxdrive-gnome/src/main.rs:9-11` → imprime "Not yet implemented" (el binario raíz GNOME es stub; lo real vive en `preferences/`).
- `experimental/lnxdrive-cosmic/src/main.rs:8-10` → stub "Not yet implemented".
- `experimental/lnxdrive-gtk3/src/main.rs:8-10` → stub (XFCE/Cinnamon/MATE).
- `experimental/lnxdrive-plasma/src/main.cpp` → esqueleto Qt6/QML (KDE), sólo `main()` inicial.

---

## 4. Componentes clave del backend

### Motor de sincronización — IMPLEMENTADO
`lnxdrive-sync/src/engine.rs`. `SyncEngine` hace delta bidireccional: pull (query delta → creates/updates/deletes), push (scan FS → upload/delete), bookkeeping del delta token (docstring :6-15). Retry con backoff exponencial 1-16s, máx 5 (`MAX_RETRIES` :81). `SyncResult` en :48. Construido en el daemon con los tres adaptadores inyectados (main.rs:237-242). Módulos hermanos: `watcher.rs`, `scheduler.rs`, `filesystem.rs`.

### FUSE / files-on-demand — IMPLEMENTADO
`lnxdrive-fuse/` (~10.5k líneas). Punto de montaje `mount()` en `lnxdrive-fuse/src/lib.rs:132`; `impl Filesystem for LnxDriveFs` en `filesystem.rs:347`. Auto-montaje desde el daemon si `config.fuse.auto_mount` (main.rs:245-296). Módulos: `hydration.rs`, `dehydration.rs`, `inode.rs`, `xattr.rs`, `write_serializer.rs`, `cache.rs`.

### Almacenamiento (sqlite/sqlx) — IMPLEMENTADO
`lnxdrive-cache/`. `SqliteStateRepository` (`repository.rs:50`) implementa `IStateRepository` (`repository.rs:546`) con ~30 métodos async (items, accounts, sessions, audit, conflicts, inodes, hydration progress). Pool en `pool.rs` (`DatabasePool`). DB en `~/.local/share/lnxdrive/lnxdrive.db` (main.rs:85-88). sqlx configurado en `Cargo.toml` (`runtime-tokio`, `sqlite`).

### State machine (files-on-demand) — IMPLEMENTADO (en core)
`lnxdrive-core/src/domain/sync_item.rs`. Enum `ItemState` (:52): `Online` (placeholder, sólo cloud), `Hydrating`, `Hydrated`, `Pinned`, `Modified`, `Conflicted`, `Error(String)`, `Deleted`. Validación de transiciones en `can_transition_to()` (:768) con reglas documentadas (:759-767); `Error` puede volver a cualquier estado (retry), `Deleted` es terminal. Helpers de estado en :72-127.

---

## Notas y banderas

- **Discrepancia crates vacíos vs dominio:** `lnxdrive-conflict`, `lnxdrive-audit`, `lnxdrive-telemetry` son stubs (7 líneas c/u), pero conflict y audit están implementados como entidades de dominio dentro de `lnxdrive-core` (`domain/conflict.rs`, `domain/audit.rs`) y expuestos por D-Bus (`Conflicts` interface). Telemetry no tiene implementación real pese a que `prometheus` está en las deps del workspace.
- **`INotificationService`** tiene puerto pero no se localizó adaptador concreto — probablemente pendiente.
- **Un solo proveedor real hoy:** aunque `ICloudProvider` es agnóstico, el único adaptador es `GraphCloudProvider` (OneDrive). No hay soporte multi-proveedor activo.
- **`lnxdrive-cli`** es un segundo cliente del daemon (además de las UIs), también consumidor de las interfaces D-Bus.
