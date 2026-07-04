# LNXDrive — Hoja de ruta alpha → beta → 1.0

## 1. Hoja de ruta oficial (`lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md`)

Este documento es la hoja de ruta **conceptual/técnica** (por fases de implementación 0-10), NO por versión. Es más antiguo que el esquema de versiones actual y su noción de "1.0/2.0" difiere del versionado semántico vigente. Contenido:

**11 fases (0-10):**
- **Fase 0** — Infraestructura de testing (mocks Graph/wiremock, CI, containers, VM GNOME, Makefile).
- **Fase 1** — Fundamentos: core hexagonal Rust, Graph adapter, SQLite state, motor sync (upload/download), delta sync, rate limiter, CLI, servicio systemd. Entrega `lnxdrive-core`, `lnxdrive-cli`, `lnxdrive-daemon`.
- **Fase 2** — Files-on-Demand: `lnxdrive-fuse`, placeholders, hidratación/deshidratación, xattrs.
- **Fase 3** — Integración GNOME: DBus, extensión Nautilus, panel GTK4, GNOME Shell, GOA.
- **Fase 4** — Observabilidad: `lnxdrive-audit`, métricas Prometheus, logs JSON, `lnxdrive explain`, telemetría opt-in.
- **Fase 5** — Conflictos: `lnxdrive-conflict`, reglas YAML, UI resolución, integración Meld.
- **Fase 6** — Multi-cuenta: namespaces `{provider}:{alias}`, config YAML, DBus API cuentas.
- **Fase 7** — Más escritorios: KDE (Qt6/Dolphin), XFCE (Thunar), Cosmic (iced).
- **Fase 8** — Multi-proveedor: puerto `ICloudProvider`, Google Drive, Dropbox, Nextcloud/WebDAV, registry dinámico.
- **Fase 9** — Avanzado OneDrive: SharePoint/Business, shared folders, historial de versiones, share links.
- **Fase 10** — Publicar crates reutilizables en crates.io (`lnxdrive-fuse`, `-audit`, `-conflict`, `-ratelimit`) + docs.rs.

**Hitos que declara este doc (esquema antiguo, líneas 263-281):**
- **MVP** = Fases 0-3 (sync básica + FoD + GNOME).
- **Versión 1.0** = Fases 0-6 (multi-cuenta, conflictos, observabilidad).
- **Versión 2.0** = Todas las fases (multi-proveedor, todos los escritorios, crates publicados).
- Estimación total: 9-12 meses.

> Nota importante: este esquema "1.0 = fases 0-6 / 2.0 = todo" quedó **superado** por el versionado real del proyecto (alpha/beta/1.0) que se describe en el README y el Charter activo. El doc de guía no fue actualizado a ese esquema.

## 2. Docs de visión/versionado adicionales en `lnxdrive-guide/`

- `lnxdrive-guide/01-Vision/01-resumen-ejecutivo.md` — Visión (sync explicable, gobernanza YAML, UI intercambiable GNOME/KDE/Cosmic/XFCE/CLI, FoD FUSE, observabilidad). No fija metas por versión, solo diferenciadores.
- `lnxdrive-guide/01-Vision/02-principios-rectores.md`, `03-propuesta-de-valor.md` — filosofía/diferenciadores, sin versionado.
- `lnxdrive-guide/MVP-CLOSURE-PLAN.md` — plan de cierre del MVP (contexto histórico).
- `lnxdrive-guide/08-Distribucion/01-estructura-repositorios.md` y `04-internacionalizacion.md`, `05-packaging-desktop-integration.md` — mencionan versiones objetivo en contexto de packaging/i18n.

La **fuente de verdad del versionado real** NO está en `lnxdrive-guide/` sino en el `README.md` raíz, el `CHANGELOG.md` y el Charter.

## 3. Versionado real y features diferidas (fuente de verdad)

Tabla de roadmap canónica en **`README.md` (líneas 211-215):**

| Milestone | Alcance | Estado |
|-----------|---------|--------|
| **v0.1.0-alpha.1** | Motor OneDrive (sync, delta, FUSE files-on-demand), CLI, stack GNOME (panel GTK4, indicador Shell, overlays Nautilus, GOA SSO), bundle Flatpak | **Current** |
| **v0.2.0-beta** | Grupo de ajustes de Sistema (auto-start, cache, dehydration), RPM/DEB/AUR/AppImage, envío a Flathub, fallback D-Bus Unix-socket, estructura i18n, telemetría opt-in | Planned |
| **v1.0.0** | Front-ends KDE Plasma, COSMIC y GTK3 (XFCE/MATE), multi-proveedor (Google Drive, Dropbox), 5+ idiomas | Planned |

### Diferido explícitamente a **v0.2.0-beta**
Fuentes: `README.md:80,117,214`, `CHANGELOG.md:45,58`, Charter `## Out of scope` (`.straymark/charters/01-road-to-v0-1-0-alpha-1.md:39,41-47`):
- **Grupo de ajustes "System"** del panel GTK4 (auto-start, cache, política de deshidratación) — requiere nueva API D-Bus del daemon (ver `AIDEC-2026-05-31-001-defer-system-settings-group.md`).
- **Formatos de empaquetado**: RPM, DEB, AUR, AppImage (el alpha solo lleva Flatpak).
- **Envío a Flathub** (submission) — exige vendoring de dependencias Cargo y wiring definitivo del sandbox FUSE.
- **Fallback completo D-Bus por Unix-socket** (el alpha solo lleva el health-monitor + reconnect de RISK-001).
- **Estructura i18n / traducciones** (las 5+ lenguas van a v1.0.0).
- **Telemetría / crash reporting** opt-in.
- **Landing page** en strangedays.tech.
- **Cobertura formal `cargo tarpaulin`** (best-effort en alpha, objetivo formal en beta).

