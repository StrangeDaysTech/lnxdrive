# LNXDrive Monorepo — Agent Instructions

<!-- straymark:begin -->
> **Read and follow the rules in [STRAYMARK.md](STRAYMARK.md).**
> That file contains all StrayMark documentation governance rules for this project.
<!-- straymark:end -->

> This file is automatically loaded at the start of each session.
> It contains **monorepo-specific** navigation and conventions that complement
> the StrayMark governance rules. For documentation rules, templates, autonomy
> limits and the pre-commit checklist, see [STRAYMARK.md](STRAYMARK.md) and
> `.straymark/00-governance/`.

---

## 1. Project Structure (Monorepo)

This is a monorepo containing all LNXDrive components:

| Directory | Description | Tech Stack |
|-----------|-------------|------------|
| `lnxdrive-engine/` | Core daemon + library crates | Rust 1.75+, Cargo workspace (12 crates), tokio, zbus, sqlx |
| `lnxdrive-gnome/` | GNOME Shell/Nautilus/GOA integration | Meson + Rust (gtk4-rs), GJS (ES modules), C11 (Nautilus), Python |
| `experimental/lnxdrive-gtk3/` | XFCE/MATE UI (archived for v0.1.0-alpha — reactivates in v1.0.0) | Rust, Cargo, GTK3 |
| `experimental/lnxdrive-plasma/` | KDE Plasma integration (archived for v0.1.0-alpha — reactivates in v1.0.0) | C++, CMake, Qt/KDE |
| `experimental/lnxdrive-cosmic/` | COSMIC desktop UI (archived for v0.1.0-alpha — reactivates in v1.0.0) | Rust, Cargo |
| `lnxdrive-packaging/` | Distribution packages | Flatpak, AppImage, Debian, AUR |
| `lnxdrive-guide/` | Design & development guide | Markdown (Spanish) |
| `lnxdrive-testing/` | Container/VM test infrastructure | Podman, QEMU/libvirt, shell scripts |

### Key Paths
- **Design guide index**: `lnxdrive-guide/Guía-de-diseño-y-desarrollo.md`
- **StrayMark root**: `.straymark/` (single instance for entire monorepo)
- **D-Bus name**: `com.strangedaystech.LNXDrive` on `/com/strangedaystech/LNXDrive`

---

## 2. StrayMark in this monorepo

The canonical governance rules live in [STRAYMARK.md](STRAYMARK.md) (auto-managed
by `straymark` CLI). The notes below cover **only** what is specific to this monorepo.

### Agent-Logs Organization (Monorepo)

AILOGs are organized **by component** under `.straymark/07-ai-audit/agent-logs/`:

| Subdirectory | Component |
|--------------|-----------|
| `daemon/` | Core daemon (`lnxdrive-engine/`) |
| `gnome/` | GNOME integration (`lnxdrive-gnome/`) |
| `guide/` | Documentation & design (`lnxdrive-guide/`) |
| `gtk3/` | GTK3 UI (`experimental/lnxdrive-gtk3/`) — reactivates in v1.0.0 |
| `plasma/` | KDE Plasma (`experimental/lnxdrive-plasma/`) — reactivates in v1.0.0 |
| `cosmic/` | COSMIC UI (`experimental/lnxdrive-cosmic/`) — reactivates in v1.0.0 |
| `packaging/` | Distribution (`lnxdrive-packaging/`) — create when needed |
| `testing/` | Testing infra (`lnxdrive-testing/`) — create when needed |

**When creating an AILOG**, place it in the subdirectory matching the component you worked on.
Use `straymark new ailog` to scaffold; pass `--path .straymark/07-ai-audit/agent-logs/<component>/`
if the CLI prompts for a location.

### Tooling

- `straymark status` — current docs counters and structure check
- `straymark validate` — schema/frontmatter validation
- `straymark new <type>` — scaffold a new doc (AILOG, AIDEC, ADR, ETH, …)
- `straymark analyze <files>` — cognitive/cyclomatic complexity (used by the pre-commit checklist in `STRAYMARK.md`)

---

## 3. Git Operations

> **CRITICAL: Never commit directly to `main`.** All changes go through feature/fix branches and Pull Requests.

| Branch Prefix | Purpose |
|---------------|---------|
| `feature/` or `feat/` | New features |
| `fix/` | Bug fixes |
| `hotfix/` | Urgent production fixes |
| `docs/` | Documentation only |
| `refactor/` | Code refactoring |
| `test/` | Test changes |
| `chore/` | Maintenance, tooling, framework upgrades |

**Conventional Commits:** `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`.

> **Full details:** `.straymark/00-governance/GIT-BRANCHING-STRATEGY.md`

---

## 4. LNXDrive Design Guide Reference

> Comprehensive design and development guide at: `lnxdrive-guide/`

### Navigation by Task Type

| Task | Documents to Load |
|------|-------------------|
| **Understand the project** | `lnxdrive-guide/01-Vision/01-resumen-ejecutivo.md` |
| **Architecture** | `lnxdrive-guide/03-Arquitectura/01-arquitectura-hexagonal.md` |
| **Implement Core/Daemon** | `lnxdrive-guide/04-Componentes/07-motor-sincronizacion.md` |
| **Implement FUSE** | `lnxdrive-guide/04-Componentes/01-files-on-demand-fuse.md` |
| **Implement GNOME UI** | `lnxdrive-guide/04-Componentes/02-ui-gnome.md` |
| **Implement KDE UI** | `lnxdrive-guide/04-Componentes/03-ui-kde-plasma.md` |
| **Implement GTK3 UI** | `lnxdrive-guide/04-Componentes/04-ui-gtk3.md` |
| **Implement COSMIC UI** | `lnxdrive-guide/04-Componentes/05-ui-cosmic.md` |
| **Implement CLI** | `lnxdrive-guide/04-Componentes/06-cli.md` |
| **Add cloud provider** | `lnxdrive-guide/07-Extensibilidad/02-puerto-icloudprovider.md` |
| **Write tests** | `lnxdrive-guide/06-Testing/01-estrategia-testing.md` |
| **Check roadmap** | `lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md` |

**Main index**: `lnxdrive-guide/Guía-de-diseño-y-desarrollo.md`

---

## 5. Context7 — Up-to-date Documentation Lookup

This project has access to the **Context7 MCP server** for real-time documentation.

| Situation | Action |
|-----------|--------|
| Implementing with a library | Query Context7 for current API and examples |
| Integrating an external API | Query Context7 for latest endpoints |
| Upgrading a dependency | Query Context7 for breaking changes |

**How to Use:**
1. `resolve-library-id` with the library name
2. `query-docs` with the resolved ID and specific question

> Prefer Context7 documentation over training data for fast-moving libraries.

---

*StrayMark | LNXDrive Monorepo*
*[Strange Days Tech](https://strangedays.tech) — Because every change tells a story.*
