# GUÍA DE INICIO — LNXDrive Monorepo

## INTRODUCCIÓN

Este es el monorepo de **LNXDrive**, un cliente de sincronización de archivos en la nube para Linux. Contiene todos los componentes del proyecto:

| Componente | Directorio | Fase en la Hoja de Ruta |
|------------|-----------|------------------------|
| Core daemon + CLI + crates | `lnxdrive/` | Fase 1-2 |
| Integración GNOME | `lnxdrive-gnome/` | Fase 3 |
| UI GTK3 (XFCE/MATE) | `lnxdrive-gtk3/` | Fase 4 |
| Integración KDE Plasma | `lnxdrive-plasma/` | Fase 5 |
| UI COSMIC | `lnxdrive-cosmic/` | Fase 6 |
| Distribución/Packaging | `lnxdrive-packaging/` | Fase 7 |
| Guía de diseño y desarrollo | `lnxdrive-guide/` | Transversal |
| Infraestructura de testing | `lnxdrive-testing/` | Transversal |

### Recursos clave
- **Guía de Diseño**: `lnxdrive-guide/Guía-de-diseño-y-desarrollo.md`
- **Hoja de Ruta**: `lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md`
- **DevTrail** (documentación de proceso): `.devtrail/`
- **Instrucciones para agentes IA**: `CLAUDE.md` / `GEMINI.md`

### Herramientas integradas
- **DevTrail**: Documentación de trazabilidad de desarrollo (AILOGs, AIDECs, ADRs, etc.)
- **SpecKit**: Skills para planificación e implementación (`/speckit.*`)
- **Context7**: Investigación de APIs de librerías, frameworks y servicios (MCP server)

## WORKFLOW CON SPECKIT

Para iniciar el desarrollo de cualquier componente:

```
/speckit.specify   — Crear especificaciones
/speckit.clarify   — Resolver ambigüedades
/speckit.plan      — Diseñar plan de implementación
/speckit.tasks     — Generar tareas granulares (marcar paralelizables)
/speckit.analyze   — Análisis de descubrimientos
/speckit.implement — Implementar por Stages (no usar "Fase" para etapas internas)
```

### Política de nomenclatura
Toda referencia a etapas de diseño/desarrollo **dentro** de un componente debe usar "Stage" u otra denominación. "Fase" se reserva exclusivamente para la Hoja de Ruta principal del proyecto.

---

## ESTADO ACTUAL

### Completadas
- **Fase 1-2** (Core daemon + Files-on-Demand): Implementado en `lnxdrive/`
- **Fase 3** (Integración GNOME): Implementado en `lnxdrive-gnome/`

### Pendientes
- **Fase 4** (UI GTK3): `lnxdrive-gtk3/`
- **Fase 5** (Integración KDE): `lnxdrive-plasma/`
- **Fase 6** (UI COSMIC): `lnxdrive-cosmic/`
- **Fase 7** (Distribución): `lnxdrive-packaging/`
