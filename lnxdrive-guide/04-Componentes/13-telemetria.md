# Telemetría Interna (Auto-observación)

> **Ubicación:** `04-Componentes/13-telemetria.md`
> **Relacionado:** [Logging y Tracing](../06-Testing/07-logging-tracing.md), [Auditoría](12-auditoria.md), [Configuración YAML](../05-Implementacion/05-configuracion-yaml.md)
> **Reescrito:** 2026-07-04 conforme a [ADR-2026-07-04-002](../../.straymark/02-design/decisions/ADR-2026-07-04-002-telemetria-interna-only.md)

---

> [!IMPORTANT]
> **Decisión de producto (ADR-2026-07-04-002, aprobada por el operador):**
> la telemetría de LNXDrive es **interna-only**. No existe — ni existirá en este
> componente — ningún mecanismo de export de datos hacia los desarrolladores.
> El diseño anterior de este documento (export OTLP/gRPC → backend
> OpenTelemetry en Google Cloud, Anonymizer, `lnxdrive report send`, flujo de
> consentimiento opt-in) queda **descartado**; se conserva en el historial de
> git como referencia. La garantía al adoptante es absoluta y verificable:
> *ningún dato sale de tu equipo hacia nosotros — no hay mecanismo para ello*.

---

## Parte XIII: Sistema de Telemetría Interna

### 13.1 Propósito

`lnxdrive-telemetry` es el componente de **auto-observación** del sistema: LNXDrive
determina su propio estado para **avisar al usuario** y, cuando procede,
**reaccionar localmente**. Dos consumidores, ambos dentro del equipo del usuario:

1. **El propio sistema** — detección de condiciones anómalas y acciones
   correctivas locales.
2. **El usuario** — avisos accionables y artefactos de diagnóstico locales
   (reports) que puede inspeccionar y, si él quiere, adjuntar manualmente a un
   issue de GitHub.

### 13.2 Principios Fundamentales

| Principio | Descripción |
|-----------|-------------|
| **Local-only** | Todos los artefactos viven y mueren en el equipo del usuario. No hay ruta de salida |
| **Auto-conocimiento** | El sistema sabe si está sano: salud del daemon, progreso/errores de sync, presión de recursos |
| **Avisos accionables** | Cuando algo requiere al usuario (re-autenticar, disco lleno, fallos repetidos), se le avisa con contexto |
| **Resiliencia** | Fallos del agente de telemetría no afectan la sincronización |
| **Transparencia** | El usuario puede inspeccionar cada artefacto (`lnxdrive report list/view`) |

---

### 13.3 Arquitectura

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ARQUITECTURA DE TELEMETRÍA INTERNA                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                       │
│  │   Daemon     │  │    FUSE      │  │   CLI/UI     │                       │
│  │  (tracing)   │  │  (tracing)   │  │  (tracing)   │                       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                       │
│         │                 │                 │                               │
│         │  Panic hooks + error collectors   │                               │
│         └────────────────┬┴─────────────────┘                               │
│                          ▼                                                  │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                    ALMACENAMIENTO LOCAL                               │  │
│  │  ~/.local/share/lnxdrive/reports/                                     │  │
│  │                                                                       │  │
│  │  • crash-2026-01-31-abc123.json   (crash reports)                     │  │
│  │  • error-2026-01-31-def456.json   (errores no-fatales)                │  │
│  │  • system-info.json               (hardware, OS, versión)             │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                          ▲                                                  │
│                          │                                                  │
│                  ┌───────┴───────────────────────────────┐                  │
│                  │    lnxdrive-telemetry                 │                  │
│                  │    (agente de auto-observación)       │                  │
│                  │                                       │                  │
│                  │  • Evalúa salud del sistema           │                  │
│                  │  • Aplica reglas de umbral            │                  │
│                  │  • Emite avisos al usuario            │                  │
│                  │    (INotificationService)             │                  │
│                  │  • Dispara acciones locales           │                  │
│                  └───────────────────────────────────────┘                  │
│                                                                             │
│  ═══ No existe ninguna flecha hacia fuera de este diagrama. ═══             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 13.4 Agente de Auto-observación

El agente evalúa **señales de salud** contra **reglas de umbral** y produce
**avisos o acciones locales**. Ejemplos del catálogo inicial:

| Señal observada | Umbral (ejemplo) | Respuesta |
|---|---|---|
| Fallos consecutivos de sync | ≥ 5 en 10 min | Aviso al usuario con la causa dominante (vía `lnxdrive explain`) |
| Token próximo a expirar sin refresh exitoso | < 10 min restantes | Aviso "re-autenticación necesaria" con acción directa |
| Espacio en disco bajo | < umbral configurable | Aviso + sugerencia de deshidratar; pausa de hidrataciones nuevas |
| Salud D-Bus degradada (`dbus_health`) | reconexiones repetidas | Aviso de diagnóstico |
| Backlog de cola de sync creciendo sin drenar | tendencia sostenida | Aviso "sync atascado" con puntero al reporte |
| Panic/crash de cualquier componente | inmediato | Crash report local + aviso con `report view <id>` |

```rust
// crates/lnxdrive-telemetry/src/main.rs (esquema)

#[tokio::main]
async fn main() -> Result<()> {
    let config = TelemetryConfig::load()?;

    let collector = LocalReportCollector::new(&config.reports_dir)?;
    let health = HealthEvaluator::new(&config.rules);
    let notifier = NotificationAdapter::new()?;   // puerto INotificationService

    loop {
        // 1. Evaluar señales de salud (D-Bus Status, journal de errores, disco)
        for finding in health.evaluate().await? {
            match finding.action {
                Action::Notify(msg) => notifier.warn(&msg).await?,
                Action::LocalMitigation(m) => m.apply().await?,   // p. ej. pausar hidrataciones
            }
        }

        // 2. Registrar artefactos locales pendientes (crash/error reports)
        collector.collect_pending()?;

        tokio::time::sleep(config.check_interval).await;
    }
}
```

Propiedades del proceso (sin cambios respecto al diseño original):

1. **No afecta al daemon**: errores del agente no causan fallos de sincronización.
2. **Eficiente**: bajo consumo (`Nice=19`, `MemoryMax=50M`), evaluación por lotes.

---

### 13.5 Generación de Reportes Locales

#### Crash Reports (Panics)

```rust
// crates/lnxdrive-telemetry/src/panic_handler.rs

pub fn install_crash_reporter(reports_dir: &Path) {
    let reports_dir = reports_dir.to_path_buf();
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let crash_report = CrashReport {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            component: std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned())),
            panic_message: panic_info.payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string()),
            location: panic_info.location().map(|l| format!("{}:{}:{}",
                l.file(), l.line(), l.column())),
            backtrace: std::backtrace::Backtrace::capture().to_string(),
            os_info: OsInfo::collect(),
        };

        // Guardar localmente (nunca falla silenciosamente)
        if let Err(e) = save_crash_report(&reports_dir, &crash_report) {
            eprintln!("Failed to save crash report: {}", e);
        }

        // Llamar hook original
        default_hook(panic_info);
    }));
}

#[derive(Serialize)]
struct OsInfo {
    os: String,           // "linux"
    kernel: String,       // "6.5.0"
    desktop: String,      // "GNOME 45"
    arch: String,         // "x86_64"
    // NO incluir: hostname, username, locale, timezone
    // (el reporte es local, pero el usuario puede decidir adjuntarlo a un
    //  issue público — se genera limpio de PII desde el origen)
}
```

#### Error Reports (No Fatales)

```rust
// crates/lnxdrive-telemetry/src/error_reporter.rs

impl ErrorReporter {
    pub fn report(&self, error: &impl std::error::Error, context: &str) {
        // Solo registrar errores "interesantes" (no timeouts comunes, etc.)
        if !Self::should_report(error) {
            return;
        }
        // ... construir ErrorReport y guardarlo en reports_dir (local)
    }

    fn should_report(error: &impl std::error::Error) -> bool {
        let msg = error.to_string().to_lowercase();
        !msg.contains("timeout") &&
        !msg.contains("connection refused") &&
        !msg.contains("rate limit")
    }
}
```

> Los reportes se generan **limpios de PII desde el origen** (basename en vez de
> ruta completa donde sea posible, sin hostname/username). No porque se envíen —
> no se envían — sino porque el destino típico de un reporte que el usuario
> decide compartir es un issue público de GitHub, y debe ser seguro pegarlo.

---

### 13.6 CLI: Gestión de Reportes Locales

```bash
# Ver reportes locales
lnxdrive report list

# Salida:
# ID                                    Type    Date        Size
# ────────────────────────────────────────────────────────────────
# crash-abc123                          crash   2026-01-31  4.2 KB
# error-def456                          error   2026-01-31  1.1 KB
#
# Total: 2 reports (5.3 KB)

# Ver contenido de un reporte (para inspección o para adjuntar a un issue)
lnxdrive report view crash-abc123

# Eliminar reportes
lnxdrive report delete crash-abc123
lnxdrive report delete --all
```

