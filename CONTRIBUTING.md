# Contributing to LNXDrive

Thank you for your interest in contributing to LNXDrive! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Contributor License Agreement (CLA)](#contributor-license-agreement-cla)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)

---

## Code of Conduct

This project is governed by our [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

In short: be respectful, inclusive, and constructive in all interactions. Harassment, discrimination, and trolling are not tolerated. Please read the [full Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

---

## Contributor License Agreement (CLA)

This project requires all contributors to sign a **Contributor License Agreement (CLA)** before their pull requests can be merged. We use [CLA Assistant](https://cla-assistant.io/) to manage this process.

### How it works

1. When you open your first pull request, CLA Assistant will automatically post a comment asking you to sign the CLA.
2. Click the link in the comment to review and sign the agreement.
3. The CLA only needs to be signed once -- it covers all future contributions to this project.
4. Once signed, CLA Assistant will update the PR check status and your contribution can proceed to review.

If you have questions about the CLA, please open a [Discussion](https://github.com/StrangeDaysTech/lnxdrive/discussions).

---

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

**When reporting a bug, include:**

- A clear, descriptive title
- Steps to reproduce the behavior
- Expected behavior vs. actual behavior
- Screenshots or logs (if applicable)
- Environment details (distribution, desktop environment, kernel version)

### Suggesting Features

Feature suggestions are welcome! Please include:

- A clear description of the feature
- The problem it solves
- Potential implementation approaches
- Any alternatives you've considered

### Improving Documentation

Documentation improvements are highly valued:

- Fix typos or unclear wording
- Add examples
- Improve explanations
- Translate to other languages

### Submitting Code

Code contributions should:

- Fix a bug or implement a feature
- Include appropriate tests (if applicable)
- Follow the project's style guidelines
- Update documentation as needed

---

## Development Setup

### Prerequisites

- **Git**
- **Rust 1.75+** (install via [rustup.rs](https://rustup.rs/))
- **Meson & Ninja** (for GNOME integration builds)
- **GJS** (for GNOME Shell extension development)
- **GTK 4 development libraries** (for GNOME UI)
- **D-Bus development libraries** (for IPC)
- **FUSE 3 development libraries** (for files-on-demand)
- **SQLite 3** (for local state storage)
- **Podman** (optional, for container-based testing)

### Setup Steps

1. **Fork the repository**

   Click "Fork" on the [GitHub repository page](https://github.com/StrangeDaysTech/lnxdrive).

2. **Clone your fork**
   ```bash
   git clone https://github.com/your-username/lnxdrive.git
   cd lnxdrive
   ```

3. **Build the engine**
   ```bash
   cd lnxdrive-engine
   cargo build
   cargo test
   ```

4. **Build the GNOME integration** (optional)
   ```bash
   cd lnxdrive-gnome
   meson setup builddir
   meson compile -C builddir
   ```

5. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

---

## Project Structure

LNXDrive is a monorepo with the following components:

```
lnxdrive/
├── lnxdrive-engine/       # Core daemon + library crates (Rust, Cargo workspace)
│   └── crates/
│       ├── lnxdrive-core/       # Domain logic and ports
│       ├── lnxdrive-daemon/     # systemd service
│       ├── lnxdrive-cli/        # Command-line interface
│       ├── lnxdrive-ipc/        # D-Bus IPC layer
│       ├── lnxdrive-fuse/       # Files-on-demand (FUSE)
│       ├── lnxdrive-sync/       # Sync engine
│       ├── lnxdrive-graph/      # Microsoft Graph adapter
│       ├── lnxdrive-cache/      # SQLite state cache
│       ├── lnxdrive-conflict/   # Conflict resolution
│       ├── lnxdrive-audit/      # Audit trail
│       └── lnxdrive-telemetry/  # Metrics and observability
├── lnxdrive-gnome/        # GNOME Shell/Nautilus/GOA integration
├── lnxdrive-gtk3/         # XFCE/MATE UI (GTK3)
├── lnxdrive-plasma/       # KDE Plasma integration (Qt/C++)
├── lnxdrive-cosmic/       # COSMIC desktop UI (Rust)
├── lnxdrive-packaging/    # Flatpak, AppImage, Debian, AUR
├── lnxdrive-guide/        # Design and development guide
├── lnxdrive-testing/      # Container/VM test infrastructure
└── .straymark/             # Project documentation trail
```

For a deep understanding of the architecture and design decisions, refer to the [Design Guide](lnxdrive-guide/).

---

## Pull Request Process

### Branch Naming

| Prefix | Purpose |
|--------|---------|
| `feature/` or `feat/` | New features |
| `fix/` | Bug fixes |
| `hotfix/` | Urgent production fixes |
| `docs/` | Documentation only |
| `refactor/` | Code refactoring |
| `test/` | Test changes |

> **Important:** Never commit directly to `main`. All changes must go through feature/fix branches and Pull Requests.

### Before Submitting

- [ ] Code compiles without warnings
- [ ] Tests pass (`cargo test` in the relevant crate)
- [ ] Documentation is updated if needed
- [ ] Write a clear PR description

### PR Title Format

Use conventional commit format:

```
type(scope): description

Examples:
feat(sync): add delta sync for Microsoft Graph
fix(fuse): correct placeholder state on hydration
docs(guide): clarify hexagonal architecture layers
refactor(daemon): extract signal handling to module
```

**Types:**
- `feat` -- New feature
- `fix` -- Bug fix
- `docs` -- Documentation changes
- `chore` -- Maintenance tasks
- `refactor` -- Code refactoring
- `test` -- Test additions or fixes

### Review Process

1. A maintainer will review your PR
2. Address any requested changes
3. Once approved, a maintainer will merge

---

## Style Guidelines

### Rust Code

- Follow standard Rust conventions (`rustfmt`, `clippy`)
- Use the workspace dependency versions defined in the root `Cargo.toml`
- Write doc comments for public APIs
- Prefer explicit error handling over `unwrap()`

### GNOME / GJS Code

- Follow GNOME coding conventions
- Use ES modules for GJS extensions

### Markdown / Documentation

- Use ATX-style headers (`#`, `##`, etc.)
- Use fenced code blocks with language identifiers
- Keep lines under 120 characters when practical

### Commit Messages

- Use conventional commits format
- Write the subject line in imperative mood ("add feature", not "added feature")
- Keep the subject under 72 characters

---

## Questions?

If you have questions about contributing:

1. Check existing [Issues](https://github.com/StrangeDaysTech/lnxdrive/issues)
2. Check [Discussions](https://github.com/StrangeDaysTech/lnxdrive/discussions)
3. Open a new Discussion for general questions
4. Open an Issue for specific bugs or features

---

## Recognition

Contributors are recognized in:

- GitHub's contributor graph
- Release notes for significant contributions

Thank you for helping make LNXDrive better!

---

*LNXDrive -- Because your files belong everywhere.*

[Strange Days Tech, S.A.S.](https://strangedays.tech)
