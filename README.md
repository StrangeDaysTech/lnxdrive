<p align="center">
  <img src="lnxdrive-gnome/data/icons/hicolor/scalable/apps/com.strangedaystech.LNXDrive.svg" alt="LNXDrive" width="128" height="128">
</p>

<h1 align="center">LNXDrive</h1>

<p align="center">
  <strong>Cloud storage sync for Linux -- native, explainable, and built for every desktop.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg" alt="License"></a>
  <a href="https://github.com/StrangeDaysTech/lnxdrive"><img src="https://img.shields.io/badge/status-early%20development-orange.svg" alt="Status"></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust"></a>
</p>

---

## What is LNXDrive?

LNXDrive is a cloud storage synchronization client for Linux designed from scratch with clean, hexagonal architecture. Unlike existing solutions that simply sync files in the background, LNXDrive is an **explainable, governable, and adaptable** sync system.

**Key differentiators:**

- **Explainable sync** -- You always know *why* something failed or didn't sync. No more cryptic "sync error" messages.
- **Files-on-Demand** -- A robust FUSE-based virtual filesystem with clear file states (online-only, locally available, always-keep). See your cloud files without downloading them.
- **Native desktop integration** -- Purpose-built UI for GNOME, KDE Plasma, COSMIC, and GTK3-based desktops (XFCE, MATE). Not a generic tray icon -- real shell extensions, file manager overlays, and system settings panels.
- **Multi-provider, multi-account** -- Connect OneDrive, Google Drive, Dropbox, and more. Unlimited account namespaces (`onedrive:work`, `gdrive:personal`).
- **Declarative configuration** -- Versionable YAML config, not scattered dotfiles and CLI flags.
- **Full observability** -- Structured JSON logs, Prometheus metrics, and a complete audit trail.

## How it works

LNXDrive runs as a user-level **systemd service** (`lnxdrive-daemon`) that manages all sync operations. It communicates with desktop integrations through **D-Bus** (`com.strangedaystech.LNXDrive`), and exposes a **FUSE filesystem** for files-on-demand.

```
Cloud Providers          LNXDrive Engine              Desktop
  (OneDrive,        ┌─────────────────────┐      ┌──────────────┐
   GDrive,          │  lnxdrive-daemon    │      │ GNOME Shell  │
   Dropbox...)  <──>│  ├─ sync engine     │<────>│ KDE Plasma   │
                    │  ├─ FUSE mount      │ D-Bus│ COSMIC       │
                    │  ├─ conflict mgr    │      │ GTK3 (XFCE)  │
                    │  └─ audit + metrics │      │ CLI           │
                    └─────────────────────┘      └──────────────┘
```

---

## Getting Started

> LNXDrive is in **early development**. The instructions below describe the intended installation flow. Pre-built packages are not yet available.

### Requirements

- Linux with kernel 5.15+ and FUSE 3 support
- systemd (user session)
- A supported desktop environment (GNOME 45+, KDE Plasma 6, COSMIC, XFCE 4.18+, or MATE 1.26+)

### Installation

Pre-built packages will be available for:

| Format | Desktop | Status |
|--------|---------|--------|
| Flatpak | All | Planned |
| AppImage | All | Planned |
| `.deb` | Debian/Ubuntu | Planned |
| AUR | Arch Linux | Planned |

### First steps

1. **Install LNXDrive** using your preferred package format
2. **Launch the setup wizard** from your application menu or run `lnxdrive setup`
3. **Add a cloud account** -- the wizard will guide you through OAuth authentication
4. **Choose your sync mode** -- select which folders to sync and whether to use files-on-demand
5. **Start syncing** -- LNXDrive runs automatically as a systemd user service

### CLI quick start

```bash
# Check daemon status
lnxdrive status

# Add a OneDrive account
lnxdrive account add onedrive:work

# List synced files
lnxdrive ls /

# Force sync a specific path
lnxdrive sync ~/OneDrive/Documents

# View sync activity
lnxdrive log --follow
```

---

## For Developers

### Project Structure

This is a **monorepo** containing all LNXDrive components:

| Directory | Description | Stack |
|-----------|-------------|-------|
| [`lnxdrive-engine/`](lnxdrive-engine/) | Core daemon and library crates | Rust 1.75+, tokio, zbus, sqlx |
| [`lnxdrive-gnome/`](lnxdrive-gnome/) | GNOME Shell, Nautilus, and GOA integration | Meson + Rust (gtk4-rs), GJS, C |
| [`experimental/lnxdrive-gtk3/`](experimental/lnxdrive-gtk3/) | XFCE/MATE desktop UI — archived for v0.1.0-alpha (reactivates in v1.0.0) | Rust, GTK3 |
| [`experimental/lnxdrive-plasma/`](experimental/lnxdrive-plasma/) | KDE Plasma integration — archived for v0.1.0-alpha (reactivates in v1.0.0) | C++, CMake, Qt/KDE |
| [`experimental/lnxdrive-cosmic/`](experimental/lnxdrive-cosmic/) | COSMIC desktop UI — archived for v0.1.0-alpha (reactivates in v1.0.0) | Rust, libcosmic |
| [`lnxdrive-packaging/`](lnxdrive-packaging/) | Distribution packages | Flatpak, AppImage, Debian, AUR |
| [`lnxdrive-guide/`](lnxdrive-guide/) | Design and development guide | Markdown |
| [`lnxdrive-testing/`](lnxdrive-testing/) | Container/VM test infrastructure | Podman, QEMU/libvirt |

### Architecture

The engine follows a **hexagonal (ports & adapters) architecture**:

- **Domain core** (`lnxdrive-core`) defines sync logic, conflict resolution rules, and provider-agnostic interfaces
- **Adapters** implement cloud provider APIs (Microsoft Graph, etc.), storage backends (SQLite), and IPC (D-Bus)
- **Desktop UIs** are fully interchangeable -- each desktop environment has its own native adapter

For a comprehensive deep-dive, see the [Design and Development Guide](lnxdrive-guide/).

### Building from source

```bash
# Clone the repository
git clone https://github.com/StrangeDaysTech/lnxdrive.git
cd lnxdrive

# Build the engine
cd lnxdrive-engine
cargo build
cargo test

# Build GNOME integration (requires meson, gtk4-devel, etc.)
cd ../lnxdrive-gnome
meson setup builddir
meson compile -C builddir
```

### Development workflow

1. **Read the [Contributing Guide](CONTRIBUTING.md)** -- it covers branching, commit conventions, and the review process
2. **Read the [Code of Conduct](CODE_OF_CONDUCT.md)** -- participation requires respectful, inclusive behavior
3. **Never commit directly to `main`** -- use feature branches and pull requests
4. **Follow conventional commits** -- `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`
5. **Sign the CLA** -- required on your first pull request

### Documentation trail

This project uses [StrayMark](https://github.com/StrangeDaysTech/straymark) to maintain a complete documentation trail of architectural decisions, AI-assisted changes, and technical debt. See `.straymark/` for the full audit history.

---

## Roadmap

LNXDrive development is organized in phases:

| Phase | Milestone | Status |
|-------|-----------|--------|
| 0 | Testing infrastructure (containers, CI/CD, mock servers) | In progress |
| 1 | Core engine + CLI (sync, delta, rate limiting, systemd service) | Planned |
| 2 | Files-on-Demand (FUSE, placeholders, hydration) | Planned |
| 3 | GNOME integration (Shell extension, Nautilus overlays, GOA) | Planned |
| 4 | Conflict resolution UI and declarative policies | Planned |
| 5 | KDE Plasma integration | Planned |
| 6 | Multi-provider support (Google Drive, Dropbox) | Planned |
| 7 | COSMIC desktop integration | Planned |
| 8 | GTK3 integration (XFCE/MATE) | Planned |
| 9 | Packaging and distribution | Planned |
| 10 | Observability and advanced features | Planned |

For the full roadmap with detailed deliverables, see [the roadmap document](lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md).

---

## Contributing

We welcome contributions of all kinds -- code, documentation, testing, translations, and ideas.

Please read our **[Contributing Guide](CONTRIBUTING.md)** and **[Code of Conduct](CODE_OF_CONDUCT.md)** before getting started. All contributors must sign a Contributor License Agreement (CLA) on their first pull request.

---

## License

LNXDrive is free software licensed under the **[GNU General Public License v3.0 or later](LICENSE)**.

The [Design and Development Guide](lnxdrive-guide/) is licensed separately under the [MIT License](lnxdrive-guide/LICENSE).

---

<p align="center">
  <a href="https://strangedays.tech"><strong>Strange Days Tech, S.A.S.</strong></a><br>
  Because your files belong everywhere.
</p>