> [!NOTE]
> No existe `lnxdrive report send`. Si quieres compartir un reporte con el
> proyecto, `report view` y pégalo en un issue de GitHub — decisión tuya, acción
> tuya, cada vez.

---

### 13.7 Relación con Otros Sistemas

| Sistema | Propósito | Ámbito |
|---------|-----------|--------|
| **journald** | Logs locales del usuario | Local |
| **tracing (Rust)** | Logging estructurado | Local (fuente de señales del agente) |
| **Prometheus** | Métricas operacionales, scraping por el usuario | Local (`127.0.0.1`) |
| **Auditoría** ([12-auditoria.md](12-auditoria.md)) | "¿Por qué pasó esto?" — trazabilidad de decisiones de sync | Local (SQLite) |
| **Telemetría interna** | "¿Estoy sano?" — salud, avisos, reacciones | Local |

> [!WARNING]
> Ninguno de estos sistemas tiene ruta de salida hacia los desarrolladores.
> La única conexión de red de LNXDrive es hacia el proveedor de nube que el
> usuario configuró.

---

### 13.8 Seguridad y Privacidad

La garantía deja de ser una política y pasa a ser una **propiedad estructural**:

- **No hay código de export**: el crate no incluye cliente OTLP ni endpoint
  remoto alguno. La revisión de dependencias (`cargo deny`) lo hace auditable.
- **Criterio de test** (ver [Estrategia de Testing §3.3](../06-Testing/01-estrategia-testing.md)):
  bajo operación completa, las únicas conexiones salientes observables apuntan
  al proveedor de nube configurado. Test de regresión en la suite de seguridad.
- **Reports limpios de PII desde el origen** (§13.5), porque su destino
  voluntario típico es un issue público.

---

### 13.9 Integración con systemd

```ini
# ~/.config/systemd/user/lnxdrive-telemetry.service
[Unit]
Description=LNXDrive Internal Telemetry (self-observation agent)
After=default.target

[Service]
Type=simple
ExecStart=%h/.local/bin/lnxdrive-telemetry
Restart=on-failure
RestartSec=60

# Aislamiento: no afecta al daemon principal
Nice=19
IOSchedulingClass=idle
MemoryMax=50M

[Install]
WantedBy=default.target
```

> Nota: ya no depende de `network-online.target` — el agente no usa la red.

---

## ⚠️ Riesgos y Mitigaciones

### T1: Exfiltración de Datos Sensibles

| Atributo | Valor |
|----------|-------|
| **Prioridad** | P0 (Crítica) |
| **Estado** | **Eliminado por diseño** (ADR-2026-07-04-002) |

Sin ruta de export, la clase de riesgo desaparece estructuralmente. Lo que
queda es **vigilar que siga siendo verdad**:

**Tests Requeridos:**
- `test_no_external_connections` — cero egress no-proveedor bajo operación completa
- `test_no_pii_in_reports` — reports locales limpios (rutas, usernames, tokens)

### T2: Degradación de Rendimiento

| Atributo | Valor |
|----------|-------|
| **Prioridad** | P1 (Alta) |
| **Componentes** | proceso lnxdrive-telemetry |

**Mitigación:**
- Proceso separado con `Nice=19` y `IOSchedulingClass=idle`
- Límite de memoria (`MemoryMax=50M`)
- Evaluación por lotes, intervalos configurables

### T3: Fatiga de Avisos

| Atributo | Valor |
|----------|-------|
| **Prioridad** | P2 (Media) |
| **Componentes** | HealthEvaluator, NotificationAdapter |

**Mitigación:**
- Umbrales conservadores por defecto; deduplicación de avisos repetidos
- Todo aviso debe ser accionable (qué pasó + qué puedo hacer)

---

> [!NOTE]
> Para la matriz completa de riesgos, ver:
> - [TRACE-risks-mitigations.md](../../.straymark/02-design/risk-analysis/TRACE-risks-mitigations.md)

---

## Ver también

- [Auditoría](12-auditoria.md) - Trazabilidad de decisiones ("¿por qué?")
- [Logging y Tracing](../06-Testing/07-logging-tracing.md) - Sistema de logging local
- [Configuración YAML](../05-Implementacion/05-configuracion-yaml.md) - Opciones de telemetría
- [CLI](06-cli.md) - Subcomando `report`
- [ADR-2026-07-04-002](../../.straymark/02-design/decisions/ADR-2026-07-04-002-telemetria-interna-only.md) - La decisión que gobierna este documento
