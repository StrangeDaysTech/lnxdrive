# Estrategia de Testing

> **Ubicación:** `06-Testing/01-estrategia-testing.md`
> **Relacionado:** [Anexo A: Metodologías de Testing](../Anexos/A-metodologias-testing-completo.md), [Hoja de Ruta](../09-Referencia/02-hoja-de-ruta.md)
> **Actualizado:** 2026-07-04 (replanteo — nuevo tier E2E-real, §3)

---

## 1. Introduccion y Problematica

LNXDrive involucra tres categorias de componentes que interactuan profundamente con el sistema operativo:

| Componente | Riesgo si falla | Impacto potencial |
|------------|-----------------|-------------------|
| **Servicios systemd** | Medio-Alto | Procesos zombie, recursos no liberados, conflictos de puertos |
| **FUSE filesystem** | Alto | Kernel panic (raro), procesos colgados en I/O, puntos de montaje huerfanos |
| **Extensiones desktop** | Alto | GNOME Shell crash, Nautilus inutilizable, sesion grafica corrupta |

**El desafio**: Desarrollar y depurar estos componentes sin:
- Bloquear la sesion de trabajo del desarrollador
- Corromper el entorno de escritorio
- Dejar el sistema en estado inconsistente
- Perder trabajo por crashes del entorno grafico

---

## 2. Estrategias de Aislamiento

### 2.1 Niveles de Aislamiento Disponibles

```
+---------------------------------------------------------------------------+
|  ESPECTRO DE AISLAMIENTO                                                  |
+---------------------------------------------------------------------------+
|                                                                           |
|  Menor aislamiento <------------------------------> Mayor aislamiento     |
|  Mayor velocidad                                      Menor velocidad     |
|                                                                           |
|  +---------+ +---------+ +---------+ +---------+ +---------+              |
|  | Proceso | |Namespace| |Container| |   VM    | | VM con  |              |
|  | normal  | | aislado | |(Podman) | | headless| | GUI     |              |
|  +---------+ +---------+ +---------+ +---------+ +---------+              |
|       |           |           |           |           |                   |
|       v           v           v           v           v                   |
|   Unit tests   FUSE dev   systemd    FUSE+systemd  Desktop                |
|   Mocks        aislado    testing    integration   extensions             |
|                                                                           |
+---------------------------------------------------------------------------+
```

### 2.2 Recomendacion por Componente

| Componente | Desarrollo | Testing Unitario | Testing Integracion | Testing E2E |
|------------|------------|------------------|---------------------|-------------|
| Core (logica) | Host directo | Host directo | Host directo | Container |
| FUSE | Namespace/Container | Container | VM headless | VM headless |
| systemd service | User systemd | Container con systemd | VM headless | VM headless |
| Nautilus extension | Container GUI | Container GUI | VM con GNOME | VM con GNOME |
| GNOME Shell ext | VM con GNOME | VM con GNOME | VM con GNOME | VM con GNOME |

---

## 3. El Tier E2E-real (gate de milestones)

> Añadido en el replanteo del 2026-07-04. Diagnóstico que lo motiva:
> hasta esa fecha, **toda** la pirámide de testing (unit → integración → "E2E")
> corría contra mocks (`wiremock`/`mock-graph`/`mock-dbus-daemon.py`); "entorno
> real" siempre significó *escritorio Linux real*, nunca *nube real*. Eso hizo
> estructuralmente imposible falsificar la afirmación "funciona": el login GUI
> estuvo roto durante meses mientras el mock lo fingía completo.

### 3.1 Definición

**E2E-real** = pruebas contra una **cuenta real de pruebas** del proveedor de
nube (OneDrive), ejecutadas por el operador en su máquina.

| Propiedad | Valor |
|-----------|-------|
| Cuenta | Dedicada a pruebas (nunca la personal del operador) |
| Automatización | Tests `#[ignore]` que el operador corre localmente + guiones manuales por milestone |
| CI | **No corre en CI** (requiere credenciales reales y entorno gráfico) — mismo patrón que el gate FUSE real-mount (test T101 `#[ignore]`) |
| Guiones | `new-guide/09-guiones-verificacion.md`, escritos al **abrir** cada milestone |

### 3.2 La regla anti-mock

> **Ningún milestone de capacidad se cierra contra mock.**

Los mocks conservan su papel (CI de componentes, desarrollo aislado, regresión
rápida) — son la base de las secciones §1–§2 y de
[Mocking de APIs](05-mocking-apis.md). Lo que esta regla prohíbe es usar su
resultado como evidencia de capacidad: un hito de la
[hoja de ruta](../09-Referencia/02-hoja-de-ruta.md) solo se marca demostrado
cuando el guion E2E-real correspondiente pasó contra el servicio real.

### 3.3 Complemento: verificación de promesa de privacidad

Bajo operación completa (sync + files-on-demand + panel), las únicas conexiones
salientes observables deben apuntar al proveedor de nube configurado — LNXDrive
no tiene mecanismo de export de datos hacia sus desarrolladores
([ADR-2026-07-04-002](../../.straymark/02-design/decisions/ADR-2026-07-04-002-telemetria-interna-only.md)).
Este criterio se integra al [Testing de Seguridad](09-testing-seguridad.md).

---

## 4. Tests Derivados del Análisis de Riesgos

El [análisis de riesgos](../../.straymark/02-design/risk-analysis/TRACE-risks-mitigations.md) identificó casos de prueba específicos para mitigar cada riesgo. La matriz completa de mitigación a test se encuentra en ese documento.

### Resumen de Cobertura por Prioridad

| Prioridad | Riesgos | Tests Requeridos | Documento |
|-----------|---------|------------------|-----------|
| **P0** | D1, D2, D5, A3 | 12+ | [Testing Seguridad](09-testing-seguridad.md) |
| **P1** | A1, A2, B5, C3, D3, C6 | 18+ | Componentes individuales |
| **P2** | C1, C2, C5, A4 | 10+ | Componentes individuales |
| **P3** | B6, F3, F4 | 6+ | [Configuración](05-mocking-apis.md) |

> [!TIP]
> Cada componente incluye sección "⚠️ Riesgos y Mitigaciones" con tests específicos a implementar.

---

## Ver tambien

- [Testing de Servicios systemd](02-testing-systemd.md) - Desarrollo y testing de servicios systemd
- [Testing de FUSE Filesystem](03-testing-fuse.md) - Testing del sistema de archivos FUSE
- [Testing de Extensiones Desktop](04-testing-desktop.md) - Testing de Nautilus y GNOME Shell
- [Mocking de APIs Externas](05-mocking-apis.md) - Mock de Microsoft Graph API
- [Pipeline CI/CD](06-ci-cd-pipeline.md) - Integracion continua y despliegue
- [Testing de Seguridad](09-testing-seguridad.md) - Tests de riesgos de seguridad D1-D5