En el alpha, el bundle Flatpak se publica solo en GitHub Releases, NO incluye las extensiones Nautilus/Shell/GOA (cargan en procesos host) y FUSE bajo sandbox requiere `--device=all`, solo smoke-tested en VM (`CHANGELOG.md:52-58`).

### Diferido explícitamente a **v1.0.0**
Fuentes: `experimental/README.md:12-14,30,34`, `README.md:160-162,215`, `CLAUDE.md:23-25,51-53`, Charter `:40,43`:
- **UIs experimentales reactivan en v1.0.0**:
  - `experimental/lnxdrive-gtk3/` (Rust+GTK3, XFCE/MATE) — hoy stub `println!("not yet implemented")`.
  - `experimental/lnxdrive-plasma/` (C++/Qt6/KDE Frameworks, Plasma 6) — `main.cpp` arranca `KApplication`, motor QML `TODO`, `plasmoid/` y `dolphin-plugin/` vacíos.
  - `experimental/lnxdrive-cosmic/` (Rust+libcosmic) — stub.
  - Secuencia esperada de reactivación (`experimental/README.md:37-45`): primero Plasma, luego COSMIC (junto al primer stable de COSMIC de System76), luego GTK3. Cada reactivación será su propio StrayMark Charter. Requisito previo: escribir un documento de contrato D-Bus en `lnxdrive-guide/04-Componentes/`.
- **Multi-proveedor**: Google Drive, Dropbox.
- **5+ idiomas** (i18n completo).

## 4. Estado actual declarado

- **Versión objetivo en curso: `v0.1.0-alpha.1`** — marcada **"Current"** en README; el CHANGELOG tiene la entrada `[0.1.0-alpha.1]` con fecha "set at tag time (Charter-01 Fase 6)", es decir el tag aún no se ha cortado.
- **Charter activo: `CHARTER-01-road-to-v0-1-0-alpha-1`** (`.straymark/charters/01-road-to-v0-1-0-alpha-1.md`), **`status: in-progress`**, iniciado 2026-05-29, esfuerzo L (~5-7 semanas). Es el único charter en `.straymark/charters/`.
- **Progreso del Charter (7 fases 0-6):**
  - Fase 0 (gobernanza, archivado de UIs a `experimental/`, milestones) — hecho.
  - Fase 1 (seguridad P0: RISK-002 tokens OAuth→keyring, RISK-003 FUSE per-inode lock, RISK-001 D-Bus health monitor, ISSUE-002 YAML hardening, cargo deny/audit en CI) — hecho.
  - Fase 2 (engine polish, T101 perf validado) — **Done**.
  - Fase 3 (panel GTK4, 3 grupos + Conflicts, verificado en VM GNOME Wayland) — **Done**.
  - Fase 4 (packaging Flatpak, `org.gnome.Platform 49`, SPDX corregido) — **Done**.
  - Fase 5 (release infra: `release.yml`, `SECURITY.md`, `CHANGELOG.md`, screenshots, unificación de versión, README) — hecho.
  - Fase 6 (tag firmado `v0.1.0-alpha.1`, pre-release GitHub, anuncios) — **pendiente**; el Charter sigue `in-progress` y el CHANGELOG indica que la fecha se fija al taggear.

**Conclusión de estado:** el proyecto está en **v0.1.0-alpha.1, en progreso**, ejecutando las últimas remediaciones de auditoría de Fases 4-5 previas al tag/release (Fase 6). El Charter-01 sigue abierto (`in-progress`).

---

## Roadmap sintetizado (alpha → beta → 1.0)

- **v0.1.0-alpha.1 (actual, casi listo para tag):** motor OneDrive completo (sync + delta + FUSE files-on-demand), CLI, stack GNOME (panel GTK4 con Account/Folders/Network/Conflicts, indicador Shell, overlays Nautilus, GOA SSO), bundle Flatpak en GitHub Releases con SHA256SUMS; 4 riesgos P0 de seguridad cerrados. Solo GNOME.
- **v0.2.0-beta (planned):** grupo System del panel (auto-start/cache/dehydration + nueva API D-Bus), packaging RPM/DEB/AUR/AppImage, submission a Flathub (+ vendoring), fallback D-Bus Unix-socket completo, estructura i18n, telemetría opt-in, landing page, cobertura formal.
- **v1.0.0 (planned):** reactivación de las 3 UIs de `experimental/` (KDE Plasma → COSMIC → GTK3/XFCE-MATE, cada una su propio Charter, tras estabilizar y documentar el contrato D-Bus), multi-proveedor (Google Drive, Dropbox) y 5+ idiomas.

### Rutas clave
- `README.md` (líneas 209-218: tabla roadmap canónica)
- `CHANGELOG.md` (entrada 0.1.0-alpha.1 + diferimientos a v0.2)
- `experimental/README.md` (UIs diferidas a v1.0.0)
- `.straymark/charters/01-road-to-v0-1-0-alpha-1.md` (Charter activo, scope/out-of-scope, fases)
- `lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md` (hoja de ruta técnica por fases, esquema antiguo)
- `CLAUDE.md` (líneas 23-25, 51-53: reactivación UIs en v1.0.0)
- `.straymark/07-ai-audit/decisions/AIDEC-2026-05-31-001-defer-system-settings-group.md` (defer grupo System a v0.2)
