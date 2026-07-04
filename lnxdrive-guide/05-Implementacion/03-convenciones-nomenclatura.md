# Convenciones de Nomenclatura

> **Ubicacion:** `05-Implementacion/03-convenciones-nomenclatura.md`
> **Relacionado:** [Patrones Rust](04-patrones-rust.md), [Stack Tecnologico](01-stack-tecnologico.md)

---

## Estructura del Repositorio Mono-repo

> [!NOTE]
> **Actualizado 2026-07-04.** La **lista canónica** de crates del workspace es la
> de [Estructura de Repositorios](../08-Distribucion/01-estructura-repositorios.md)
> (11 crates bajo `lnxdrive-engine/crates/` — no existe `apps/`). Este documento
> ya no mantiene una lista propia; la versión anterior divergía (incluía
> `lnxdrive-ratelimit` y `lnxdrive-state` como crates, que hoy viven dentro de
> `lnxdrive-graph` y `lnxdrive-cache` respectivamente y son solo candidatos de
> extracción publicable post-1.0).

```
┌─────────────────────────────────────────────────────────────────────┐
│  Repositorio mono-repo con workspace Cargo:                         │
│                                                                     │
│  lnxdrive-engine/                                                   │
│  ├── Cargo.toml              (workspace root)                       │
│  └── crates/                 → lista canónica en 08-Distribucion/01 │
│                                                                     │
│  Versionado:                                                        │
│  • Crates internos: version del proyecto (0.1.0-alpha.1 unificada)  │
│  • Crates publicados (post-1.0): semver independiente               │
│  • Binarios/apps: version unificada del release                     │
│                                                                     │
│  Publicacion (post-1.0, tras restaurar delimitacion de crates —     │
│  ADR-2026-07-04-001):                                               │
│  • crates.io para Rust · docs.rs · ejemplos por crate               │
└─────────────────────────────────────────────────────────────────────┘
```

## 19.1 Convenciones de Nomenclatura (Rust API Guidelines)

### Reglas de Casing

| Construccion | Convencion | Ejemplos |
|--------------|------------|----------|
| Tipos, Traits | `UpperCamelCase` | `FileSync`, `SyncError`, `WatchManager` |
| Funciones, Metodos, Variables | `snake_case` | `sync_file()`, `buffer_size`, `last_modified` |
| Constantes, Statics | `SCREAMING_SNAKE_CASE` | `MAX_BUFFER_SIZE`, `DEFAULT_TIMEOUT` |
| Modulos | `snake_case` | `file_watcher`, `sync_engine` |
| Features de Cargo | `kebab-case` | `async-std`, `full-sync` |

### Reglas para Acronimos

```rust
// UpperCamelCase: acronimo como una palabra
pub struct HttpClient;     // ✓ Correcto
pub struct HTTPClient;     // ✗ Incorrecto

pub struct JsonParser;     // ✓ Correcto
pub struct JSONParser;     // ✗ Incorrecto

pub struct Uuid;           // ✓ Correcto
pub struct UUID;           // ✗ Incorrecto

// snake_case: acronimo en minusculas
fn parse_json() {}         // ✓ Correcto
fn parse_JSON() {}         // ✗ Incorrecto

let http_client = ...;     // ✓ Correcto
let HTTP_client = ...;     // ✗ Incorrecto
```

### Convenciones de Metodos

```rust
// Getters: SIN prefijo "get_" para acceso simple
impl FileEntry {
    // ✓ Correcto - acceso simple
    pub fn path(&self) -> &Path { &self.path }
    pub fn size(&self) -> u64 { self.size }
    pub fn is_directory(&self) -> bool { self.is_dir }

    // ✓ Correcto - usar "get" solo cuando hay ambiguedad
    pub fn get(&self, key: &str) -> Option<&Value> { ... }
}

// Conversiones: seguir convencion as_/to_/into_
impl SyncPath {
    // as_ : borrowed → borrowed (barato)
    pub fn as_path(&self) -> &Path { &self.0 }
    pub fn as_str(&self) -> &str { ... }

    // to_ : borrowed → owned (costoso, puede fallar)
    pub fn to_string(&self) -> String { ... }
    pub fn to_path_buf(&self) -> PathBuf { ... }

    // into_ : owned → owned (consume self)
    pub fn into_path_buf(self) -> PathBuf { self.0 }
    pub fn into_string(self) -> String { ... }
}

// Predicados: prefijo is_, has_, can_, should_
impl FileEntry {
    pub fn is_hidden(&self) -> bool { ... }
    pub fn has_conflicts(&self) -> bool { ... }
    pub fn can_sync(&self) -> bool { ... }
    pub fn should_upload(&self) -> bool { ... }
}
```

