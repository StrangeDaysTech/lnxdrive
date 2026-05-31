<!--
StrayMark unified audit prompt — v1 (EN canonical).

This file is a TEMPLATE. `straymark charter audit <CHARTER-ID>` resolves the
placeholders below against the Charter's content + git range + originating
AILOGs, and writes the resolved prompt to:

    .straymark/audits/<CHARTER-ID>/audit-prompt.md

The resolved prompt is what each external auditor reads. The auditor saves
its report to a canonical location keyed on its model identifier so that the
review skill can iterate over N reports (one per auditor) — see CLI-REFERENCE
for the canonical naming.

Localization: the CLI uses `.straymark/config.yml`'s `language` field to pick
the right template. When `language: es`, the template at
`.straymark/audit-prompts/i18n/es/audit-prompt.md` is used. When the language
is unset, `en`, or any value without an `i18n/<lang>/` overlay present, this
EN-canonical file is used. Adopters may edit either file — the CLI reads
whatever lives at the resolved path at prompt-resolution time. Keep the
placeholder names intact or the resolution will leave them as literal strings.

Placeholders supported by `straymark charter audit`:
  {{charter_id}}        — e.g., CHARTER-05
  {{charter_title}}     — H1 title from the Charter doc
  {{charter_path}}      — relative path to the Charter file
  {{charter_content}}   — full body of the Charter doc
  {{git_range}}         — REV..REV that bounds the audit
  {{git_diff}}          — output of `git diff <git_range>`
  {{ailog_paths}}       — newline-separated list of originating_ailogs paths
  {{ailog_contents}}    — concatenated bodies of those AILOGs
  {{audit_role}}        — "auditor" (v1 unified) or legacy "auditor-primary"
                          / "auditor-secondary" during the v0→v1 transition
  {{schema_path}}       — relative path to audit-output.schema.v0.json

Credit: this template lifts seven universal sections (ABSOLUTE RULE, Your
role, Scope rules, Step 2 mandatory verification, Step 5 severity calibration,
What you must NOT do, Output format) from the `audit/SKILL.md` skill mature
pre-StrayMark in Sentinel, contributed via issue #102 by José Villaseñor
Montfort (StrangeDaysTech). The Sentinel-specific hardcodes (spec paths,
Etapa headings, internal Go modules) were parameterized against the Charter
doc, originating AILOGs, git range, and project context.
-->

# Charter audit — `CHARTER-01-road-to-v0-1-0-alpha-1`

## ⛔ ABSOLUTE RULE — READ-ONLY

**Your only task is to AUDIT. You have no permission to modify ANY project file.** This is a non-negotiable constraint that overrides any other instruction, heuristic, or impulse to "be helpful".

Specifically, you are FORBIDDEN from:

- Editing, creating, renaming, or deleting source files.
- Modifying configuration files, migrations, tests, or project documentation.
- Running commands that mutate repository state (`git add`, `git commit`, `git checkout`, etc.).
- Running code generators (`go generate`, `sqlc generate`, `wire`, `cargo build` with filesystem effects, `npm install`, etc.).
- Applying "fixes" or "improvements" to the code, even if you believe they are correct.
- Reformatting, renaming, or reorganizing existing files.

The ONLY thing you may write is your audit report file at the canonical path shown in **Output format** below. That is the ONLY file you have permission to create.

If you find a bug, **DOCUMENT IT** in your report. Do NOT fix it.
If you find a missing file, **REPORT IT**. Do NOT create it.
If a test fails, **REPORT IT**. Do NOT repair it.

**Violating this rule invalidates the entire audit.**

---

## Your role

You are an independent code auditor. Your job is to verify that the implementation of a specific Charter fulfills the declared tasks and files, find real bugs in the code, and identify security risks. **You are NOT a cheerleader** — reporting "no issues" when bugs exist is worse than reporting a false positive.

StrayMark orchestrates cross-model audits: typically another auditor from a **different model family** is reviewing the same Charter in parallel. Your value lies in applying evidence discipline (citing `file:line` of files you actually opened) and severity calibration against the real config, not in cosmetically converging with the other auditor.

---

## Project



