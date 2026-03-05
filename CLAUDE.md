# LNXDrive Monorepo — Agent Instructions

> **This file is automatically loaded at the start of each session.**
> It contains project navigation, DevTrail rules, and tool integration guidelines.

---

## 1. Project Structure (Monorepo)

This is a monorepo containing all LNXDrive components:

| Directory | Description | Tech Stack |
|-----------|-------------|------------|
| `lnxdrive/` | Core daemon + library crates | Rust 1.75+, Cargo workspace (12 crates), tokio, zbus, sqlx |
| `lnxdrive-gnome/` | GNOME Shell/Nautilus/GOA integration | Meson + Rust (gtk4-rs), GJS (ES modules), C11 (Nautilus), Python |
| `lnxdrive-gtk3/` | XFCE/MATE UI | Rust, Cargo, GTK3 |
| `lnxdrive-plasma/` | KDE Plasma integration | C++, CMake, Qt/KDE |
| `lnxdrive-cosmic/` | COSMIC desktop UI | Rust, Cargo |
| `lnxdrive-packaging/` | Distribution packages | Flatpak, AppImage, Debian, AUR |
| `lnxdrive-guide/` | Design & development guide | Markdown (Spanish) |
| `lnxdrive-testing/` | Container/VM test infrastructure | Podman, QEMU/libvirt, shell scripts |

### Key Paths
- **Design guide index**: `lnxdrive-guide/Guía-de-diseño-y-desarrollo.md`
- **DevTrail root**: `.devtrail/` (single instance for entire monorepo)
- **D-Bus name**: `com.strangedaystech.LNXDrive` on `/com/strangedaystech/LNXDrive`

---

## 2. DevTrail — Documentation Guidelines

### Fundamental Principle

> **"No significant change without a documented trace."**

### Language Configuration

Check `.devtrail/config.yml` for the project's language setting:

```yaml
language: en  # Options: en, es (default: en)
```

### Your Identity as an Agent

- **Identify yourself** as: `claude-code-v1.0` (or your version)
- **Declare** your confidence level: `high | medium | low`
- **Record** identification in the `agent:` field of metadata

### Documentation Reporting

At the end of each task, report DevTrail status:

```
DevTrail: Created AILOG-2026-03-04-001-implement-feature.md
DevTrail: No documentation required (minor change / <10 lines)
DevTrail: Documentation pending - use /devtrail-status to review
```

### When to Document

| Situation | Action |
|-----------|--------|
| >10 lines of code in business logic | Create AILOG |
| Decision between technical alternatives | Create AIDEC |
| Changes in security/authentication | Create AILOG + mark `risk_level: high` |
| Personal data (GDPR/PII) | Create AILOG + request ETH |
| Integration with external service | Create AILOG |
| Change in public API or DB schema | Create AILOG |

**DO NOT DOCUMENT:** Trivial changes (whitespace, typos, formatting), sensitive information.

### File Naming

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

### Agent-Logs Organization (Monorepo)

AILOGs are organized by component in `.devtrail/07-ai-audit/agent-logs/`:

| Subdirectory | Component |
|--------------|-----------|
| `daemon/` | Core daemon (lnxdrive/) |
| `gnome/` | GNOME integration (lnxdrive-gnome/) |
| `guide/` | Documentation & design (lnxdrive-guide/) |
| `gtk3/` | GTK3 UI (lnxdrive-gtk3/) — create when needed |
| `plasma/` | KDE Plasma (lnxdrive-plasma/) — create when needed |
| `cosmic/` | COSMIC UI (lnxdrive-cosmic/) — create when needed |
| `packaging/` | Distribution (lnxdrive-packaging/) — create when needed |
| `testing/` | Testing infra (lnxdrive-testing/) — create when needed |

**When creating an AILOG**, place it in the subdirectory matching the component you worked on.

### Minimum Metadata

```yaml
---
id: AILOG-2026-03-04-001
title: Brief description
status: accepted
created: 2026-03-04
agent: claude-code-v1.0
confidence: high | medium | low
review_required: true | false
risk_level: low | medium | high | critical
---
```

### Autonomy Limits

| Type | I can do | Requires human |
|------|----------|----------------|
| **AILOG** | Create freely | - |
| **AIDEC** | Create freely | - |
| **ETH** | Create draft | Approval |
| **ADR** | Create draft | Review |
| **REQ** | Propose | Validation |
| **INC** | Contribute analysis | Conclusions |
| **TDE** | Identify | Prioritize |

### Quick Type Reference

| Prefix | Name | Location |
|--------|------|----------|
| `AILOG` | AI Action Log | `.devtrail/07-ai-audit/agent-logs/<component>/` |
| `AIDEC` | AI Decision | `.devtrail/07-ai-audit/decisions/` |
| `ETH` | Ethical Review | `.devtrail/07-ai-audit/ethical-reviews/` |
| `ADR` | Architecture Decision Record | `.devtrail/02-design/decisions/` |
| `REQ` | Requirement | `.devtrail/01-requirements/` |
| `TES` | Test Plan | `.devtrail/04-testing/` |
| `INC` | Incident Post-mortem | `.devtrail/05-operations/incidents/` |
| `TDE` | Technical Debt | `.devtrail/06-evolution/technical-debt/` |

### When to Load Templates

| Situation | Document to load |
|-----------|------------------|
| Going to create an AILOG | `.devtrail/templates/TEMPLATE-AILOG.md` |
| Going to create an AIDEC | `.devtrail/templates/TEMPLATE-AIDEC.md` |
| Going to create an ADR | `.devtrail/templates/TEMPLATE-ADR.md` |
| Going to create a REQ | `.devtrail/templates/TEMPLATE-REQ.md` |
| Questions about naming | `.devtrail/00-governance/DOCUMENTATION-POLICY.md` |
| Questions about autonomy | `.devtrail/00-governance/AGENT-RULES.md` |

---

## 3. Git Operations

> **CRITICAL: Never commit directly to `main` branch.**

All changes must go through feature/fix branches and Pull Requests.

| Branch Prefix | Purpose |
|---------------|---------|
| `feature/` or `feat/` | New features |
| `fix/` | Bug fixes |
| `hotfix/` | Urgent production fixes |
| `docs/` | Documentation only |
| `refactor/` | Code refactoring |
| `test/` | Test changes |

**Conventional Commits:** `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`

> **Full details:** `.devtrail/00-governance/GIT-BRANCHING-STRATEGY.md`

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

*DevTrail v1.0.0 | LNXDrive Monorepo*
*[Strange Days Tech](https://strangedays.tech) — Because every change tells a story.*