## Nombres de Crates del Proyecto

Lista canónica de los 11 crates del workspace `lnxdrive-engine/` (coincide con
[Estructura de Repositorios](../08-Distribucion/01-estructura-repositorios.md)):

| Crate | Proposito | Publicacion futura (post-1.0) |
|-------|-----------|-------------------------------|
| `lnxdrive-core` | Dominio + puertos + casos de uso (hexagonal) | Interno |
| `lnxdrive-daemon` | Binario del daemon (`lnxdrived`) | App |
| `lnxdrive-ipc` | Librería D-Bus para clientes | Interno |
| `lnxdrive-cli` | Binario CLI | App |
| `lnxdrive-fuse` | Filesystem virtual FUSE | crates.io |
| `lnxdrive-sync` | Motor de sincronización (incluye file watching) | Interno |
| `lnxdrive-graph` | Cliente Microsoft Graph (incluye rate limiting) | Interno |
| `lnxdrive-cache` | Persistencia SQLite (incluye gestión de estado) | Interno |
| `lnxdrive-conflict` | Detección/resolución de conflictos (se puebla en M6) | crates.io |
| `lnxdrive-audit` | Motor de auditoría (en migración desde core, v0.2) | crates.io |
| `lnxdrive-telemetry` | Auto-observación interna-only (v0.2, ADR-2026-07-04-002) | Interno |

Las UIs de escritorio **no** son crates del engine: `lnxdrive-gnome/` es su
propio árbol (Meson + Rust), y `experimental/lnxdrive-{gtk3,plasma,cosmic}/`
reactivan en v1.0.0. `lnxdrive-ratelimit` y `lnxdrive-state` no existen como
crates: son candidatos de **extracción publicable** post-1.0 desde
`lnxdrive-graph`/`lnxdrive-cache` (ver
[Artefactos Reutilizables](../07-Extensibilidad/04-artefactos-reutilizables.md)).

## Estructura de Directorios del Crate Core

```
lnxdrive-core/
├── src/
│   ├── lib.rs
│   │
│   ├── domain/                 # Capa de dominio (sin dependencias externas)
│   │   ├── mod.rs
│   │   ├── models/             # Entidades y value objects
│   │   │   ├── mod.rs
│   │   │   ├── file_entry.rs
│   │   │   ├── sync_state.rs
│   │   │   └── conflict.rs
│   │   ├── errors.rs           # Errores de dominio
│   │   └── services/           # Logica de negocio pura
│   │       ├── mod.rs
│   │       ├── conflict_resolver.rs
│   │       └── delta_calculator.rs
│   │
│   ├── ports/                  # Interfaces (traits) - contratos
│   │   ├── mod.rs
│   │   ├── inbound/            # Ports de entrada (casos de uso)
│   │   │   ├── mod.rs
│   │   │   ├── sync_service.rs
│   │   │   └── file_service.rs
│   │   └── outbound/           # Ports de salida (dependencias)
│   │       ├── mod.rs
│   │       ├── file_system.rs
│   │       ├── cloud_storage.rs
│   │       ├── cache.rs
│   │       └── event_bus.rs
│   │
│   ├── adapters/               # Implementaciones concretas
│   │   ├── mod.rs
│   │   ├── inbound/            # Adaptadores de entrada
│   │   │   ├── mod.rs
│   │   │   ├── cli_handler.rs
│   │   │   └── dbus_handler.rs
│   │   └── outbound/           # Adaptadores de salida
│   │       ├── mod.rs
│   │       ├── local_fs.rs
│   │       ├── onedrive_api.rs
│   │       ├── sqlite_cache.rs
│   │       └── tokio_events.rs
│   │
│   └── application/            # Orquestacion y DI
│       ├── mod.rs
│       ├── sync_coordinator.rs
│       └── container.rs        # Dependency injection
```

---

## Ver tambien

- [Patrones Rust](04-patrones-rust.md) - Patrones de diseno idiomaticos
- [Stack Tecnologico](01-stack-tecnologico.md) - Vision general del stack
- [Arquitectura Hexagonal](../03-Arquitectura/01-arquitectura-hexagonal.md) - Filosofia arquitectonica