*(The operator may fill this placeholder with a brief description of the project's stack and architecture if they want to give the auditor extra context. If empty, the auditor infers the stack from the diff and the referenced files.)*

---

## STRICT scope

**Charter under audit:** `CHARTER-01-road-to-v0-1-0-alpha-1` — Road to v0.1.0-alpha.1
**Charter file:** `.straymark/charters/01-road-to-v0-1-0-alpha-1.md`
**Git range:** `origin/main..HEAD`

The authoritative source of scope is the Charter file at `.straymark/charters/01-road-to-v0-1-0-alpha-1.md`. Read it in full before starting — it declares which files are modified, which tasks are executed, which risks are accepted, and what counts as successful closure.

### Scope rules

- Report only findings that touch **files or tasks declared in the Charter**, or that appear modified in the `git_range`.
- If you find a problem in code that belongs to another Charter (another unit of work), report it as **"Out-of-scope note"** in a separate section, NOT as a defect of this Charter.
- Do NOT report as defects:
  - Modules not yet implemented that are planned for future Charters.
  - Wiring / DI not connected when the wiring task belongs to another Charter.
  - Missing integration tests when the test task belongs to another Charter.
  - Files that do not exist but whose task is marked as `[ ]` (pending) in the Charter.

### Originating AILOGs

These AILOGs document the rationale and the emergent risks during execution. **Read them before auditing** — the `R<N>` risks already documented there are NOT new findings, they are consciously accepted trade-offs.

```
.straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md
```

```markdown
--- AILOG-2026-05-29-001 ---
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


```

---

## Charter content

```markdown
---
charter_id: CHARTER-01-road-to-v0-1-0-alpha-1
status: in-progress
started_at: 2026-05-29
effort_estimate: L
trigger: "MVP audit on 2026-05-28 found engine ~70% / GNOME UI ~45% ready, four P0 risks unmitigated, zero release artifacts. Operator committed scope to v0.1.0 alpha (GNOME-only, P0 risks block release) on 2026-05-29."
originating_ailogs: [AILOG-2026-05-29-001]
---

# Charter: Road to v0.1.0-alpha.1

> **Status (mirrored from frontmatter — source of truth is above):** in-progress (started 2026-05-29). Effort: L (~5–7 calendar weeks).
>
> **Origin:** Follow-up of `AILOG-2026-05-29-001` — full diagnosis of the MVP state, the scope-narrowing decisions taken with the operator, and the phase outline that this Charter formalizes.

## Context

The lnxdrive monorepo finished its MVP implementation (SpecKit features `001-core-cli` 100%, `002-files-on-demand` 99.1%) in February 2026. The engine has 759 tests and the GNOME stack (Shell extension, Nautilus, GOA) is operational, but the project has zero public release artifacts and four P0 risks documented in `.straymark/02-design/risk-analysis/` without remediation AILOGs — most critically `RISK-002` (OAuth tokens in cleartext on D-Bus, CVSS 9.1). The operator audited the state on 2026-05-28 and decided on 2026-05-29 to ship a focused v0.1.0 alpha for Linux/GNOME early adopters rather than continue spreading effort across four UIs. This Charter is the unit of work that turns the existing code into a downloadable Flatpak bundle on GitHub Releases, with security risks closed and a public backlog visible to potential contributors.

## Scope

**In scope:**

1. **Governance foundation** — declare this Charter, archive `lnxdrive-gtk3/`, `lnxdrive-plasma/`, `lnxdrive-cosmic/` under `experimental/`, create `Road to v0.1.0` GitHub project board with milestones `v0.1.0-alpha`, `v0.2.0-beta`, `v1.0.0`, bulk-convert `BACKLOG-simulation-issues.md` (27 entries) + `RISK-001/002/003` into GitHub issues with priority and milestone labels.
2. **Security hardening** — implement remediations for the four P0 entries:
   - `RISK-002`: move OAuth tokens to `secret-service` via `keyring-rs`; refactor `lnxdrive-engine/crates/lnxdrive-daemon/` D-Bus interface so it exposes opaque `SessionHandle` IDs, never raw tokens. AILOG with `risk_level: high` + ETH.
   - `RISK-003`: implement `lnxdrive-engine/crates/lnxdrive-fuse/src/write_serializer.rs` (currently a stub) with per-inode locking during hydration.
   - `RISK-001`: D-Bus health monitor + reconnect in `lnxdrive-daemon`. Full Unix-socket fallback explicitly deferred to v0.2.
   - `ISSUE-002`: harden the YAML config parser against billion-laughs (size + alias caps); regression fixture in `lnxdrive-engine/tests/security/`.
   - `cargo audit` + `cargo deny` jobs in CI.
3. **Engine polish** — close the one remaining task (T101 performance validation) in `lnxdrive-engine/specs/002-files-on-demand/tasks.md`. **Done** (Fase 2): T101 validated via a real-mount integration test — `getattr` 43.7µs, `readdir` 1.40ms/1000 entries, idle RSS 37.9MB/10k files (all under target). The test was the first real FUSE mount exercised in the codebase and surfaced four functional listing bugs (init runtime-context panic, root self-listing, unstable `readdir` order, `opendir` dir-cache) plus an inode-persistence defect, all fixed with regression tests — see AILOG-2026-05-31-001. The other three items this row originally listed (remove `todo!()/unimplemented!()`, remove debug `println!`, enable `cargo test --workspace` in CI) were **already completed during Fase 1** (verified against `main`: zero such sites in crates; `cargo test --workspace` live at `.github/workflows/engine-ci.yml:66`).
4. **GTK4 preferences panel** — the panel already exists under `lnxdrive-gnome/preferences/` (the root `src/main.rs` stub is just a placeholder). Fase 3 audits it (`.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`) and fixes the findings. It ships **three** settings groups wired to the daemon — Account, Folders (Sync), Network (Advanced) — plus Conflicts. The fourth group, **System** (auto-start, cache, dehydration), is **deferred to a v0.2 Charter** because it needs new daemon D-Bus API and is post-alpha (see AIDEC-2026-05-31-001). Key fix: realign the panel with the Fase-1 RISK-002 daemon API (`CompleteAuthViaGOA`).
5. **Flatpak packaging** — complete `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` with install stages (icons, `*.desktop`, metainfo XML), correct permissions (`--filesystem=home:rw`, `--talk-name=org.freedesktop.secrets`), and target `org.gnome.Platform 47`. Fix `lnxdrive.spdx` (currently describes StrayMark by mistake). Complete the metainfo XML with description, releases section, and screenshot URLs.
6. **Release infrastructure & public assets** — `.github/workflows/release.yml` (tag → bundle → GitHub Release with SHA256SUMS); `SECURITY.md`; `CHANGELOG.md`; 6 UI screenshots in `docs/screenshots/`; version `0.1.0-alpha.1` consistent across every `Cargo.toml`, Flatpak manifest, and metainfo XML; README install section + competitive comparison vs `jstaf/onedriver` and `abraunegg/onedrive`.
7. **Tag, release, announce** — signed tag `v0.1.0-alpha.1`, GitHub Pre-release with Flatpak bundle, posts on r/linux, r/gnome, r/onedrive, and StrangeDaysTech Mastodon.

**Out of scope:**

- GTK4 preferences panel features beyond the four basic groups — deferred to milestone `v0.2.0-beta`.
- KDE Plasma, GTK3 (XFCE/MATE), COSMIC UIs — archived under `experimental/`, deferred to milestone `v1.0.0`.
- RPM, DEB, AUR, AppImage packaging — deferred to milestone `v0.2.0-beta` (alpha ships Flatpak only).
- Flathub submission — deferred to milestone `v0.2.0-beta`.
- i18n / translations — structure in `v0.2.0-beta`, 5+ languages in `v1.0.0`.
- Telemetry / crash reporting — deferred to milestone `v0.2.0-beta`.
- Landing page on strangedays.tech — deferred to milestone `v0.2.0-beta`.
- D-Bus full Unix-socket fallback — alpha ships only the health monitor; full fallback in `v0.2.0-beta`.
- `cargo tarpaulin` coverage reports — best-effort in alpha, formal target in `v0.2.0-beta`.

## Files to modify

This Charter spans many files across 7 phases. The table below names the load-bearing changes per phase; mechanical sweeps (path renames, version bumps) are described once and not enumerated.

| File | Change |
|---|---|
| `.straymark/charters/01-road-to-v0-1-0-alpha-1.md` | This Charter (declared) |
| `.straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0.1.0-alpha-foundation.md` | Originating AILOG, `risk_level: medium`, `review_required: true` |
| `experimental/lnxdrive-{gtk3,plasma,cosmic}/` | New directory; `git mv` from monorepo root (Fase 0) |
| `experimental/README.md` | New; explains why these UIs are archived, what reactivates them |
| `README.md`, `CLAUDE.md`, `GEMINI.md`, `ayuda.md` | Remove archived UIs from the monorepo matrix (Fase 0) |
| `lnxdrive-engine/crates/lnxdrive-ipc/src/{service.rs, auth_backend.rs}` + `lnxdrive-engine/crates/lnxdrive-daemon/src/goa_auth_backend.rs` (new) + `lnxdrive-graph/src/auth.rs` (`KeyringTokenStorage`) | `RISK-002`: OAuth tokens moved off the public D-Bus surface. The token-bearing `Auth.CompleteAuthWithTokens(...)` method is replaced by `Auth.CompleteAuthViaGOA(goa_account_path) → bool`; `GoaAuthBackend` fetches the token from GOA and persists it in the keyring via `KeyringTokenStorage`, so tokens never cross D-Bus. **Drift from original scope:** the Charter scoped an "opaque `SessionHandle`" exposed by a new `dbus_iface.rs`; the shipped, security-equivalent design uses the GOA-path method instead (no handle issued) in the existing `service.rs`. Deliberate operator decision (minimum-viable, pre-release alpha); the broader `TokenSource` abstraction is deferred to TDE-2026-05-29-001. See AILOG-2026-05-29-002 (Context + Drift). Row backported per the R4 atomic-update rule during the Fase-1 external audit (2026-05-28). (Fase 1) |
| `lnxdrive-engine/crates/lnxdrive-fuse/src/{inode_entry.rs, filesystem.rs, hydration.rs}` + `tests/integration_write_during_hydration.rs` (new) | `RISK-003`: per-inode `parking_lot::Mutex` on `InodeEntry`; `FuseHandler::write()` returns `EBUSY` (was `EIO`) when `HydrationManager::is_hydrating(ino)` under the inode lock; `HydrationManager::hydrate()` registers in the active map atomically with the lock before any `.await`. The original Charter entry pointed at `write_serializer.rs` based on the risk doc; audit on 2026-05-28 confirmed `write_serializer.rs` was already implemented (serializes DB writes via `tokio::sync::mpsc`) and the actual data-integrity gap was the FUSE write path. (Fase 1) |
| `lnxdrive-engine/crates/lnxdrive-daemon/src/{health.rs (new), main.rs}` + `lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs` | `RISK-001`: D-Bus session bus health monitor + reconnect. New `health.rs` supervises the connection (active `get_id()` probe + timeout; reconnect with backoff re-registering all 9 interfaces; yields on name-taken). `main.rs` wraps `DbusService` in `Arc`, hands the connection to the monitor, and splits `run`/`run_inner` for a single monitor-join exit point. `service.rs` adds a `DaemonState::dbus_health` field + read-only `dbus_health` property on `StatusInterface` (distinct from cloud `connection_status`). Original entry named only `health.rs`; `main.rs` + cross-crate `service.rs` added atomically (drift R7, AILOG-2026-05-28-002). NameLost fast-path and full Unix-socket fallback deferred to v0.2. (Fase 1) |
| `lnxdrive-engine/Cargo.toml` + `crates/lnxdrive-core/src/config.rs` + `crates/{lnxdrive-core,lnxdrive-cli}/Cargo.toml` + `crates/lnxdrive-cli/src/commands/config.rs` + `lnxdrive-engine/tests/security/billion_laughs.yaml` (new) | `ISSUE-002`: YAML hardening + regression fixture. Migrate `serde_yaml 0.9` (deprecated) → `serde_norway` (RUSTSEC-recommended fork with built-in recursion + alias-repetition caps, on by default), and add a 1 MiB input size cap in a new `Config::from_yaml_str`. Original entry named `lnxdrive-config/src/parser.rs` (no such crate exists); the real config parser is `lnxdrive-core/src/config.rs`. Final mitigation shape = in-tree size cap + alias cap delegated to the library (not a hand-written pre-scanner). Dependency decision recorded in AIDEC-2026-05-28-001; details + cross-crate sweep (lnxdrive-cli) in AILOG-2026-05-28-003. (Fase 1) |
| `.github/workflows/engine-ci.yml` (new, repo root) + `lnxdrive-engine/deny.toml` (new) + workspace `Cargo.toml`/`Cargo.lock` | `cargo deny` + supply-chain hardening (Fase 1). **Premise correction:** the engine CI lived at `lnxdrive-engine/.github/workflows/ci.yml`, a subdirectory path GitHub Actions ignores, so it **never ran** (fmt/clippy/build/test/audit never enforced). Relocate to the repo root with `working-directory` + path filter so it runs; add a `cargo-deny` job + `deny.toml` (subsumes the planned separate `cargo audit`). Resolve 6 advisories (cargo update + drop prometheus protobuf feature), defer 2 breaking ones (sqlx 0.8, paste) as TDE-2026-05-28-002, fix 5 pre-existing clippy lints, leave fmt non-blocking pending the workspace reformat (TDE-2026-05-28-001). Details: AILOG-2026-05-28-004. (Fase 1) |
| `lnxdrive-engine/specs/002-files-on-demand/tasks.md` | Close the one remaining `[ ]` task (Fase 2) |
| The ~4 engine files containing `todo!()/unimplemented!()` (incl. `audit.rs`, `filesystem.rs`) | Implement, remove, or feature-gate; replace ~10 debug `println!` with `tracing::debug!` (Fase 2) |
| `lnxdrive-gnome/src/main.rs`, `lnxdrive-gnome/data/ui/preferences.ui` (new), `lnxdrive-gnome/Cargo.toml` | GTK4 prefs panel with 4 settings groups (Fase 3) |
| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` | Complete install stages, permissions, target `org.gnome.Platform 47` (Fase 4) |
| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in` | Full description, releases section, screenshot URLs (Fase 4) |
| `lnxdrive.spdx` | Replace contents — currently describes StrayMark; should describe LNXDrive (Fase 4) |
| `.github/workflows/release.yml` (new) | Tag → flatpak-builder bundle → GitHub Release + SHA256SUMS (Fase 5) |
| `SECURITY.md` (new) | Disclosure policy, SLA, known limitations referencing risk-analysis docs (Fase 5) |
| `CHANGELOG.md` (new) | `[0.1.0-alpha.1]` initial entry (Fase 5) |
| `docs/screenshots/*.png` (new, 6 files) | Indicator, status menu, Nautilus overlays, prefs window, conflict dialog, onboarding wizard (Fase 5) |
| Every `Cargo.toml` with `version =`, all manifests, metainfo XML | Unify to `0.1.0-alpha.1` (Fase 5) |
| `README.md` | Install section with `flatpak install` command, screenshot embeds, comparison table (Fase 5) |
| Per-phase AILOG | One per phase under `.straymark/07-ai-audit/agent-logs/{daemon,gnome,guide,packaging,testing}/`, `risk_level` per phase (`high` for Fase 1 RISK-002, `medium` for Fase 1 others + Fase 4, `low` for Fase 2/3/5/6) |

## Verification

### Local checks

```bash
# Governance — Charter and AILOG present, status moved correctly
straymark status | grep -E "Charters|AILOG.*5[78]|17/17"
straymark charter status CHARTER-01-road-to-v0-1-0-alpha-1
straymark validate          # 0 errors required

# Engine — build & test pass on a clean checkout
cd lnxdrive-engine
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit                  # no High/Critical advisories
cargo deny check             # licenses, bans, sources

# Security — RISK-002 regression: D-Bus must not leak tokens
# Run the daemon in a nested session, watch the session bus during full auth flow.
dbus-monitor --session > /tmp/dbus-trace.log &
DBUS_MON_PID=$!
# (run end-to-end auth + sync, then stop)
kill $DBUS_MON_PID
! grep -E "Bearer |eyJ[A-Za-z0-9_-]{20,}|refresh_token" /tmp/dbus-trace.log

# Packaging — Flatpak bundle builds and installs cleanly
flatpak-builder --user --install --force-clean build-dir \
  lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml

# Release artifact — SHA256SUMS verifies bundle
sha256sum --check SHA256SUMS

# Drift check before commit (manual until Phase 2 of CLI roadmap ships)
straymark charter drift CHARTER-01-road-to-v0-1-0-alpha-1 origin/main..HEAD
```

### Production smoke (after deploy)

```bash
# On a clean Fedora 41 or Ubuntu 24.10 VM (lnxdrive-testing/ has QEMU infra):
flatpak install --user \
  https://github.com/StrangeDaysTech/lnxdrive/releases/download/v0.1.0-alpha.1/lnxdrive.flatpak
flatpak run com.strangedaystech.LNXDrive
# Manual: GOA sign-in → 100-file folder sync → offline edit → reconnect → no spurious conflict.

# GitHub release exists and is marked pre-release
gh release view v0.1.0-alpha.1 --json isPrerelease,assets | jq

# Milestone closed
gh issue list --milestone v0.1.0-alpha --state closed | grep -E "RISK-00[123]|ISSUE-002"

# Charter closed with telemetry
straymark charter status CHARTER-01-road-to-v0-1-0-alpha-1 | grep "status: closed"
```

## Risks

- **R1 — RISK-002 refactor explodes scope.** Probability medium, severity high.
  If moving tokens to keyring requires re-architecting `lnxdrive-graph`'s auth flow beyond a localized adapter swap (e.g., refresh-token semantics change), the 1.5–2 week budget for Fase 1 absorbs the slip; if it leaks into Fase 2+, document as `R<N+1> (new, not in Charter)` in the corresponding AILOG and surface to the operator before Fase 3 starts.
- **R2 — Flatpak bundle behaves differently than `cargo run`.** Probability high (sandboxing always surprises), severity medium.
  Mitigation: smoke-test the Flatpak in a Fedora 41 + Ubuntu 24.10 VM via `lnxdrive-testing/` infra **before** publishing the release. If FUSE mount fails under the sandbox, escalate to a v0.1.0-alpha.2 with the missing portal/permission rather than ship broken.
- **R3 — `cargo test --workspace` reveals flaky tests once turned on in CI.** Probability medium, severity low.
  Mitigation: Fase 2 budgets time to triage; flaky tests get marked `#[ignore]` with an issue + comment, never silently deleted. If >10% of the suite is flaky, treat as a `TDE` and pause Fase 2 to address it.
- **R4 — Drift between this Charter's declared files and what actually gets touched.** Probability medium, severity low.
  Mitigation: run `straymark charter drift CHARTER-01-road-to-v0-1-0-alpha-1 origin/main..HEAD` before opening each phase PR; document deviations in the phase AILOG under `## Risk` as `R<N+1> (new, not in Charter)` and atomically update this Charter's `## Files to modify` table in the same PR (format v4 atomic-update pattern).
- **R5 — Public announcement attracts more bug reports than the team can triage.** Probability medium, severity low.
  Mitigation: the announcement explicitly markets the release as alpha for early adopters, points users to the GitHub issue tracker with a triage SLA in `SECURITY.md`, and the `Road to v0.1.0` project board makes the open backlog visible so reporters see their issue isn't a black hole.

## Tasks

This Charter executes in **multi-batch mode** (7 phases over 5–7 calendar weeks). After each phase's PR merges, run `straymark charter batch-complete CHARTER-01-road-to-v0-1-0-alpha-1 <N>` to advance the Batch Ledger in the phase AILOG.

1. Sync `main`, branch `chore/governance-foundation-v0.1.0-alpha` (this PR — Fase 0 part 1).
2. Fase 0 part 2 (separate PR): create GitHub milestones, project board, bulk-convert backlog to issues.
3. Fase 1 (one PR per risk, in order RISK-002 → RISK-003 → RISK-001 → ISSUE-002, + final CI hardening PR).
4. Fase 2: engine polish + `cargo test --workspace` in CI.
5. Fase 3: GTK4 preferences panel.
6. Fase 4: Flatpak packaging + `lnxdrive.spdx` fix + metainfo completion.
7. Fase 5: release infrastructure (`release.yml`, `SECURITY.md`, `CHANGELOG.md`, screenshots, version unification, README install section).
8. Per phase: AILOG with appropriate `risk_level` and `review_required` flags; if scope drift detected, atomically update this Charter's `## Files to modify`.
9. Pre-commit each phase: `straymark charter drift CHARTER-01-road-to-v0-1-0-alpha-1 origin/main..HEAD`. Document drift in the phase AILOG.
10. Fase 6: signed git tag `v0.1.0-alpha.1`, `release.yml` produces bundle, publish GitHub Pre-release, announcement on r/linux + r/gnome + r/onedrive + Mastodon.
11. `straymark charter close CHARTER-01-road-to-v0-1-0-alpha-1` with telemetry comparing declared vs actual (effort estimate accuracy, R1–R5 outcomes, emergent risks count).

## Charter Closure

When closing this Charter:

1. **Atomic update (format v4)**: if the drift check (Tasks #9) reported any drift not already captured in a phase AILOG, edit `## Files to modify` and/or add a `## Closing notes` block in the **same commit/PR** that closes Fase 6 — not as a separate housekeeping PR.

2. **Post-merge drift check**: run `straymark charter drift CHARTER-01-road-to-v0-1-0-alpha-1 origin/main..HEAD` once main has the closing commit; validate the output is clean or that all drifts are documented in the corresponding phase AILOG.

3. **Move the row** in `.straymark/charters/README.md` (if present) to `## Closed` and reference the Fase 6 PR.

4. **Status frontmatter** moves from `in-progress` to `closed`; add `closed_at: 2026-XX-XX`.

5. **Do not delete** this file — the planning history matters as much as the AILOG of execution.

<!--
Format conventions — 7 patterns embedded in this template, distilled from the
6-cycle Sentinel /plan-audit experiment (2026-04-28). The provenance is part of the
historical record. The original conventions block (the template's footer comment) is
preserved unchanged below for future maintainers; the 7 conventions are:

1. Verification splits into `### Local checks` (executable literal in clean shell)
   and `### Production smoke (after deploy)` (not executable without infrastructure).

2. Effort is measured in TIME (XS/S/M/L), not in `~N lines`.

3. Modifiers like `(optional)` or `(after deploy)` live as structured sub-sections,
   never as inline parenthetical comments.

4. R<N> risks are enumerated in the Charter; new risks emergent during execution are
   documented in the AILOG as `R<N+1> (new, not in Charter)`.

5. The `## Charter Closure` section requires the implementer to update the Charter
   doc atomically (same PR as the fix) when drift is detected by Tasks #9, not in
   a separate post-merge housekeeping PR.

6. Auto-checklist drift (`straymark charter drift`) runs in pre-commit and at
   Charter closure. Detects OMISSION and SCOPE EXPANSION drifts.

7. When a Charter closes an Etapa or SpecKit `Polish` Phase, the polish Charter
   doubles as a debt-detection mechanism — its load-bearing job is to exercise the
   documented operator runbook end-to-end against the real binary. See
   `.straymark/00-governance/POLISH-CHARTER-PATTERN.md`.
-->

```

---

## Diff

```diff
diff --git a/.straymark/07-ai-audit/agent-logs/gnome/AILOG-2026-05-31-002-fase-3-gtk4-panel-audit-and-fixes.md b/.straymark/07-ai-audit/agent-logs/gnome/AILOG-2026-05-31-002-fase-3-gtk4-panel-audit-and-fixes.md
new file mode 100644
index 0000000..701f9ff
--- /dev/null
+++ b/.straymark/07-ai-audit/agent-logs/gnome/AILOG-2026-05-31-002-fase-3-gtk4-panel-audit-and-fixes.md
@@ -0,0 +1,120 @@
+---
+id: AILOG-2026-05-31-002
+title: Fase 3 — GTK4 preferences panel audit + findings remediation
+status: draft
+created: 2026-05-31
+agent: claude-opus-4-8-v1.0
+confidence: high
+review_required: true
+risk_level: medium
+tags: [gnome, preferences, gtk4, dbus, goa, risk-002, charter-01, phase-3, audit]
+related:
+  - CHARTER-01-road-to-v0-1-0-alpha-1
+  - phase-3-gtk4-panel-audit
+  - AIDEC-2026-05-31-001
+  - AILOG-2026-05-29-002
+eu_ai_act_risk: not_applicable
+nist_genai_risks: [information_security]
+iso_42001_clause: [8]
+---
+
+# AILOG: Fase 3 — GTK4 panel audit + remediation
+
+## Summary
+
+Fase 3 was scoped as "implement the GTK4 preferences panel (currently a stub)".
+The stub is only `lnxdrive-gnome/src/main.rs`; the real panel already exists under
+`lnxdrive-gnome/preferences/` and compiles. Per the operator, the work became a
+**deep audit** of that panel (3 parallel Explore agents, calibrated against
+source — `.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`) followed by
+remediation of the findings.
+
+Six findings (1 High, 3 Medium, 1 Low, 1 gap); four agent over-classifications
+rejected. All resolved: **H1–H5 fixed, G1 deferred** ([[AIDEC-2026-05-31-001]]).
+`cargo check` and `cargo clippy -- -D warnings` are clean for the panel (the
+latter for the first time).
+
+## The findings & fixes
+
+- **H1 (High) — RISK-002 drift.** Fase 1 removed `Auth.CompleteAuthWithTokens`
+  from the daemon and shipped `CompleteAuthViaGOA` (tokens off the bus), but the
+  panel still called the removed method, and its GOA code sat behind a `goa`
+  feature that `Cargo.toml` never defined → compiled out. This is the **third
+  occurrence (N=3)** of the "declared but not wired" pattern reported upstream to
+  StrayMark (#205), and the first that is a *regression* of a shipped Fase-1
+  mitigation. Fix: define the `goa` feature (default on); add the
+  `complete_auth_via_goa` proxy and drop `complete_auth_with_tokens`; hand the
+  GOA account object-path to the daemon (tokens never client-side). This also
+  surfaced and fixed a latent type error in `goa_sso` that had **never compiled**
+  (the feature was always off) — concrete evidence the GOA path was dead code.
+  The manual browser auth path (`start_auth` + `AuthStateChanged`) was unaffected.
+- **H2 (Medium) — daemon state not consumed.** Added the missing Sync/Status
+  properties+signals and `Settings.config_changed` to the proxies, and wired a
+  real consumer (AccountPage refreshes quota on `QuotaChanged`).
+- **H3 (Medium) — silent errors.** `folder_tree`, `sync_page`, and the onboarding
+  pages now surface load/save failures in the UI (inline error, error group,
+  toast/banner) instead of stderr; `folder_tree` distinguishes a parse error from
+  an empty tree.
+- **H4 (Medium) — folder_tree load race.** Merged the two independent load tasks
+  into one ordered task (selections first, then populate) so selections can no
+  longer apply to an empty tree.
+- **H5 (Low) — lint debt.** `cargo check` warnings cleared (unused imports,
+  deprecated `ActionRow::icon_name` → `add_prefix`); the audit also surfaced 145
+  pre-existing `needless_borrow` clippy lints across the panel (the panel had
+  never passed clippy `-D warnings`), auto-fixed in this pass.
+- **G1 (gap) — "System" settings group.** Deferred to a v0.2 Charter
+  ([[AIDEC-2026-05-31-001]]): cache/dehydration need new daemon D-Bus API and are
+  post-alpha. Fase 3 ships three wired groups (Account, Folders, Network) +
+  Conflicts.
+
+## Rejected (calibration)
+
+The `.expect()` cascades in GTK factories (idiomatic, type-guaranteed), the
+missing `Files` interface (Nautilus' concern, not the prefs UI), the
+`STRATEGY_VALUES[i]` index (equal-length consts), and a FUSE-style async deadlock
+(zbus uses async-io + glib `spawn_local`, no `block_on`) were verified and
+rejected as not-a-bug for this codebase.
+
+## Verification
+
+```bash
+cd lnxdrive-gnome/preferences
+cargo check                              # clean
+cargo clippy --all-targets -- -D warnings  # clean (first time for the panel)
+```
+
+Runtime verification (panel launches, authenticates against a live daemon, pages
+load/save over D-Bus, GOA flow) is **manual** — it needs a GTK display and an
+authenticated daemon, the same constraint class as the FUSE mount test; recorded
+as a follow-up, not run in this environment.
+
+## Drift
+
+- Fase 3 scope as written ("implement from a stub") did not match reality (panel
+  ~95% built). Re-framed as audit + remediation; Charter row updated.
+- G1 dropped from the alpha (deferred to v0.2), reducing "four groups" to three +
+  Conflicts. Documented in the AIDEC and Charter.
+- An external pre-merge audit of this phase is planned before merge, per the
+  operator's phase-scoped external-audit workflow.
+
+## Risk
+
+All changes are in the GTK client; no daemon code changed. H1 realigns the panel
+with the (already shipped, audited) RISK-002 daemon API, so it cannot reintroduce
+the token-on-bus exposure — it removes the client-side token fetch entirely. The
+proxy additions (H2) are declarative. Error surfacing (H3) and the load
+reordering (H4) only change UI behaviour. The clippy auto-fix (H5) is mechanical.
+No tests broken; the panel has no unit tests (UI), so runtime behaviour rests on
+the planned manual verification + external audit.
+
+## Telemetry
+
+| Metric | Value |
+|---|---|
+| Findings (audit) | 6 (1 High, 3 Medium, 1 Low, 1 gap) + 4 rejected |
+| Findings resolved | H1–H5 fixed, G1 deferred |
+| Files changed | ~13 (panel) + 3 governance docs |
+| New docs | audit, AIDEC, this AILOG |
+| clippy lints cleared | 145 needless_borrow + others |
+| Daemon code changed | 0 |
+| Pre-commit hook failures | none |
diff --git a/.straymark/07-ai-audit/decisions/AIDEC-2026-05-31-001-defer-system-settings-group.md b/.straymark/07-ai-audit/decisions/AIDEC-2026-05-31-001-defer-system-settings-group.md
new file mode 100644
index 0000000..2306506
--- /dev/null
+++ b/.straymark/07-ai-audit/decisions/AIDEC-2026-05-31-001-defer-system-settings-group.md
@@ -0,0 +1,102 @@
+---
+id: AIDEC-2026-05-31-001
+title: Posponer el grupo de ajustes "System" del panel (G1) a v0.2
+status: accepted
+created: 2026-05-31
+agent: claude-opus-4-8-v1.0
+confidence: high
+review_required: true
+risk_level: low
+tags: [gnome, preferences, settings, scope, deferral, charter-01, phase-3, v0.2]
+related:
+  - CHARTER-01-road-to-v0-1-0-alpha-1
+  - phase-3-gtk4-panel-audit
+---
+
+# AIDEC: Posponer el grupo "System" (G1) a v0.2
+
+## Context
+
+La auditoría de Fase 3 (`.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`)
+registró el hallazgo **G1**: el Charter-01 nombra cuatro grupos de ajustes
+(Account, Folders, Network, System), pero el panel implementa Account, Sync
+(≈Folders), Advanced (≈Network) y Conflicts — **no existe un grupo "System"**, y
+el daemon **no expone API D-Bus** para sus ajustes candidatos: arranque
+automático, gestión de caché y política de deshidratación.
+
+De esos tres, solo el **arranque automático** es implementable sin API D-Bus
+nueva (gestionando una unit de usuario de systemd o un `.desktop` de autostart
+desde el panel). **Caché** y **deshidratación** requieren extender la interfaz
+`Settings` del daemon con métodos nuevos y la lógica para aplicarlos — trabajo
+**cruzado** (daemon + panel) y de diseño no trivial.
+
+## Problem
+
+¿Dónde y cuándo abordamos G1, dado que el Charter-01 es estrictamente "Road to
+v0.1.0-alpha.1" y G1 mezcla un control trivial (auto-start) con ajustes que no
+tienen backend y exceden el MVP del alpha?
+
+## Alternatives Considered
+
+### Alternativa 1 — Implementar G1 completo ahora, dentro de Charter-01
+
+Crear la página "System" con auto-start + caché + deshidratación, añadiendo la
+API D-Bus necesaria en el daemon.
+
+**Pros:** cierra el "cuatro grupos" literal del Charter.
+**Cons:** caché/deshidratación **no son MVP alpha**; obliga a diseñar y exponer
+API D-Bus nueva (superficie + pruebas) bajo presión del release alpha; infla un
+Charter cuyo objetivo declarado es el alpha mínimo. Contradice
+[[feedback_minimum_viable_plus_tde]].
+
+### Alternativa 2 — Página "System" solo con auto-start ahora, resto diferido
+
+Enviar una página con el único control implementable y dejar caché/deshidratación
+para después.
+
+**Pros:** algo de "System" visible en el alpha sin API nueva.
+**Cons:** un grupo "System" a medias (un solo toggle) confunde más que ayuda;
+mezcla alcance v0.1 y v0.2 en un mismo grupo; habría que rediseñarlo al añadir el
+resto. Bajo valor para el usuario alpha.
+
+### Alternativa 3 — Fase nueva dentro de Charter-01 para G1
+
+Añadir una "Fase 7: System settings" al roadmap de Charter-01.
+
+**Pros:** mantiene G1 rastreado en el Charter activo.
+**Cons:** **incoherente con el alcance del Charter** — Charter-01 es "Road to
+v0.1.0-alpha.1"; una fase de ajustes que requiere API nueva y no es MVP no
+pertenece a un Charter de alpha. Diluiría el criterio de "hecho" del alpha.
+
+### Alternativa 4 — Diferir G1 a un Charter v0.2 futuro (ELEGIDA)
+
+Documentar G1 como diferido; abordarlo en un Charter v0.2 (cuando v0.2 arranque),
+junto con el resto de ajustes avanzados y su API D-Bus.
+
+**Pros:** respeta el alcance del alpha; agrupa el grupo "System" completo de forma
+coherente (auto-start + caché + deshidratación + su API) en el ciclo donde
+pertenece; no introduce API D-Bus a medias en el alpha.
+**Cons:** el panel del alpha mostrará tres grupos en vez de cuatro — aceptable y
+documentado.
+
+## Decision
+
+**Alternativa 4.** G1 (grupo "System") se **pospone a v0.2** y se abordará en un
+**Charter v0.2 futuro**, no como fase de Charter-01 ni como implementación parcial
+en el alpha. No se crea el Charter v0.2 ahora (sería prematuro y de un solo ítem);
+esta AIDEC es la semilla de seguimiento y se promoverá al backlog de v0.2 cuando
+ese ciclo comience.
+
+El Charter-01 se actualiza para reflejar que la Fase 3 entrega **tres** grupos de
+ajustes wired al daemon (Account, Folders/Sync, Network/Advanced) más Conflicts,
+y que el grupo "System" queda **fuera de alcance del alpha** por esta decisión.
+
+## Consequences
+
+- El panel del alpha no tendrá grupo "System"; el arranque automático se gestiona
+  por el packaging/systemd del alpha, no por la UI todavía.
+- Cuando arranque v0.2, su Charter incluirá: API D-Bus de caché y deshidratación
+  en `Settings`, y la página "System" del panel (auto-start + caché +
+  deshidratación) que las consume.
+- Fase 3 puede cerrarse con los hallazgos **H** (H1–H5) resueltos sin bloquear por
+  G1.
diff --git a/.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md b/.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md
new file mode 100644
index 0000000..9220bab
--- /dev/null
+++ b/.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md
@@ -0,0 +1,194 @@
+---
+audit_role: internal-calibrated-audit
+calibrator: claude-opus-4-8
+charter_id: CHARTER-01-road-to-v0-1-0-alpha-1
+phase: "Fase 3 — GTK4 preferences panel"
+component: lnxdrive-gnome/preferences
+audited_at: 2026-05-31
+method: 3 parallel Explore agents (D-Bus contract / UI logic / async+build), reconciled and code-verified by the calibrator
+findings_consolidated: 6
+findings_by_severity:
+  high: 1
+  medium: 3
+  low: 1
+  gap: 1
+false_positives_rejected: 4
+verdict: FUNCTIONAL_WITH_DRIFT
+---
+
+# Internal audit — Fase 3 GTK4 preferences panel
+
+**Reviewer:** claude-opus-4-8
+**Date:** 2026-05-31
+**Confidence:** High
+**Component:** `lnxdrive-gnome/preferences/` (binary `lnxdrive-preferences`)
+
+## 1. Executive summary
+
+Fase 3 of Charter-01 is scoped as "implement the GTK4 preferences panel
+(currently a `println!("not yet implemented")` stub)". That stub is only the
+placeholder `lnxdrive-gnome/src/main.rs`; the **real panel already exists and is
+~95% built** under `lnxdrive-gnome/preferences/` — an `adw::Application` with a
+typed zbus client, an onboarding wizard, and four pages (Account, Sync,
+Conflicts, Advanced). It **compiles** (`cargo check` clean, 12 warnings) and,
+unlike the FUSE crate audited in Fase 2, it **does not have a fatal runtime trap**
+(async is correctly `async-io` + glib `spawn_local`, no stray `block_on`/
+`tokio::spawn`; GSettings schema id/keys, app-id, and build wiring are
+consistent).
+
+The audit was run because "compiles" is not "works" — the panel had never been
+exercised against a real daemon, and the zbus proxy contract is validated at
+**runtime**, not compile time. Three Explore agents (D-Bus contract / UI logic /
+async+build) produced findings that the calibrator reconciled and verified
+against source, rejecting four agent over-classifications.
+
+**The one serious finding is a cross-component governance drift (H1):** Fase 1
+(RISK-002) removed `Auth.CompleteAuthWithTokens` from the daemon and replaced it
+with `Auth.CompleteAuthViaGOA` to keep OAuth tokens off the D-Bus surface, but
+the **panel was never updated** — it still declares/calls `complete_auth_with_tokens`
+(now nonexistent), and that GOA code is behind `#[cfg(feature = "goa")]` while
+`Cargo.toml` defines **no `goa` feature**, so GOA SSO is compiled out entirely.
+This is the **third occurrence (N=3)** of the "declared but not wired" pattern
+already reported upstream to StrayMark (#205) — and the first one that is a
+*regression* of a shipped Fase-1 mitigation rather than an original gap.
+
+Mitigating fact: the **manual browser auth path works** (`start_auth()` +
+`AuthStateChanged` signal, both present on the daemon — `auth_page.rs:238-295`),
+so the panel can still authenticate; only the GOA "use your existing Microsoft
+account" path (FR-019–023) is broken. Hence H1 is **High, not Critical**.
+
+**Overall verdict: FUNCTIONAL_WITH_DRIFT.** The panel runs and mostly works; the
+material work is fixing the RISK-002 drift, three medium robustness items, lint
+cleanup, and the absent "System" group.
+
+## 2. Scope
+
+Audited: every Rust source under `lnxdrive-gnome/preferences/src/`, the zbus
+client contract against the daemon's interfaces in
+`lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs`, plus `Cargo.toml`,
+`meson.build`, the GSettings schema, and the desktop/metainfo files. Not run:
+live execution against a mounted daemon (no authenticated account available in
+this environment) — deferred to manual verification.
+
+## 3. Findings (calibrated)
+
+### H1 — RISK-002 drift: GOA auth broken & compiled out — **HIGH**
+
+- **Client** declares and calls `complete_auth_with_tokens(access_token,
+  refresh_token, expires_at_unix)` — `dbus_client.rs:88,235-243` and
+  `onboarding/auth_page.rs:362`.
+- **Daemon** removed that method in Fase 1; only `complete_auth(code, state)`
+  (`service.rs:873`) and `complete_auth_via_goa(goa_account_path)`
+  (`service.rs:917`) exist. `service.rs:902` states it "replaces the historical
+  `CompleteAuthWithTokens`"; the tests at `service.rs:2004` confirm it was
+  deleted.
+- The GOA UI is gated on `#[cfg(feature = "goa")]` (`auth_page.rs:20,35,143,338`)
+  but `preferences/Cargo.toml` defines **no `[features]`** → the gate is always
+  false → GOA SSO is compiled out (also the source of the `unexpected cfg value
+  'goa'` warnings).
+- **Impact:** GOA SSO (FR-019–023) is non-functional and, if re-enabled as-is,
+  would call a method the daemon no longer exposes (`UnknownMethod` at runtime).
+  Manual browser auth is unaffected.
+- **Remediation:** (a) add `[features] goa = []` (decide default on/off) to
+  `Cargo.toml`; (b) add a `complete_auth_via_goa(goa_account_path)` proxy method
+  to `dbus_client.rs` and drop/deprecate `complete_auth_with_tokens`;
+  (c) rewrite `auth_page.rs::on_goa_sign_in_clicked` to pass the GOA account
+  object path to `complete_auth_via_goa` instead of fetching tokens client-side.
+
+### H2 — Daemon state not consumed (no live status) — **MEDIUM**
+
+- The client proxies omit several daemon-exposed properties/signals: `Sync`
+  `sync_status`/`last_sync_time`/`pending_changes` + `sync_started`/
+  `sync_completed`/`sync_progress` (`service.rs:652-705`); `Status`
+  `connection_status`/`dbus_health` + `quota_changed`/`connection_changed`
+  (`service.rs:762-790`); `Settings.config_changed` (`service.rs:1066`).
+- **Impact:** the panel shows no live sync/connection status and does not refresh
+  on external changes. Functional gap, not a crash.
+- **Remediation:** add the missing properties/signals to the proxies and wire a
+  minimal set (sync status + quota refresh) into the relevant pages.
+
+### H3 — Silent error handling — **MEDIUM**
+
+- D-Bus call failures go to `eprintln!`/stderr, not the UI (e.g.
+  `sync_page.rs:200`, `account_page.rs`), so a dead daemon leaves the panel
+  showing default values as if loaded.
+- JSON parsing uses `unwrap_or_default()` (`folder_tree.rs:416`), so a malformed
+  `GetRemoteFolderTree` response renders an **empty tree indistinguishable from
+  "no folders"**.
+- **Impact:** silent degradation; user operates on stale/empty UI believing it
+  loaded.
+- **Remediation:** surface load/save failures via an `adw::Toast`/banner;
+  distinguish parse-error from empty in `folder_tree`.
+
+### H4 — `folder_tree` load race — **MEDIUM**
+
+- `FolderTree::new` fires `load_remote_tree()` and `load_selected_folders()` as
+  two independent `spawn_local` tasks (`folder_tree.rs:205-206`); `apply_selections`
+  can run before the tree is populated, dropping the selection highlight. A
+  related issue: `apply_selections` only walks root-level nodes, so lazily-loaded
+  children are not marked.
+- **Impact:** selective-sync selections may not display correctly.
+- **Remediation:** chain selections after the tree populates (await both, or
+  apply selections in the populate continuation); apply recursively as nodes
+  expand.
+
+### H5 — Compiler warnings — **LOW**
+
+- 12 warnings: unused `gtk4::prelude` imports (`sync_page.rs:11`,
+  `onboarding/mod.rs:17`, `app.rs:12`), `unexpected cfg value 'goa'` (resolved by
+  H1's feature definition), deprecated `ActionRowBuilder::icon_name`
+  (`confirm_page.rs:90,96`).
+- **Remediation:** remove unused imports; migrate the deprecated builder call;
+  the `cfg` warnings disappear once `goa` is a declared feature.
+
+### G1 — "System" settings group absent — **GAP**
+
+- The Charter names four groups (Account, Folders, Network, System). The panel
+  has Account, Sync (≈Folders), Advanced (≈Network), Conflicts — but **no
+  "System" group**, and the daemon exposes **no D-Bus API** for its candidate
+  settings (auto-start, cache, dehydration policy).
+- **Resolution — DEFERRED to v0.2** (see [[AIDEC-2026-05-31-001]]): the whole
+  "System" group is deferred to a future v0.2 Charter rather than implemented
+  partially in the alpha. Cache and dehydration controls need new daemon D-Bus
+  API; auto-start alone would be a one-toggle group mixing v0.1/v0.2 scope. Fase 3
+  ships three wired groups (Account, Folders/Sync, Network/Advanced) + Conflicts;
+  the "System" group is out of alpha scope by that decision.
+
+## 4. Rejected (agent over-classifications)
+
+The calibrator verified and **rejected** these as not-a-bug for this codebase:
+
+- **`.expect()` cascade in GTK factories** (`folder_tree.rs:227,260-318`) — flagged
+  CRITICAL by the UI agent, but these are idiomatic gtk4-rs factory closures where
+  the item type is guaranteed by construction (`TreeListModel`/`ListItem` always
+  yield the registered type). The async+build agent correctly rated them low.
+  **Rejected as CRITICAL; at most a stylistic LOW.**
+- **`Files` interface missing from client** — the panel is the *preferences* UI;
+  pin/unpin/file-status is Nautilus' concern, not this binary's. **Not applicable.**
+- **`conflict_list.rs:296` `STRATEGY_VALUES[i]` index** — the two arrays are
+  fixed-size consts of equal length; no runtime risk exists today. **LOW, not
+  CRITICAL.**
+- **async-runtime deadlock (FUSE-style)** — verified absent: zbus uses `async-io`
+  (not tokio), all D-Bus calls run via `glib spawn_local`, no `block_on`. **No bug.**
+
+## 5. Remediation plan (→ Fase 3 implementation)
+
+Ordered, each on the `feat/charter-01-phase-3-*` branch with regression coverage
+where testable and a closing AILOG:
+
+1. **H1 (High):** define the `goa` feature; replace the client/`auth_page` token
+   path with `complete_auth_via_goa`. Backport a governance note (this is a
+   Fase-1 RISK-002 regression) and feed the N=3 "declared but not wired" data
+   point into the upstream-feedback drafts.
+2. **H3 (Medium):** toast/banner on D-Bus errors; parse-error vs empty in
+   `folder_tree`.
+3. **H4 (Medium):** fix the `folder_tree` load ordering + recursive selection.
+4. **H2 (Medium):** extend proxies and wire live sync/quota status.
+5. **G1 (Gap):** decide System-group scope; implement auto-start or document
+   deferral.
+6. **H5 (Low):** clear warnings + deprecation.
+
+Verification: `cargo clippy -p lnxdrive-preferences -- -D warnings` clean; unit
+tests for any non-GTK logic added; manual run against a live daemon recorded in
+the closing AILOG (the panel cannot be exercised end-to-end in CI — same
+`/dev/fuse`/display constraint class as the T101 mount test).
diff --git a/.straymark/audits/CHARTER-01/upstream-feedback-drafts.md b/.straymark/audits/CHARTER-01/upstream-feedback-drafts.md
index 53a33d3..031bbab 100644
--- a/.straymark/audits/CHARTER-01/upstream-feedback-drafts.md
+++ b/.straymark/audits/CHARTER-01/upstream-feedback-drafts.md
@@ -10,7 +10,8 @@
 > |---|---|---|---|
 > | 2a | `charter drift` rejects the range its Charter template ships | CLI/format friction (ad-hoc) | ✅ filed — [straymark#207](https://github.com/StrangeDaysTech/straymark/issues/207) |
 > | 2b | `charter audit --prepare` default range under-covers phase audits | Documentation gap (ad-hoc) | ✅ filed — [straymark#208](https://github.com/StrangeDaysTech/straymark/issues/208) |
-> | 1 | "declared but not wired" transfers to N=2 (crate/D-Bus surface) | Pattern candidate | 🕓 draft below — file at Charter close |
+> | 1 | "declared but not wired" — now N=3 (cross-component regression of a shipped mitigation, found in Fase 3) | Pattern candidate | ✅ filed — [straymark#209](https://github.com/StrangeDaysTech/straymark/issues/209) (advanced from Charter-close cadence: the Fase-3 panel audit produced the N=3 data point) |
+> | 4 | Charter scope declared against assumed (un-read) code → code-reconnaissance gate at creation | Process / methodology gap | ✅ filed — [straymark#210](https://github.com/StrangeDaysTech/straymark/issues/210) |
 > | 3 | External-audit calibration results (dual-model + calibrator-hunts-missed) | External audit results / pattern | 🕓 draft below — file at Charter close |
 >
 > The cadence committed in #205 is **per Charter close** for telemetry + audit
diff --git a/.straymark/charters/01-road-to-v0-1-0-alpha-1.md b/.straymark/charters/01-road-to-v0-1-0-alpha-1.md
index 4efd532..2d78735 100644
--- a/.straymark/charters/01-road-to-v0-1-0-alpha-1.md
+++ b/.straymark/charters/01-road-to-v0-1-0-alpha-1.md
@@ -29,7 +29,7 @@ The lnxdrive monorepo finished its MVP implementation (SpecKit features `001-cor
    - `ISSUE-002`: harden the YAML config parser against billion-laughs (size + alias caps); regression fixture in `lnxdrive-engine/tests/security/`.
    - `cargo audit` + `cargo deny` jobs in CI.
 3. **Engine polish** — close the one remaining task (T101 performance validation) in `lnxdrive-engine/specs/002-files-on-demand/tasks.md`. **Done** (Fase 2): T101 validated via a real-mount integration test — `getattr` 43.7µs, `readdir` 1.40ms/1000 entries, idle RSS 37.9MB/10k files (all under target). The test was the first real FUSE mount exercised in the codebase and surfaced four functional listing bugs (init runtime-context panic, root self-listing, unstable `readdir` order, `opendir` dir-cache) plus an inode-persistence defect, all fixed with regression tests — see AILOG-2026-05-31-001. The other three items this row originally listed (remove `todo!()/unimplemented!()`, remove debug `println!`, enable `cargo test --workspace` in CI) were **already completed during Fase 1** (verified against `main`: zero such sites in crates; `cargo test --workspace` live at `.github/workflows/engine-ci.yml:66`).
-4. **GTK4 preferences panel** — implement four basic settings groups (Account, Folders, Network, System) in `lnxdrive-gnome/src/main.rs` (currently a `println!("not yet implemented")` stub) wired to the existing D-Bus daemon API.
+4. **GTK4 preferences panel** — the panel already exists under `lnxdrive-gnome/preferences/` (the root `src/main.rs` stub is just a placeholder). Fase 3 audits it (`.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`) and fixes the findings. It ships **three** settings groups wired to the daemon — Account, Folders (Sync), Network (Advanced) — plus Conflicts. The fourth group, **System** (auto-start, cache, dehydration), is **deferred to a v0.2 Charter** because it needs new daemon D-Bus API and is post-alpha (see AIDEC-2026-05-31-001). Key fix: realign the panel with the Fase-1 RISK-002 daemon API (`CompleteAuthViaGOA`).
 5. **Flatpak packaging** — complete `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` with install stages (icons, `*.desktop`, metainfo XML), correct permissions (`--filesystem=home:rw`, `--talk-name=org.freedesktop.secrets`), and target `org.gnome.Platform 47`. Fix `lnxdrive.spdx` (currently describes StrayMark by mistake). Complete the metainfo XML with description, releases section, and screenshot URLs.
 6. **Release infrastructure & public assets** — `.github/workflows/release.yml` (tag → bundle → GitHub Release with SHA256SUMS); `SECURITY.md`; `CHANGELOG.md`; 6 UI screenshots in `docs/screenshots/`; version `0.1.0-alpha.1` consistent across every `Cargo.toml`, Flatpak manifest, and metainfo XML; README install section + competitive comparison vs `jstaf/onedriver` and `abraunegg/onedrive`.
 7. **Tag, release, announce** — signed tag `v0.1.0-alpha.1`, GitHub Pre-release with Flatpak bundle, posts on r/linux, r/gnome, r/onedrive, and StrangeDaysTech Mastodon.
diff --git a/lnxdrive-gnome/preferences/Cargo.toml b/lnxdrive-gnome/preferences/Cargo.toml
index 3b9bf7e..a8a4fd3 100644
--- a/lnxdrive-gnome/preferences/Cargo.toml
+++ b/lnxdrive-gnome/preferences/Cargo.toml
@@ -21,3 +21,10 @@ serde = { version = "1", features = ["derive"] }
 serde_json = "1"
 tokio = { version = "1", features = ["rt"] }
 futures-util = "0.3"
+
+[features]
+# GNOME Online Accounts SSO (FR-019–023). Enabled by default; the GOA button is
+# only shown when an "lnxdrive_microsoft" account exists, so it degrades to the
+# manual browser flow when GOA or the provider is absent.
+default = ["goa"]
+goa = []
diff --git a/lnxdrive-gnome/preferences/src/app.rs b/lnxdrive-gnome/preferences/src/app.rs
index 16b8925..b7b1b66 100644
--- a/lnxdrive-gnome/preferences/src/app.rs
+++ b/lnxdrive-gnome/preferences/src/app.rs
@@ -9,7 +9,6 @@ use gtk4::glib;
 use gtk4::prelude::*;
 use gtk4::subclass::prelude::ObjectSubclassIsExt;
 use libadwaita as adw;
-use libadwaita::prelude::*;
 
 use crate::dbus_client::DbusClient;
 use crate::window::LnxdriveWindow;
diff --git a/lnxdrive-gnome/preferences/src/conflicts/conflict_dialog.rs b/lnxdrive-gnome/preferences/src/conflicts/conflict_dialog.rs
index ffceb7d..0e4be3e 100644
--- a/lnxdrive-gnome/preferences/src/conflicts/conflict_dialog.rs
+++ b/lnxdrive-gnome/preferences/src/conflicts/conflict_dialog.rs
@@ -219,18 +219,18 @@ impl ConflictDetailDialog {
 
         // Local version
         let local_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Local Version"))
+            .title(gettext("Local Version"))
             .build();
         let local_size_row = adw::ActionRow::builder()
-            .title(&gettext("Size"))
-            .subtitle(&format_bytes(conflict.local_size))
+            .title(gettext("Size"))
+            .subtitle(format_bytes(conflict.local_size))
             .build();
         let local_modified_row = adw::ActionRow::builder()
-            .title(&gettext("Modified"))
+            .title(gettext("Modified"))
             .subtitle(&conflict.local_modified)
             .build();
         let local_hash_row = adw::ActionRow::builder()
-            .title(&gettext("Hash"))
+            .title(gettext("Hash"))
             .subtitle(&conflict.local_hash)
             .build();
         local_group.add(&local_size_row);
@@ -239,18 +239,18 @@ impl ConflictDetailDialog {
 
         // Remote version
         let remote_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Remote Version"))
+            .title(gettext("Remote Version"))
             .build();
         let remote_size_row = adw::ActionRow::builder()
-            .title(&gettext("Size"))
-            .subtitle(&format_bytes(conflict.remote_size))
+            .title(gettext("Size"))
+            .subtitle(format_bytes(conflict.remote_size))
             .build();
         let remote_modified_row = adw::ActionRow::builder()
-            .title(&gettext("Modified"))
+            .title(gettext("Modified"))
             .subtitle(&conflict.remote_modified)
             .build();
         let remote_hash_row = adw::ActionRow::builder()
-            .title(&gettext("Hash"))
+            .title(gettext("Hash"))
             .subtitle(&conflict.remote_hash)
             .build();
         remote_group.add(&remote_size_row);
@@ -263,26 +263,26 @@ impl ConflictDetailDialog {
 
         // -- Resolution actions -----------------------------------------------
         let actions_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Resolution"))
+            .title(gettext("Resolution"))
             .build();
 
         let keep_local_row = adw::ActionRow::builder()
-            .title(&gettext("Keep Local"))
-            .subtitle(&gettext("Upload the local version, overwriting the remote"))
+            .title(gettext("Keep Local"))
+            .subtitle(gettext("Upload the local version, overwriting the remote"))
             .activatable(true)
             .build();
         keep_local_row.add_suffix(&gtk4::Image::from_icon_name("go-up-symbolic"));
 
         let keep_remote_row = adw::ActionRow::builder()
-            .title(&gettext("Keep Remote"))
-            .subtitle(&gettext("Download the remote version, overwriting the local"))
+            .title(gettext("Keep Remote"))
+            .subtitle(gettext("Download the remote version, overwriting the local"))
             .activatable(true)
             .build();
         keep_remote_row.add_suffix(&gtk4::Image::from_icon_name("go-down-symbolic"));
 
         let keep_both_row = adw::ActionRow::builder()
-            .title(&gettext("Keep Both"))
-            .subtitle(&gettext("Rename the local file and download the remote version"))
+            .title(gettext("Keep Both"))
+            .subtitle(gettext("Rename the local file and download the remote version"))
             .activatable(true)
             .build();
         keep_both_row.add_suffix(&gtk4::Image::from_icon_name("edit-copy-symbolic"));
diff --git a/lnxdrive-gnome/preferences/src/conflicts/conflict_list.rs b/lnxdrive-gnome/preferences/src/conflicts/conflict_list.rs
index 55c6fc6..684b9d3 100644
--- a/lnxdrive-gnome/preferences/src/conflicts/conflict_list.rs
+++ b/lnxdrive-gnome/preferences/src/conflicts/conflict_list.rs
@@ -150,12 +150,12 @@ impl ConflictListPage {
 
         // -- Conflicts list group ---------------------------------------------
         let conflicts_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Unresolved Conflicts"))
+            .title(gettext("Unresolved Conflicts"))
             .build();
 
         // Resolve All button in the header
         let resolve_all_button = gtk4::Button::builder()
-            .label(&gettext("Resolve All"))
+            .label(gettext("Resolve All"))
             .css_classes(["flat"])
             .build();
 
@@ -167,7 +167,7 @@ impl ConflictListPage {
 
         // Empty state label
         let empty_label = gtk4::Label::builder()
-            .label(&gettext("No unresolved conflicts"))
+            .label(gettext("No unresolved conflicts"))
             .css_classes(["dim-label"])
             .margin_top(12)
             .margin_bottom(12)
@@ -222,11 +222,11 @@ impl ConflictListPage {
         self.remove(&group);
 
         let new_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Unresolved Conflicts"))
+            .title(gettext("Unresolved Conflicts"))
             .build();
 
         let resolve_all_button = gtk4::Button::builder()
-            .label(&gettext("Resolve All"))
+            .label(gettext("Resolve All"))
             .css_classes(["flat"])
             .build();
 
@@ -238,8 +238,8 @@ impl ConflictListPage {
 
         if conflicts.is_empty() {
             let empty_row = adw::ActionRow::builder()
-                .title(&gettext("No unresolved conflicts"))
-                .subtitle(&gettext("All files are in sync"))
+                .title(gettext("No unresolved conflicts"))
+                .subtitle(gettext("All files are in sync"))
                 .build();
             empty_row.add_prefix(&gtk4::Image::from_icon_name("emblem-ok-symbolic"));
             new_group.add(&empty_row);
@@ -287,8 +287,8 @@ impl ConflictListPage {
 
         // Build a simple strategy chooser dialog
         let dialog = adw::AlertDialog::builder()
-            .heading(&gettext("Resolve All Conflicts"))
-            .body(&gettext("Choose a strategy to apply to all unresolved conflicts."))
+            .heading(gettext("Resolve All Conflicts"))
+            .body(gettext("Choose a strategy to apply to all unresolved conflicts."))
             .build();
 
         dialog.add_response("cancel", &gettext("Cancel"));
diff --git a/lnxdrive-gnome/preferences/src/dbus_client.rs b/lnxdrive-gnome/preferences/src/dbus_client.rs
index f46ab7e..82dc28a 100644
--- a/lnxdrive-gnome/preferences/src/dbus_client.rs
+++ b/lnxdrive-gnome/preferences/src/dbus_client.rs
@@ -84,13 +84,11 @@ pub trait LnxdriveAuth {
     /// Finish an auth flow with an explicit code + state (manual/CLI/GOA).
     async fn complete_auth(&self, code: &str, state: &str) -> zbus::Result<bool>;
 
-    /// Complete auth using pre-obtained tokens (e.g. from GNOME Online Accounts).
-    async fn complete_auth_with_tokens(
-        &self,
-        access_token: &str,
-        refresh_token: &str,
-        expires_at_unix: i64,
-    ) -> zbus::Result<bool>;
+    /// Complete auth using an existing GNOME Online Accounts account. The daemon
+    /// fetches the tokens from GOA and persists them in the keyring itself, so
+    /// tokens never cross the D-Bus surface (RISK-002). `goa_account_path` is the
+    /// GOA account object path (e.g. `/org/gnome/OnlineAccounts/Accounts/...`).
+    async fn complete_auth_via_goa(&self, goa_account_path: &str) -> zbus::Result<bool>;
 
     /// Log out the current user and revoke tokens.
     async fn logout(&self) -> zbus::Result<()>;
@@ -128,6 +126,10 @@ trait LnxdriveSettings {
 
     /// Return the remote folder tree as a JSON string.
     async fn get_remote_folder_tree(&self) -> zbus::Result<String>;
+
+    /// Emitted when any configuration key changes (e.g. from the CLI).
+    #[zbus(signal)]
+    fn config_changed(&self, key: &str) -> zbus::Result<()>;
 }
 
 /// com.strangedaystech.LNXDrive.Status — account and quota information
@@ -136,12 +138,28 @@ trait LnxdriveSettings {
     default_service = "com.strangedaystech.LNXDrive",
     default_path = "/com/strangedaystech/LNXDrive"
 )]
-trait LnxdriveStatus {
+pub trait LnxdriveStatus {
     /// Return (used_bytes, total_bytes).
     async fn get_quota(&self) -> zbus::Result<(u64, u64)>;
 
     /// Return a dict of account metadata (display_name, email, etc.).
     async fn get_account_info(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
+
+    /// Cloud connection state: "online", "offline", or "reconnecting".
+    #[zbus(property)]
+    fn connection_status(&self) -> zbus::Result<String>;
+
+    /// Session-bus health: "online", "reconnecting", or "lost".
+    #[zbus(property)]
+    fn dbus_health(&self) -> zbus::Result<String>;
+
+    /// Emitted when the storage quota changes (e.g. after a sync).
+    #[zbus(signal)]
+    fn quota_changed(&self, used: u64, total: u64) -> zbus::Result<()>;
+
+    /// Emitted when the cloud connection state changes.
+    #[zbus(signal)]
+    fn connection_changed(&self, status: &str) -> zbus::Result<()>;
 }
 
 /// com.strangedaystech.LNXDrive.Sync — sync control
@@ -159,6 +177,30 @@ trait LnxdriveSync {
 
     /// Resume sync.
     async fn resume(&self) -> zbus::Result<()>;
+
+    /// Current sync state: "idle", "syncing", "paused", or "error".
+    #[zbus(property)]
+    fn sync_status(&self) -> zbus::Result<String>;
+
+    /// Unix timestamp of the last completed sync (0 = never).
+    #[zbus(property)]
+    fn last_sync_time(&self) -> zbus::Result<i64>;
+
+    /// Number of pending file operations.
+    #[zbus(property)]
+    fn pending_changes(&self) -> zbus::Result<u32>;
+
+    /// Emitted when a sync cycle starts.
+    #[zbus(signal)]
+    fn sync_started(&self) -> zbus::Result<()>;
+
+    /// Emitted when a sync cycle completes.
+    #[zbus(signal)]
+    fn sync_completed(&self, files_synced: u32, errors: u32) -> zbus::Result<()>;
+
+    /// Emitted for each file during sync.
+    #[zbus(signal)]
+    fn sync_progress(&self, file: &str, current: u32, total: u32) -> zbus::Result<()>;
 }
 
 /// com.strangedaystech.LNXDrive.Conflicts — conflict detection and resolution
@@ -231,17 +273,12 @@ impl DbusClient {
         Ok(proxy.complete_auth(code, state).await?)
     }
 
-    /// Complete auth with pre-obtained tokens from GNOME Online Accounts.
-    pub async fn complete_auth_with_tokens(
-        &self,
-        access_token: &str,
-        refresh_token: &str,
-        expires_at_unix: i64,
-    ) -> Result<bool, DbusError> {
+    /// Complete auth via an existing GNOME Online Accounts account. The daemon
+    /// fetches the tokens from GOA and persists them in the keyring; tokens never
+    /// cross the D-Bus surface (RISK-002). Pass the GOA account object path.
+    pub async fn complete_auth_via_goa(&self, goa_account_path: &str) -> Result<bool, DbusError> {
         let proxy = LnxdriveAuthProxy::new(&self.connection).await?;
-        Ok(proxy
-            .complete_auth_with_tokens(access_token, refresh_token, expires_at_unix)
-            .await?)
+        Ok(proxy.complete_auth_via_goa(goa_account_path).await?)
     }
 
     /// Log out the current user.
diff --git a/lnxdrive-gnome/preferences/src/goa_sso.rs b/lnxdrive-gnome/preferences/src/goa_sso.rs
index 9aa9fc5..2908d41 100644
--- a/lnxdrive-gnome/preferences/src/goa_sso.rs
+++ b/lnxdrive-gnome/preferences/src/goa_sso.rs
@@ -13,53 +13,19 @@ const GOA_MANAGER_PATH: &str = "/org/gnome/OnlineAccounts";
 
 /// Checks whether a GOA account with provider type "lnxdrive_microsoft" exists.
 pub async fn has_lnxdrive_goa_account() -> bool {
-    match find_goa_account_path().await {
-        Ok(Some(_)) => true,
-        _ => false,
-    }
+    matches!(find_goa_account_path().await, Ok(Some(_)))
 }
 
-/// Retrieves OAuth2 tokens from the existing GOA account.
+/// Returns the D-Bus object path of the existing "lnxdrive_microsoft" GOA
+/// account, if any.
 ///
-/// Returns (access_token, refresh_token, expires_at_unix) on success.
-pub async fn get_goa_tokens() -> Result<(String, String, i64), String> {
-    let path = find_goa_account_path()
-        .await
-        .map_err(|e| format!("D-Bus error: {e}"))?
-        .ok_or_else(|| "No LNXDrive GOA account found".to_string())?;
-
-    let conn = Connection::session()
+/// Post-RISK-002 the client no longer fetches tokens itself: it hands this path
+/// to the daemon via `Auth.CompleteAuthViaGOA`, and the daemon reads the tokens
+/// from GOA and stores them in the keyring, so tokens never cross D-Bus.
+pub async fn lnxdrive_goa_account_path() -> Result<Option<String>, String> {
+    find_goa_account_path()
         .await
-        .map_err(|e| format!("Session bus: {e}"))?;
-
-    // Call GetAccessToken on the OAuth2Based interface
-    let msg = conn
-        .call_method(
-            Some(GOA_BUS_NAME.into()),
-            &path,
-            Some("org.gnome.OnlineAccounts.OAuth2Based".into()),
-            "GetAccessToken",
-            &(),
-        )
-        .await
-        .map_err(|e| format!("GetAccessToken: {e}"))?;
-
-    let (access_token, expires_in): (String, i32) = msg
-        .body()
-        .deserialize()
-        .map_err(|e| format!("Deserialize: {e}"))?;
-
-    // GOA doesn't expose refresh_token via D-Bus; the GOA daemon manages it.
-    // For daemon-side refresh, we pass a sentinel and rely on GOA-aware refresh.
-    let refresh_token = "__goa_managed__".to_string();
-
-    let now = std::time::SystemTime::now()
-        .duration_since(std::time::UNIX_EPOCH)
-        .unwrap_or_default()
-        .as_secs() as i64;
-    let expires_at = now + expires_in as i64;
-
-    Ok((access_token, refresh_token, expires_at))
+        .map_err(|e| format!("D-Bus error: {e}"))
 }
 
 /// Finds the D-Bus object path of the first GOA account with provider
@@ -70,9 +36,9 @@ async fn find_goa_account_path() -> Result<Option<String>, zbus::Error> {
     // Use the ObjectManager to enumerate all GOA accounts
     let msg = conn
         .call_method(
-            Some(GOA_BUS_NAME.into()),
+            Some(GOA_BUS_NAME),
             GOA_MANAGER_PATH,
-            Some("org.freedesktop.DBus.ObjectManager".into()),
+            Some("org.freedesktop.DBus.ObjectManager"),
             "GetManagedObjects",
             &(),
         )
diff --git a/lnxdrive-gnome/preferences/src/onboarding/auth_page.rs b/lnxdrive-gnome/preferences/src/onboarding/auth_page.rs
index e4789a0..d3069cd 100644
--- a/lnxdrive-gnome/preferences/src/onboarding/auth_page.rs
+++ b/lnxdrive-gnome/preferences/src/onboarding/auth_page.rs
@@ -99,7 +99,7 @@ impl AuthPage {
 
         // Sign-in button
         let sign_in_button = gtk4::Button::builder()
-            .label(&gettext("Sign In"))
+            .label(gettext("Sign In"))
             .halign(gtk4::Align::Center)
             .css_classes(["suggested-action", "pill"])
             .build();
@@ -116,7 +116,7 @@ impl AuthPage {
 
         // Waiting-state cancel button (hidden initially)
         let cancel_button = gtk4::Button::builder()
-            .label(&gettext("Cancel"))
+            .label(gettext("Cancel"))
             .halign(gtk4::Align::Center)
             .css_classes(["destructive-action", "pill"])
             .visible(false)
@@ -126,15 +126,15 @@ impl AuthPage {
 
         // Waiting label (hidden initially, placed next to spinner)
         let waiting_label = gtk4::Label::builder()
-            .label(&gettext("Waiting for authentication..."))
+            .label(gettext("Waiting for authentication..."))
             .visible(false)
             .build();
 
         // Status page
         let status_page = adw::StatusPage::builder()
             .icon_name("dialog-password-symbolic")
-            .title(&gettext("Sign in to OneDrive"))
-            .description(&gettext(
+            .title(gettext("Sign in to OneDrive"))
+            .description(gettext(
                 "Connect your Microsoft account to start syncing files.",
             ))
             .build();
@@ -143,7 +143,7 @@ impl AuthPage {
         #[cfg(feature = "goa")]
         {
             let goa_button = gtk4::Button::builder()
-                .label(&gettext("Use existing Microsoft account"))
+                .label(gettext("Use existing Microsoft account"))
                 .halign(gtk4::Align::Center)
                 .css_classes(["suggested-action", "pill"])
                 .visible(false) // hidden until GOA check completes
@@ -356,12 +356,11 @@ impl AuthPage {
         let wl = waiting_label.clone();
 
         glib::MainContext::default().spawn_local(async move {
-            match goa_sso::get_goa_tokens().await {
-                Ok((access_token, refresh_token, expires_at)) => {
-                    match dbus_client
-                        .complete_auth_with_tokens(&access_token, &refresh_token, expires_at)
-                        .await
-                    {
+            // Hand the GOA account path to the daemon; it fetches the tokens from
+            // GOA itself (RISK-002 — tokens never cross D-Bus).
+            match goa_sso::lnxdrive_goa_account_path().await {
+                Ok(Some(account_path)) => {
+                    match dbus_client.complete_auth_via_goa(&account_path).await {
                         Ok(true) => {
                             // Fetch account info and push folder page
                             if let Ok(info) = dbus_client.get_account_info().await {
@@ -379,24 +378,26 @@ impl AuthPage {
                         }
                         Ok(false) => {
                             page.show_error(&gettext(
-                                "The daemon rejected the GOA tokens. Try signing in manually.",
+                                "The daemon rejected the GOA account. Try signing in manually.",
                             ));
                             page.set_waiting_state(false, &wl);
                         }
                         Err(e) => {
-                            page.show_error(&format!(
-                                "{}: {}",
-                                gettext("D-Bus error"),
-                                e
-                            ));
+                            page.show_error(&format!("{}: {}", gettext("D-Bus error"), e));
                             page.set_waiting_state(false, &wl);
                         }
                     }
                 }
+                Ok(None) => {
+                    page.show_error(&gettext(
+                        "No GNOME Online Accounts account found for LNXDrive.",
+                    ));
+                    page.set_waiting_state(false, &wl);
+                }
                 Err(e) => {
                     page.show_error(&format!(
                         "{}: {}",
-                        gettext("Could not get GOA tokens"),
+                        gettext("Could not query GNOME Online Accounts"),
                         e
                     ));
                     page.set_waiting_state(false, &wl);
diff --git a/lnxdrive-gnome/preferences/src/onboarding/confirm_page.rs b/lnxdrive-gnome/preferences/src/onboarding/confirm_page.rs
index 57e81b8..5c45c21 100644
--- a/lnxdrive-gnome/preferences/src/onboarding/confirm_page.rs
+++ b/lnxdrive-gnome/preferences/src/onboarding/confirm_page.rs
@@ -84,17 +84,19 @@ impl ConfirmPage {
             .clone()
             .unwrap_or_else(|| gettext("Not selected"));
 
+        // `ActionRow::icon_name` is deprecated since libadwaita 1.3; add the icon
+        // as a prefix widget instead.
         let email_row = adw::ActionRow::builder()
-            .title(&gettext("Account"))
+            .title(gettext("Account"))
             .subtitle(&account_email)
-            .icon_name("avatar-default-symbolic")
             .build();
+        email_row.add_prefix(&gtk4::Image::from_icon_name("avatar-default-symbolic"));
 
         let folder_row = adw::ActionRow::builder()
-            .title(&gettext("Sync Folder"))
+            .title(gettext("Sync Folder"))
             .subtitle(&sync_folder)
-            .icon_name("folder-symbolic")
             .build();
+        folder_row.add_prefix(&gtk4::Image::from_icon_name("folder-symbolic"));
 
         let summary_group = adw::PreferencesGroup::new();
         summary_group.add(&email_row);
@@ -102,7 +104,7 @@ impl ConfirmPage {
 
         // "Start Syncing" button
         let start_button = gtk4::Button::builder()
-            .label(&gettext("Start Syncing"))
+            .label(gettext("Start Syncing"))
             .halign(gtk4::Align::Center)
             .css_classes(["suggested-action", "pill"])
             .build();
@@ -118,8 +120,8 @@ impl ConfirmPage {
         // Status page with check icon
         let status_page = adw::StatusPage::builder()
             .icon_name("emblem-ok-symbolic")
-            .title(&gettext("All Set!"))
-            .description(&gettext(
+            .title(gettext("All Set!"))
+            .description(gettext(
                 "Your OneDrive account is ready. Review the details below and start syncing.",
             ))
             .build();
diff --git a/lnxdrive-gnome/preferences/src/onboarding/folder_page.rs b/lnxdrive-gnome/preferences/src/onboarding/folder_page.rs
index cf0627a..cbae7ef 100644
--- a/lnxdrive-gnome/preferences/src/onboarding/folder_page.rs
+++ b/lnxdrive-gnome/preferences/src/onboarding/folder_page.rs
@@ -79,14 +79,14 @@ impl FolderPage {
         // Path display row
         let initial_path = imp.selected_path.borrow().display().to_string();
         let path_row = adw::ActionRow::builder()
-            .title(&gettext("Sync Folder"))
+            .title(gettext("Sync Folder"))
             .subtitle(&initial_path)
             .build();
 
         // "Choose Folder..." button as a suffix
         let choose_button = gtk4::Button::builder()
             .icon_name("folder-open-symbolic")
-            .tooltip_text(&gettext("Choose Folder..."))
+            .tooltip_text(gettext("Choose Folder..."))
             .valign(gtk4::Align::Center)
             .css_classes(["flat"])
             .build();
@@ -96,8 +96,8 @@ impl FolderPage {
         imp.path_row.replace(Some(path_row.clone()));
 
         let prefs_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Sync Location"))
-            .description(&gettext(
+            .title(gettext("Sync Location"))
+            .description(gettext(
                 "Choose where OneDrive files will be stored on your computer.",
             ))
             .build();
@@ -105,7 +105,7 @@ impl FolderPage {
 
         // Action buttons
         let continue_button = gtk4::Button::builder()
-            .label(&gettext("Continue"))
+            .label(gettext("Continue"))
             .halign(gtk4::Align::Center)
             .css_classes(["suggested-action", "pill"])
             .build();
@@ -159,7 +159,7 @@ impl FolderPage {
     /// Open a folder chooser dialog.
     fn on_choose_folder(&self) {
         let dialog = gtk4::FileDialog::builder()
-            .title(&gettext("Choose Sync Folder"))
+            .title(gettext("Choose Sync Folder"))
             .modal(true)
             .build();
 
diff --git a/lnxdrive-gnome/preferences/src/onboarding/mod.rs b/lnxdrive-gnome/preferences/src/onboarding/mod.rs
index 21bd452..3b799bf 100644
--- a/lnxdrive-gnome/preferences/src/onboarding/mod.rs
+++ b/lnxdrive-gnome/preferences/src/onboarding/mod.rs
@@ -14,7 +14,6 @@ pub mod folder_page;
 use std::cell::RefCell;
 
 use gtk4::glib;
-use gtk4::prelude::*;
 use libadwaita as adw;
 use libadwaita::prelude::*;
 
diff --git a/lnxdrive-gnome/preferences/src/preferences/account_page.rs b/lnxdrive-gnome/preferences/src/preferences/account_page.rs
index fc4ba1e..e48639e 100644
--- a/lnxdrive-gnome/preferences/src/preferences/account_page.rs
+++ b/lnxdrive-gnome/preferences/src/preferences/account_page.rs
@@ -6,6 +6,7 @@
 
 use std::cell::RefCell;
 
+use futures_util::StreamExt;
 use gettextrs::gettext;
 use gtk4::glib;
 use gtk4::prelude::*;
@@ -14,7 +15,7 @@ use libadwaita::prelude::*;
 
 use gtk4::subclass::prelude::ObjectSubclassIsExt;
 
-use crate::dbus_client::DbusClient;
+use crate::dbus_client::{DbusClient, LnxdriveStatusProxy};
 
 // ---------------------------------------------------------------------------
 // AccountPage — adw::PreferencesPage subclass
@@ -77,28 +78,52 @@ impl AccountPage {
         page.build_ui();
         page.load_account_info();
         page.load_quota();
+        page.subscribe_quota_changes();
 
         page
     }
 
+    /// Keep the quota display live by listening for the daemon's `QuotaChanged`
+    /// signal, instead of only reading it once at construction.
+    fn subscribe_quota_changes(&self) {
+        let client = match self.imp().dbus_client.borrow().clone() {
+            Some(c) => c,
+            None => return,
+        };
+
+        let page = self.clone();
+        glib::MainContext::default().spawn_local(async move {
+            let conn = client.connection().clone();
+            if let Ok(proxy) = LnxdriveStatusProxy::new(&conn).await {
+                if let Ok(mut stream) = proxy.receive_quota_changed().await {
+                    while let Some(signal) = stream.next().await {
+                        if let Ok(args) = signal.args() {
+                            page.update_quota_display(args.used, args.total);
+                        }
+                    }
+                }
+            }
+        });
+    }
+
     fn build_ui(&self) {
         let imp = self.imp();
 
         // -- OneDrive Account group ------------------------------------------
 
         let account_group = adw::PreferencesGroup::builder()
-            .title(&gettext("OneDrive Account"))
+            .title(gettext("OneDrive Account"))
             .build();
 
         let email_row = adw::ActionRow::builder()
-            .title(&gettext("Email"))
-            .subtitle(&gettext("Loading..."))
+            .title(gettext("Email"))
+            .subtitle(gettext("Loading..."))
             .build();
         imp.email_row.replace(Some(email_row.clone()));
 
         let name_row = adw::ActionRow::builder()
-            .title(&gettext("Display Name"))
-            .subtitle(&gettext("Loading..."))
+            .title(gettext("Display Name"))
+            .subtitle(gettext("Loading..."))
             .build();
         imp.name_row.replace(Some(name_row.clone()));
 
@@ -108,7 +133,7 @@ impl AccountPage {
         // -- Storage group ---------------------------------------------------
 
         let storage_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Storage"))
+            .title(gettext("Storage"))
             .build();
 
         let level_bar = gtk4::LevelBar::builder()
@@ -123,7 +148,7 @@ impl AccountPage {
         imp.level_bar.replace(Some(level_bar.clone()));
 
         let quota_label = gtk4::Label::builder()
-            .label(&gettext("Loading storage info..."))
+            .label(gettext("Loading storage info..."))
             .css_classes(["dim-label", "caption"])
             .margin_start(12)
             .margin_end(12)
@@ -151,11 +176,11 @@ impl AccountPage {
         // -- Session group ---------------------------------------------------
 
         let session_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Session"))
+            .title(gettext("Session"))
             .build();
 
         let sign_out_button = gtk4::Button::builder()
-            .label(&gettext("Sign Out"))
+            .label(gettext("Sign Out"))
             .halign(gtk4::Align::Center)
             .css_classes(["destructive-action", "pill"])
             .margin_top(8)
@@ -279,8 +304,8 @@ impl AccountPage {
     fn on_sign_out(&self) {
         // Create a confirmation dialog.
         let confirm = adw::AlertDialog::builder()
-            .heading(&gettext("Sign Out?"))
-            .body(&gettext(
+            .heading(gettext("Sign Out?"))
+            .body(gettext(
                 "You will be signed out of your OneDrive account. Syncing will stop.",
             ))
             .build();
diff --git a/lnxdrive-gnome/preferences/src/preferences/advanced_page.rs b/lnxdrive-gnome/preferences/src/preferences/advanced_page.rs
index 0091919..ff84551 100644
--- a/lnxdrive-gnome/preferences/src/preferences/advanced_page.rs
+++ b/lnxdrive-gnome/preferences/src/preferences/advanced_page.rs
@@ -91,8 +91,8 @@ impl AdvancedPage {
         // -- Exclusion Patterns group (FR-015) --------------------------------
 
         let patterns_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Exclusion Patterns"))
-            .description(&gettext(
+            .title(gettext("Exclusion Patterns"))
+            .description(gettext(
                 "Files and folders matching these glob patterns will not be synced.",
             ))
             .build();
@@ -119,13 +119,13 @@ impl AdvancedPage {
             .build();
 
         let entry = gtk4::Entry::builder()
-            .placeholder_text(&gettext("e.g. *.tmp, .git/, ~$*"))
+            .placeholder_text(gettext("e.g. *.tmp, .git/, ~$*"))
             .hexpand(true)
             .build();
         imp.pattern_entry.replace(Some(entry.clone()));
 
         let add_button = gtk4::Button::builder()
-            .label(&gettext("Add"))
+            .label(gettext("Add"))
             .css_classes(["suggested-action"])
             .build();
 
@@ -154,8 +154,8 @@ impl AdvancedPage {
         // -- Bandwidth Limits group (FR-017) ----------------------------------
 
         let bandwidth_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Bandwidth Limits"))
-            .description(&gettext(
+            .title(gettext("Bandwidth Limits"))
+            .description(gettext(
                 "Limit upload and download speeds. Set to 0 for unlimited.",
             ))
             .build();
@@ -245,7 +245,7 @@ impl AdvancedPage {
 
         let delete_button = gtk4::Button::builder()
             .icon_name("edit-delete-symbolic")
-            .tooltip_text(&gettext("Remove pattern"))
+            .tooltip_text(gettext("Remove pattern"))
             .valign(gtk4::Align::Center)
             .css_classes(["flat", "circular"])
             .build();
diff --git a/lnxdrive-gnome/preferences/src/preferences/folder_tree.rs b/lnxdrive-gnome/preferences/src/preferences/folder_tree.rs
index d51257d..9ffbb31 100644
--- a/lnxdrive-gnome/preferences/src/preferences/folder_tree.rs
+++ b/lnxdrive-gnome/preferences/src/preferences/folder_tree.rs
@@ -10,6 +10,7 @@
 
 use std::cell::RefCell;
 
+use gettextrs::gettext;
 use gtk4::gio;
 use gtk4::glib;
 use gtk4::prelude::*;
@@ -202,8 +203,7 @@ impl FolderTree {
         }
 
         tree.build_ui();
-        tree.load_remote_tree();
-        tree.load_selected_folders();
+        tree.load_tree_and_selections();
 
         tree
     }
@@ -357,29 +357,14 @@ impl FolderTree {
         self.append(&scrolled);
     }
 
-    /// Fetch the remote folder tree JSON from the daemon and populate the root store.
-    fn load_remote_tree(&self) {
-        let client = match self.imp().dbus_client.borrow().clone() {
-            Some(c) => c,
-            None => return,
-        };
-
-        let tree = self.clone();
-        glib::MainContext::default().spawn_local(async move {
-            match client.get_remote_folder_tree().await {
-                Ok(json) => {
-                    tree.populate_from_json(&json);
-                }
-                Err(e) => {
-                    eprintln!("Could not load remote folder tree: {}", e);
-                }
-            }
-        });
-    }
-
-    /// Load the currently selected folders from the daemon so we can mark
-    /// them as checked.
-    fn load_selected_folders(&self) {
+    /// Load the selected folders and the remote tree in one ordered task.
+    ///
+    /// Selections are fetched *first* so `populate_from_json` can mark nodes with
+    /// the correct checked state as it builds them. Doing these as two
+    /// independent `spawn_local` tasks (as before) raced: selections could be
+    /// applied to a still-empty tree, or the tree populated before selections
+    /// arrived, leaving nothing checked.
+    fn load_tree_and_selections(&self) {
         let client = match self.imp().dbus_client.borrow().clone() {
             Some(c) => c,
             None => return,
@@ -387,15 +372,21 @@ impl FolderTree {
 
         let tree = self.clone();
         glib::MainContext::default().spawn_local(async move {
+            // 1. Selections first.
             match client.get_selected_folders().await {
-                Ok(folders) => {
-                    *tree.imp().selected_folders.borrow_mut() = folders;
-                    // Re-apply selections after the tree has been populated.
-                    tree.apply_selections();
-                }
-                Err(e) => {
-                    eprintln!("Could not load selected folders: {}", e);
-                }
+                Ok(folders) => *tree.imp().selected_folders.borrow_mut() = folders,
+                Err(e) => tree.show_error(&format!(
+                    "{}: {e}",
+                    gettext("Could not load selected folders")
+                )),
+            }
+            // 2. Then the tree, which reads the selections set above.
+            match client.get_remote_folder_tree().await {
+                Ok(json) => tree.populate_from_json(&json),
+                Err(e) => tree.show_error(&format!(
+                    "{}: {e}",
+                    gettext("Could not load the folder list")
+                )),
             }
         });
     }
@@ -409,18 +400,29 @@ impl FolderTree {
             None => return,
         };
 
-        root_store.remove_all();
-
-        // The JSON may be a single root object or an array of roots.
+        // The JSON may be a single root object or an array of roots. A parse
+        // failure is surfaced as an error rather than silently rendering an
+        // empty tree (which is indistinguishable from "no folders").
         let nodes: Vec<FolderNodeJson> = if json.trim_start().starts_with('[') {
-            serde_json::from_str(json).unwrap_or_default()
+            match serde_json::from_str(json) {
+                Ok(n) => n,
+                Err(e) => {
+                    self.show_error(&format!("{}: {e}", gettext("Invalid folder list")));
+                    return;
+                }
+            }
         } else {
             match serde_json::from_str::<FolderNodeJson>(json) {
                 Ok(root) => root.children,
-                Err(_) => Vec::new(),
+                Err(e) => {
+                    self.show_error(&format!("{}: {e}", gettext("Invalid folder list")));
+                    return;
+                }
             }
         };
 
+        root_store.remove_all();
+
         let selected = imp.selected_folders.borrow().clone();
         for node in &nodes {
             let is_selected = selected.iter().any(|p| p == &node.path);
@@ -430,23 +432,16 @@ impl FolderTree {
         }
     }
 
-    /// Walk the root store and mark nodes whose path is in the selected list.
-    fn apply_selections(&self) {
-        let imp = self.imp();
-        let store = match imp.root_store.borrow().clone() {
-            Some(s) => s,
-            None => return,
-        };
-        let selected = imp.selected_folders.borrow().clone();
-
-        for i in 0..store.n_items() {
-            if let Some(item) = store.item(i) {
-                if let Some(node) = item.downcast_ref::<FolderNode>() {
-                    let is_selected = selected.iter().any(|p| p == &node.path());
-                    node.set_selected(is_selected);
-                }
-            }
-        }
+    /// Show a load/parse error inline at the top of the widget, instead of
+    /// failing silently to stderr (which left the tree looking empty).
+    fn show_error(&self, message: &str) {
+        let label = gtk4::Label::builder()
+            .label(message)
+            .wrap(true)
+            .xalign(0.0)
+            .css_classes(["error"])
+            .build();
+        self.prepend(&label);
     }
 
     /// Called whenever a checkbox is toggled. Propagates the selection to
diff --git a/lnxdrive-gnome/preferences/src/preferences/sync_page.rs b/lnxdrive-gnome/preferences/src/preferences/sync_page.rs
index cf70a86..7f3ef36 100644
--- a/lnxdrive-gnome/preferences/src/preferences/sync_page.rs
+++ b/lnxdrive-gnome/preferences/src/preferences/sync_page.rs
@@ -8,7 +8,6 @@ use std::cell::RefCell;
 
 use gettextrs::gettext;
 use gtk4::glib;
-use gtk4::prelude::*;
 use libadwaita as adw;
 use libadwaita::prelude::*;
 
@@ -101,13 +100,13 @@ impl SyncPage {
         // -- Sync Options group ----------------------------------------------
 
         let options_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Sync Options"))
+            .title(gettext("Sync Options"))
             .build();
 
         // Automatic Sync switch (FR-018)
         let auto_sync_row = adw::SwitchRow::builder()
-            .title(&gettext("Automatic Sync"))
-            .subtitle(&gettext("Sync files automatically when changes are detected"))
+            .title(gettext("Automatic Sync"))
+            .subtitle(gettext("Sync files automatically when changes are detected"))
             .build();
         imp.auto_sync_row.replace(Some(auto_sync_row.clone()));
 
@@ -123,8 +122,8 @@ impl SyncPage {
         );
 
         let conflict_row = adw::ComboRow::builder()
-            .title(&gettext("Conflict Resolution"))
-            .subtitle(&gettext("How to handle file conflicts between local and remote"))
+            .title(gettext("Conflict Resolution"))
+            .subtitle(gettext("How to handle file conflicts between local and remote"))
             .model(&conflict_model)
             .build();
         imp.conflict_row.replace(Some(conflict_row.clone()));
@@ -144,8 +143,8 @@ impl SyncPage {
         // -- Selective Sync group (FR-014) ------------------------------------
 
         let selective_group = adw::PreferencesGroup::builder()
-            .title(&gettext("Selective Sync"))
-            .description(&gettext(
+            .title(gettext("Selective Sync"))
+            .description(gettext(
                 "Choose which remote folders to sync to this computer.",
             ))
             .build();
@@ -197,12 +196,28 @@ impl SyncPage {
                     page.apply_config_yaml(&yaml);
                 }
                 Err(e) => {
-                    eprintln!("Could not load config: {}", e);
+                    page.show_error(&format!(
+                        "{}: {e}",
+                        gettext("Could not load sync settings from the daemon")
+                    ));
                 }
             }
         });
     }
 
+    /// Surface a load/save failure to the user instead of only logging to stderr,
+    /// so a dead daemon does not leave the page silently showing default values.
+    fn show_error(&self, message: &str) {
+        let group = adw::PreferencesGroup::new();
+        let row = adw::ActionRow::builder()
+            .title(gettext("Error"))
+            .subtitle(message)
+            .css_classes(["error"])
+            .build();
+        group.add(&row);
+        self.add(&group);
+    }
+
     /// Parse the daemon's YAML config and apply values to the UI widgets.
     /// We do simple line-based parsing to avoid pulling in a full YAML crate
     /// beyond serde (the config is flat key-value).
diff --git a/lnxdrive-gnome/preferences/src/window.rs b/lnxdrive-gnome/preferences/src/window.rs
index e271110..c558d19 100644
--- a/lnxdrive-gnome/preferences/src/window.rs
+++ b/lnxdrive-gnome/preferences/src/window.rs
@@ -100,13 +100,13 @@ impl LnxdriveWindow {
         // Set up window content behind the dialog.
         let status = adw::StatusPage::builder()
             .icon_name("emblem-ok-symbolic")
-            .title(&gettext("LNXDrive"))
-            .description(&gettext("Your OneDrive files are syncing."))
+            .title(gettext("LNXDrive"))
+            .description(gettext("Your OneDrive files are syncing."))
             .build();
 
         // Add a button to re-open preferences if the dialog is closed.
         let open_prefs_button = gtk4::Button::builder()
-            .label(&gettext("Preferences"))
+            .label(gettext("Preferences"))
             .halign(gtk4::Align::Center)
             .css_classes(["pill"])
             .build();
@@ -135,7 +135,7 @@ impl LnxdriveWindow {
     pub fn show_dbus_error(&self, message: &str) {
         let status = adw::StatusPage::builder()
             .icon_name("dialog-error-symbolic")
-            .title(&gettext("Cannot Connect to LNXDrive"))
+            .title(gettext("Cannot Connect to LNXDrive"))
             .description(message)
             .build();
 

```

---

## What you must do

### Step 1 — Read the scope

Read the Charter file at `.straymark/charters/01-road-to-v0-1-0-alpha-1.md` in full. Identify:

- The `## Tasks` section (or equivalent): each task, its description, and the expected file.
- The `## Files to modify` section: table of files and declared change type.
- The `## Risk` section or equivalent: `R<N>` risks consciously accepted.
- The Charter's closure criterion (what makes it "complete").

### Step 2 — Verify each task (MANDATORY)

For EACH task in the Charter, perform these steps in order:

1. **Locate file(s)**: find the file mentioned in the task. If it does not exist, report as "Not found". If it exists, continue.
2. **Read the full implementation**: read the file entirely, not just the name. **Do not report "file exists" without reading its content.**
3. **Trace execution flow**: for key functions, follow the full chain (handler → service → repository → SQL/storage, or the equivalent in the project's stack). Verify that parameters propagate correctly through each layer.
4. **Verify tests**: locate the corresponding tests. Read at least 2 test cases to confirm they cover the happy path and at least one edge case.
5. **Compare against the task**: does the implementation match what the task describes? If there are discrepancies, report with evidence (`file:line`).

> **Evidence discipline.** You may only opine on files you have opened via a tool call (Read, Grep, etc.). Any finding you produce must cite `file:line` of the specific files you opened. Findings without citations are treated as low confidence by the consolidated review and may be dropped. If you did not open a file, you cannot infer behavior, structure, or correctness about it.

### Step 3 — Run verifications (when applicable)

If your environment allows you to run project commands (build, lint, test), run them over the Charter's scope and report the output verbatim. **Read/verify commands only** — never generators or mutating commands.

> *Stack examples* (adapt to the project you are auditing):
> - **Go**: `go vet ./...`, `go build ./...`, `go test ./<module>/... -v -count=1 2>&1 | tail -50`
> - **Rust**: `cargo check`, `cargo clippy --all-targets`, `cargo test --no-run`
> - **TypeScript/Node**: `npm run typecheck`, `npm run lint`, `npm test -- --run`
> - **Python**: `mypy <pkg>`, `ruff check`, `pytest --co`

If your environment does NOT allow command execution, skip this step and focus the audit on static reading of code + tests.

### Step 4 — Evaluate Charter closure

Read the closure criterion declared by the Charter. Assess: **is this criterion met by the current implementation?** The Charter's criterion is the source of truth for "complete or not", not your expectation of what it "should" include.

### Step 5 — Calibrate severity against the project's REAL configuration

Before assigning severity to EACH finding, verify the driver, flag, or configuration actually active in the code, NOT the theoretical worst case.

**Rule:** severity must reflect the impact the finding has with the configuration the project uses TODAY, not the impact it would have under a hypothetical configuration.

**Mandatory checks before declaring Critical or High severity:**

- [ ] **Active driver**: if the finding concerns an event bus, cache, storage, queue, or any pluggable component, open the factory/config (typically something like `internal/core/<component>/factory.go`, `src/<component>/factory.ts`, `.env.example`, `config.yml`) and confirm which driver is actually instantiated.
- [ ] **Feature flags**: if the code has conditional branches keyed on an env var or flag, confirm the default value and the value used in the tests you validated. A bug that only triggers with `FEATURE_X=true` when the default is `false` is not Critical — it is conditional.
- [ ] **Build tags / conditional compilation**: if the code is behind `//go:build foo`, `#[cfg(feature = "foo")]`, `process.env.NODE_ENV !== 'production'`, etc., confirm whether that condition holds in the production build. Defects reproducible only under a dev or test tag are not production blockers.
- [ ] **DB role / user**: if the finding touches RLS, SQL permissions, or ACLs, verify under which role the app runs. (For example, the testcontainers superuser bypasses RLS; the production role may differ. Do not confuse test behavior with production behavior.)
- [ ] **Deployment scope**: if the finding concerns concurrency, distributed cache, or multi-instance coordination, confirm the configured scaling (`maxScale`, replicas, etc.). A race-condition bug between instances is not Critical if the deployment runs with `maxScale=1`.

**How to classify when the finding is CONDITIONAL:**

- **Critical / High**: the bug triggers under the configuration that runs TODAY in main or staging.
- **Medium / Low**: the bug is a real smell but has no operational trigger under the current config.
- **Post-Charter / non-blocking**: the bug is real and critical under a component that does not yet exist (e.g., an external service still stubbed), or under a flag explicitly disabled. Document it as a future concern with a clear note of "when" and "why" — NOT as a blocker for this Charter.

**Anti-inflation rule:** you may not justify Critical severity by appealing solely to "the bug EXISTS in the code". You must demonstrate that **running** the application with its current configuration, the bug would actually manifest. If your justification begins with "if in the future X were implemented..." or "if someone enabled flag Y...", your severity must be post-Charter or Medium with a note, not Critical.

**Anti-deflation rule:** conversely, you may not classify something as Low by appealing to "this never happens in practice" if the code has a clear path that triggers it under the current config. The absence of reported incidents is not evidence of the bug's absence.

> **Example — declared deferral, not a defect.** Suppose Charter N introduces a thin in-memory adapter for a service the project plans to back with a real driver in a future Charter (call it Charter N+K). Charter N's `## Risk` section names the deferral explicitly (for example: *"R1: temporary in-memory adapter, replaced in CHARTER-N+K"*). If an auditor reading Charter N opens the component's factory and finds that the active driver is the in-memory adapter rather than the real implementation, they must **NOT** report this as a Critical finding — the deferral is declared scope, not hidden technical debt. Correct calibration requires opening the factory and verifying the active driver *before* declaring high severity; if the result matches a deferral declared in some Charter (this one or a previous one), the finding is at most *Post-Charter / non-blocking*. Conversely, if the same auditor finds another place where the same pattern was repeated **without** a declared deferral in any Charter, that **is** a finding (debt without an owner).

---

## Finding categorization

Each finding falls into one of these four categories. The consolidated review uses the same definitions:

- **`hallucination`** — the Charter or the implementation references something that does not exist (an API, a function, a field, a behavior). The agent invented it. Verify by opening the actual file or API.
- **`implementation_gap`** — the Charter declared work the diff did not deliver, OR the diff delivered work the Charter did not declare, **without** being documented as a risk in the AILOG. (If it is documented in `## Risk` as `R<N+1>` in some AILOG, that is NOT a gap — it is an accepted trade-off.)
- **`real_debt`** — a code-level concern that is correct with respect to the Charter but introduces technical debt or a subtle defect (a missing error path, a leaked resource, a non-idempotent operation). The adopter should capture this as a post-audit TDE doc.
- **`false_positive`** — what initially looked like a finding but, on closer inspection of the AILOG or the diff, is not. Document it anyway; the consolidated review uses these to recognize patterns where one auditor over-reports.

---

## Output format

Document your findings in a markdown file. The canonical output path is decided by the flow:

- In auditor-side CLI mode (skill `straymark-audit-execute`): `.straymark/audits/CHARTER-01-road-to-v0-1-0-alpha-1/report-<sluggified-model-id>.md` (the skill handles the path automatically).
- In manual paste mode (transitional v0): the operator saves your output at `audit/charters/CHARTER-01-road-to-v0-1-0-alpha-1/auditor-auditor.md` or an equivalent convention.

The file must have this frontmatter (validated against `.straymark/schemas/audit-output.schema.v0.json`):

```yaml
---
audit_role: auditor                       # v1 unified. Legacy v0: "auditor-primary" or "auditor-secondary"
auditor: <your model id and version>      # e.g., claude-sonnet-4-6, gemini-2.5-pro, copilot-v1.0.40
charter_id: CHARTER-01-road-to-v0-1-0-alpha-1
git_range: "origin/main..HEAD"
prompt_used: <path to the resolved audit-prompt you received>
audited_at: <today YYYY-MM-DD>
findings_total: <N>
findings_by_category:
  hallucination: <N>
  implementation_gap: <N>
  real_debt: <N>
  false_positive: <N>
evidence_citations: <N>                   # optional but recommended: how many file:line citations you made
audit_quality: high|medium|low            # optional, self-assessment
---

# Audit: CHARTER-01-road-to-v0-1-0-alpha-1 by <your model id>

## Executive summary

[1-2 paragraphs: did execution match the Charter's declared scope? What is the overall verdict — clean, partial, drifted? What is the most material finding, if any?]

## Compilation and test verification

[Paste the output of the Step 3 commands here, if you ran them. If not, state "(skipped — no command execution available)".]

## Task-by-task traceability

For EACH task in the Charter, one entry with this format:

### T### — [Task description]

- **File(s)**: `path/to/file.ext:lines`
- **Status**: Implemented | Partial | Not implemented
- **Verification**:
  - Implementation read: Yes/No
  - Flow traced: [handler → service → repository → SQL] (or equivalent)
  - Tests found: [test_file.ext, N test cases]
- **Findings**: [None | Description of the finding with `file:line`]

## Findings

Classified by severity. ONLY findings within the Charter's scope.

### Critical (block Charter closure)

| # | Finding | File:Line | Category | Evidence | Suggested remediation |
|---|---------|-----------|----------|----------|----------------------|

### High (security or logic bugs)

| # | Finding | File:Line | Category | Evidence | Suggested remediation |
|---|---------|-----------|----------|----------|----------------------|

### Medium (inconsistencies, minor risks)

| # | Finding | File:Line | Category | Evidence | Suggested remediation |
|---|---------|-----------|----------|----------|----------------------|

### Low (quality, naming, style improvements)

| # | Finding | File:Line | Category | Evidence | Suggested remediation |
|---|---------|-----------|----------|----------|----------------------|

## Out-of-scope notes (optional)

Observations about code that is NOT part of this Charter's scope but that you consider relevant to mention. These are NOT defects of this Charter.

| Observation | Relevant Charter / area | Note |
|-------------|-------------------------|------|

## Charter closure assessment

Does the implementation meet the closure criterion declared by `CHARTER-01-road-to-v0-1-0-alpha-1`?
[Yes / No / Partial] — [Justification grounded in evidence, citing `file:line`]

## Conclusion

[2-3 sentences. Actual state of the Charter, critical findings if any, recommended next step.]
```

---

## What you must NOT do

- **DO NOT MODIFY ANY PROJECT FILE.** Your only allowed output is the audit report. If you modify any other file, your audit will be discarded and considered invalid. This includes "fixing" bugs, "improving" code, creating missing files, or running generators. **REPORT, DO NOT ACT.** This is not optional or contextual — it is an absolute constraint.
- **DO NOT declare "no issues"** without having read the code of every task declared in the Charter.
- **DO NOT report tasks from other Charters** as defects of this one.
- **DO NOT inflate severity**: a finding from another Charter is not "Critical" here.
- **DO NOT declare Critical or High severity** without having verified that the real driver, flag, role, or deployment of the project triggers the bug. See Step 5. Declaring "critical regression" based on a stubbed component or a disabled flag invalidates the audit through false inflation.
- **DO NOT report** that a file "does not exist" without having searched with the correct path (including naming-convention variants used by the project).
- **DO NOT copy the file structure** without verifying content.
- **DO NOT ignore** the prior-audits folders (typically `audit/` or `.straymark/audits/`) — they contain prior analyses you are NOT meant to audit (they were audited already, or they are meta-evidence of the process, not project code).
- **DO NOT run** destructive or generative commands. Only read/verify commands (`go vet`, `go build`, `go test`; `cargo check`, `cargo test --no-run`; `npm run lint`, `npm test`; or their equivalents).
- **DO NOT consult external sources** beyond what is provided in this prompt and the repository files you open via tool call. The audit must be reproducible from the prompt + the repo + the available read tools.

---

*StrayMark unified audit template v1. The seven universal sections (ABSOLUTE RULE, Your role, Scope rules, Step 2 mandatory verification, Step 5 severity calibration, What you must NOT do, Output format) come from the `audit/SKILL.md` skill mature pre-StrayMark in Sentinel, contributed via issue #102 by José Villaseñor Montfort (StrangeDaysTech). Sentinel-specific hardcodes (spec paths, Etapa headings, internal modules) were parameterized against the Charter doc, originating AILOGs, git range, and project context.*
