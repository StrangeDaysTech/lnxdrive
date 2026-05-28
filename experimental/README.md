# `experimental/` — UIs deferred past v0.1.0-alpha.1

These subprojects were scoped out of the **v0.1.0-alpha.1** release so the team
could focus all effort on the GNOME stack (Shell extension + Nautilus + GOA +
GTK4 preferences). They live here as project skeletons — Cargo / CMake
manifests, entry-point stubs, GitHub copilot instructions — preserved for
future contributors and a clear historical record that they were always
part of the original product vision.

| Subproject | Tech stack | Target desktop | State as of archival | Milestone reactivation |
|------------|------------|----------------|----------------------|------------------------|
| `lnxdrive-gtk3/` | Rust + GTK3 | XFCE, MATE | `src/main.rs` is a `println!("not yet implemented")` stub | `v1.0.0` |
| `lnxdrive-plasma/` | C++ + Qt6 + KDE Frameworks | KDE Plasma 6 | `main.cpp` boots `KApplication`; QML engine `TODO`. `plasmoid/` and `dolphin-plugin/` directories empty | `v1.0.0` |
| `lnxdrive-cosmic/` | Rust + `libcosmic` | System76 COSMIC | `src/main.rs` is a `println!("not yet implemented")` stub | `v1.0.0` |

## Why not in the alpha?

- **Engineering economy** — every additional UI multiplies the surface area for
  D-Bus integration tests, packaging metadata, screenshots, and bug triage.
  Three skeleton UIs at ~5–10% completion would have absorbed weeks of
  alpha-blocker time without delivering anything users could actually run.
- **Audience signal** — the alpha targets Linux/GNOME early adopters. KDE,
  XFCE/MATE, and COSMIC users are first-class citizens of the long-term
  roadmap, but reaching them with an unfinished UI would be worse than
  reaching them later with a polished one.
- **D-Bus contract stabilization** — the GNOME UI is currently the only
  consumer of the daemon's D-Bus interface. Stabilizing that contract
  against a single, well-exercised client before opening it to three more
  is sound engineering practice; cross-UI portability work belongs in
  `v1.0.0` once the contract has shipped to real users.

## How they will come back

The `v1.0.0` milestone on GitHub tracks the work to reactivate them. The
expected sequence:

1. After `v0.1.0-alpha.1` ships, the D-Bus interface in
   `lnxdrive-engine/crates/lnxdrive-daemon/` gets a written contract document
   in `lnxdrive-guide/04-Componentes/` (currently implicit in the GNOME
   integration).
2. The contract becomes the reference for porting. The first UI to leave
   `experimental/` is most likely Plasma (largest user base after GNOME on
   non-Ubuntu distributions); COSMIC follows alongside System76's first
   COSMIC stable release; GTK3 (XFCE/MATE) closes the set.
3. Each reactivation is its own StrayMark Charter referencing this README
   for the historical context.

## Contributing here

Pull requests against `experimental/` subprojects are welcome **but**:

- They are **not built or tested by CI** for the alpha cycle (workflows in
  `lnxdrive-engine/.github/workflows/ci.yml` and the root
  `.github/workflows/docs-validation.yml` deliberately ignore this
  directory). Local-only build verification is on the contributor.
- They cannot land changes that require modifying the D-Bus contract in
  `lnxdrive-engine/` — that contract is owned by the alpha cycle. Coordinate
  on the issue tracker before opening a PR that needs new daemon APIs.
- Cargo/CMake manifests here are not part of the workspace root build;
  treat each subdirectory as an independent project for now.

## Provenance

Archived in PR
[`chore/governance-foundation-v0.1.0-alpha`](https://github.com/StrangeDaysTech/lnxdrive)
on 2026-05-29 per the scope decisions recorded in
`.straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md`
and the StrayMark Charter
`.straymark/charters/01-road-to-v0-1-0-alpha-1.md`.

`git log --follow experimental/lnxdrive-{gtk3,plasma,cosmic}/<any-file>`
returns the full pre-archival history.
