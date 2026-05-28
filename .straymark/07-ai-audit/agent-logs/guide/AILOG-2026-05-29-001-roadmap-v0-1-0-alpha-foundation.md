---
id: AILOG-2026-05-29-001
title: Roadmap to v0.1.0-alpha.1 — governance foundation and scope narrowing
status: accepted
created: 2026-05-29
agent: claude-opus-4-7-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [roadmap, governance, charter, scope, mvp, release-planning]
related: [CHARTER-01-road-to-v0-1-0-alpha-1]
---

# AILOG: Roadmap to v0.1.0-alpha.1 — governance foundation and scope narrowing

## Summary

This AILOG originates a multi-month Charter ("Road to v0.1.0-alpha.1") that
covers the work of turning the existing MVP into a publicly downloadable
alpha release. It also records the scope-narrowing decisions made before
any code was touched: archiving three UI subprojects that are still
skeletons and focusing all alpha effort on the GNOME stack.

The plan that motivated this Charter lives at
`/home/montfort/.claude/plans/hola-estamos-en-el-functional-sunrise.md`
(operator-side, not in repo).

## Context — diagnosis before this AILOG

A three-agent audit on 2026-05-28 produced this state-of-the-MVP picture:

- **lnxdrive-engine** (Rust, 12 crates, 759 tests): ~70% MVP-ready. FUSE,
  Microsoft Graph, delta sync, D-Bus IPC and file watching are
  implemented; ~4 files contain `todo!()/unimplemented!()` and ~10
  residual debug `println!` calls.
- **lnxdrive-gnome**: ~45%. Shell extension and Nautilus integration
  operational; the GTK4 preferences panel (`src/main.rs`) is a `println!`
  stub. GOA provider just landed in PR #2.
- **lnxdrive-gtk3 / lnxdrive-plasma / lnxdrive-cosmic**: 5–10% — Cargo /
  CMake skeletons with `not yet implemented` stubs.
- **lnxdrive-packaging**: Flatpak manifest is partial (no
  metainfo/desktop/icon install stages); RPM, DEB, AUR, AppImage are
  named in `lnxdrive-packaging/README.md` but do not exist.
- **lnxdrive-testing**: ~80% — Podman + QEMU + nested-GNOME infra works.
- **SpecKit**: `001-core-cli` 254/254 ✅; `002-files-on-demand` 105/106
  (99.1%, 1 task pending).
- **CI**: `lnxdrive-engine/.github/workflows/ci.yml` runs `fmt + clippy
  + build` but **does not run `cargo test`** and no release automation
  exists. Zero GitHub Releases, zero tags, zero milestones, zero public
  issues, zero UI screenshots.
- **`SECURITY.md`**: absent. `lnxdrive.spdx` describes StrayMark, not
  LNXDrive (a packaging copy/paste bug to fix during release prep).
- **StrayMark risks**: four P0 entries documented in
  `.straymark/02-design/risk-analysis/` with **no remediation AILOGs**:
    - `RISK-002` — OAuth tokens travel in cleartext over D-Bus
      (CVSS 9.1). Inspecting the session bus with `dbus-monitor` would
      expose `Bearer …` strings. Hard blocker for any public release.
    - `RISK-003` — write-during-hydration race in the FUSE layer can
      corrupt files; `crates/lnxdrive-fuse/src/write_serializer.rs`
      exists as a stub.
    - `RISK-001` — D-Bus session bus is a single point of failure for
      the daemon ↔ UI link.
    - `ISSUE-002` — config YAML parser is susceptible to a
      billion-laughs DoS.

## Decisions taken with the human operator (2026-05-29)

These decisions were taken before any code change. They define the
boundaries of the Charter that this AILOG originates.

1. **Target = v0.1.0 alpha for early adopters**, not v1.0 mass-market.
   Audience is Linux power users and GNOME enthusiasts willing to
   report bugs. Estimated 4–6 weeks (5–7 calendar weeks with margin).
2. **GNOME-only in scope.** `lnxdrive-gtk3`, `lnxdrive-plasma` and
   `lnxdrive-cosmic` move to `experimental/` with a README that marks
   them as post-1.0 placeholders. Not built in CI. Not referenced from
   the alpha packaging manifest or README's install matrix.
3. **P0 risks block the release.** No artifact ships until RISK-001,
   RISK-002, RISK-003 and ISSUE-002 have remediation AILOGs (and ETHs
   for the credential-handling one) plus regression tests. RISK-002 is
   the hardest constraint: tokens must move to `secret-service`
   (libsecret) via `keyring-rs`; the D-Bus interface must expose only
   opaque `SessionHandle` values, never raw tokens.
4. **Public-facing work tracking.** The 27 entries of
   `.straymark/02-design/risk-analysis/BACKLOG-simulation-issues.md`
   plus the four P0 items become GitHub issues with labels (P0–P3,
   `security`, `architecture`, …) and milestones (`v0.1.0-alpha`,
   `v0.2.0-beta`, `v1.0.0`), grouped under a `Road to v0.1.0`
   GitHub project board.

## Plan phases (Charter-level outline)

The Charter that follows this AILOG declares these phases as scope. The
expected effort estimate is **L** (large, multi-week, multi-batch).

- **Fase 0 — Governance foundation (this PR + setup, 3–5 days)**: this
  AILOG, the Charter, archival of non-MVP UIs, README/packaging updates,
  GitHub milestones + project board, bulk-conversion of backlog to
  issues.
- **Fase 1 — Security hardening (1.5–2 weeks)**: RISK-002 → keyring +
  D-Bus session handles; RISK-003 → write serializer in the FUSE layer;
  RISK-001 → D-Bus health monitor (full fallback deferred to v0.2);
  ISSUE-002 → YAML hardening; `cargo audit` + `cargo deny` in CI.
- **Fase 2 — Engine polish (1 week)**: close `002-files-on-demand`
  task #106; remove residual `todo!()`/`unimplemented!()` and debug
  `println!` calls; enable `cargo test --workspace` in CI.
- **Fase 3 — GTK4 preferences panel (1 week)**: implement the four
  basic settings groups (Account, Folders, Network, System) backed by
  the existing D-Bus daemon API.
- **Fase 4 — Flatpak packaging (1 week)**: complete the manifest with
  install stages for icons, `.desktop`, metainfo XML; fix
  `lnxdrive.spdx` (currently describes StrayMark, not LNXDrive); local
  smoke test via `flatpak-builder`.
- **Fase 5 — Release infrastructure (3–5 days)**: `release.yml` workflow
  (tag → bundle → GitHub Release); `SECURITY.md`; `CHANGELOG.md`;
  6 UI screenshots in `docs/screenshots/`; version unification to
  `0.1.0-alpha.1` across `Cargo.toml`, Flatpak manifest, metainfo XML;
  README install section + competitive comparison table.
- **Fase 6 — Tag, release, announce (1–2 days)**: signed git tag
  `v0.1.0-alpha.1`, GitHub Pre-release with Flatpak bundle and
  SHA256SUMS, announcement on r/linux, r/gnome, r/onedrive and
  StrangeDaysTech Mastodon. Charter closes with telemetry.

## Out of scope (recorded ex-ante so the drift gate ignores them)

- GTK4 preferences panel beyond the four basic groups → v0.2.
- Plasma, GTK3, COSMIC UIs → archived in `experimental/`, milestone
  `v1.0.0`.
- RPM, DEB, AUR, AppImage → milestone `v0.2.0-beta`.
- Flathub submission → milestone `v0.2.0-beta`.
- i18n / translations → milestone `v0.2.0-beta` (structure) and
  `v1.0.0` (5+ languages).
- Telemetry / crash reporting → milestone `v0.2.0-beta`.
- Landing page on strangedays.tech → milestone `v0.2.0-beta`.
- D-Bus full fallback (Unix socket) → milestone `v0.2.0-beta`. The
  alpha ships only the health monitor.

## This PR (governance foundation)

This commit performs the parts of Fase 0 that don't require external
state (GitHub milestones, project boards, bulk issue creation):

1. This AILOG.
2. `straymark charter new --from-ailog AILOG-2026-05-29-001
   --title "Road to v0.1.0-alpha.1" -t L`, with success criteria edited
   in to mirror the 9-point verification list of the plan.
3. `git mv lnxdrive-{gtk3,plasma,cosmic} experimental/` so blame on
   their Cargo / CMake skeletons survives.
4. `experimental/README.md` explaining why they are there and what
   reactivates them.
5. README, CLAUDE.md, GEMINI.md, ayuda.md updates removing the archived
   UIs from the monorepo matrix; same for any subproject manifest that
   mentions them.

The remaining Fase 0 actions (milestones, project board, bulk-converting
27+4 backlog entries to GitHub issues) land in a follow-up PR because
they're external-state changes that benefit from being reviewed in
isolation.

## Verification (this PR only)

- `straymark status` → 17/17 items present, AILOG count rises by 1,
  Charters count rises from 0 to 1.
- `straymark charter status` → the new Charter appears in `declared`
  state with effort `L`, origin `AILOG-2026-05-29-001`.
- `git log --follow experimental/lnxdrive-gtk3/Cargo.toml` returns the
  full pre-archival history.
- `grep -rn "lnxdrive-gtk3\|lnxdrive-plasma\|lnxdrive-cosmic" --include="*.md" --include="*.yaml"`
  outside `experimental/` and `.straymark/07-ai-audit/agent-logs/`
  returns either zero matches or explicit "archived in experimental/"
  notes.
- The 9-point end-to-end checklist of the plan stays untouched — this
  PR only opens the door for Fase 1.
