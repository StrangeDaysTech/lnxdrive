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
- Reading, opening, grepping, or referencing **another auditor's report** (`report-*.md`, `auditor-*.md`, or any scratch file) under `.straymark/audits/` — for this Charter or any other. Your audit must be **independent**: an audit that reads, cites, summarizes, or "cross-verifies against" another auditor's report is contaminated and will be discarded. Cross-auditor convergence is signal ONLY when each auditor reached it *without* seeing the others — a copied agreement is worthless.

The ONLY thing you may write is your audit report file at the canonical path shown in **Output format** below. That is the ONLY file you have permission to create.

If you find a bug, **DOCUMENT IT** in your report. Do NOT fix it.
If you find a missing file, **REPORT IT**. Do NOT create it.
If a test fails, **REPORT IT**. Do NOT repair it.

**Violating this rule invalidates the entire audit.**

---

## Output contract (read this first)

You are about to read a lot — the Charter, the originating AILOGs, the diff — before you reach the full **Output format** at the very end of this prompt. Lock these invariants in now, so the long read does not pull your report toward the wrong shape:

1. **You write exactly one file**: your audit report, at the canonical path in **Output format**. Nothing else (see the ABSOLUTE RULE).
2. **Required report frontmatter** (validated against `.straymark/schemas/audit-output.schema.v0.json`): `audit_role`, `auditor`, `charter_id`, `git_range`, `prompt_used`, `audited_at`, `findings_total`, `findings_by_category` — where `findings_by_category` has exactly the four keys `hallucination`, `implementation_gap`, `real_debt`, `false_positive`. `evidence_citations` and `audit_quality` are optional but recommended.
3. **The four finding categories** (`hallucination`, `implementation_gap`, `real_debt`, `false_positive`) are defined under **Finding categorization** below — *before* the point where you must assign them.
4. **⚠️ Your report frontmatter is DELIBERATELY DIFFERENT from the AILOG/AIDEC frontmatter you are about to read.** The AILOGs embedded below use keys like `id` / `status` / `confidence` / `risk_level` / `agent`. Your report does **not** — it uses the audit keys in (2). Do not mimic the surrounding documents; follow the schema.

This is a summary. The authoritative, complete format (frontmatter + every body section) is in **Output format** at the end of this prompt — write your report against that, not against this digest.

---

## Your role

You are an independent code auditor. Your job is to verify that the implementation of a specific Charter fulfills the declared tasks and files, find real bugs in the code, and identify security risks. **You are NOT a cheerleader** — reporting "no issues" when bugs exist is worse than reporting a false positive.

StrayMark orchestrates cross-model audits: another auditor from a **different model family** reviews the same Charter — sometimes alongside you, sometimes before you, so their `report-*.md` may already sit in `.straymark/audits/CHARTER-01-road-to-v0-1-0-alpha-1/`. **You must not read it** (see the ABSOLUTE RULE). Your value lies in *independent* evidence discipline (citing `file:line` of files you actually opened) and severity calibration against the real config — not in converging with, or even glancing at, another auditor's report. An agreement you reached by reading theirs is not convergence; it is contamination.

---

## Project



*(The operator may fill this placeholder with a brief description of the project's stack and architecture if they want to give the auditor extra context. If empty, the auditor infers the stack from the diff and the referenced files.)*

---

## STRICT scope

**Charter under audit:** `CHARTER-01-road-to-v0-1-0-alpha-1` — Road to v0.1.0-alpha.1
**Charter file:** `.straymark/charters/01-road-to-v0-1-0-alpha-1.md`
**Git range:** `31482c7..ae5a27d`

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

> **Frontmatter note.** These AILOGs carry their own frontmatter (`id`, `status`, `confidence`, `risk_level`, `agent`). That is **not** the shape of your audit report — your report uses the audit schema in **Output format**. Read the AILOGs for their content; do not let their frontmatter become a template for yours.

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

## Batch Ledger

> Backfilled on 2026-06-04 (Fase 5 PR): the Charter §Tasks mandated
> `straymark charter batch-complete` after each phase merge, but this section
> was never scaffolded at declaration time, so Fases 0–4 are recorded
> retroactively from the merge history (process drift, documented in
> AILOG-2026-06-04-002). Batch N = Fase N−1.

### Batch 1 — Fase 0: governance foundation + public backlog

Merged 2026-05-28 — PR #4 (declare Charter-01, archive non-MVP UIs, governance foundation). Part 2 (GitHub-side): 24 backlog issues created from risk-analysis on 2026-05-28; GitHub Project later removed as redundant (see CLAUDE.md §3).

### Batch 2 — Fase 1: P0 risk mitigation + CI hardening

Merged 2026-05-28→30 — PRs #32 (RISK-002 OAuth off D-Bus), #33 (RISK-003 FUSE write-during-hydration), #35 (RISK-001 D-Bus health monitor), #36 (ISSUE-002 YAML billion-laughs), #34 (config path fix), #39 (CI hardening + external audit, lands #37/#38), #40 (audit drafts). External pre-merge audit consolidated.

### Batch 3 — Fase 2: engine polish (T101 + FUSE listing repair)

Merged 2026-05-31 — PR #41: T101 closed via real-mount integration test (getattr 43.7µs, readdir 1.40ms/1000, RSS 37.9MB/10k); 4 FUSE listing bugs + inode-persistence defect fixed. See AILOG-2026-05-31-001.

### Batch 4 — Fase 3: GTK4 preferences panel audit + remediation

Merged 2026-05-31 — PR #42: panel audit (6 findings) + remediation; H1 RISK-002 drift fixed (CompleteAuthViaGOA); G1 System group deferred (AIDEC-2026-05-31-001). E2E verified in Nivel-5 VM. See AILOG-2026-05-31-002.

### Batch 5 — Fase 4: Flatpak packaging + SPDX fix + metainfo

Merged 2026-06-04 — PR #48: Flatpak manifest rewrite (runtime 49, dir sources, meson module, scoped bus), SPDX fix (LNXDrive/GPL-3.0-or-later), metainfo completed. Bundle builds+installs clean via org.flatpak.Builder. Drift R8 (runtime 47→49 EOL, metainfo path). See AILOG-2026-06-04-001 + AIDEC-2026-06-04-001.

### Batch 6 — Fase 5: release infrastructure & public assets

(pending)

### Batch 7 — Fase 6: tag, release, announce + Charter close

(pending)

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
4. **GTK4 preferences panel** — the panel already exists under `lnxdrive-gnome/preferences/` (the root `src/main.rs` stub is just a placeholder). Fase 3 audits it (`.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`) and fixes the findings. It ships **three** settings groups wired to the daemon — Account, Folders (Sync), Network (Advanced) — plus Conflicts. The fourth group, **System** (auto-start, cache, dehydration), is **deferred to a v0.2 Charter** because it needs new daemon D-Bus API and is post-alpha (see AIDEC-2026-05-31-001). Key fix: realign the panel with the Fase-1 RISK-002 daemon API (`CompleteAuthViaGOA`). **Verified end-to-end** in the Nivel-5 testing VM (real GNOME Wayland): all pages load, full D-Bus contract exercised with no failed calls, live `QuotaChanged`, and operator-confirmed visual render of every page (incl. nested selective-sync selection). External pre-merge audit consolidated in `review-fase-3.md` (1 Medium, fixed). See AILOG-2026-05-31-002.
5. **Flatpak packaging** — complete `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` with install stages (icons, `*.desktop`, metainfo XML), correct permissions (`--filesystem=home:rw`, `--talk-name=org.freedesktop.secrets`), and target `org.gnome.Platform 47`. Fix `lnxdrive.spdx` (currently describes StrayMark by mistake). Complete the metainfo XML with description, releases section, and screenshot URLs. **Done** (Fase 4): manifest rewritten — the inherited skeleton pointed at two non-existent git repos and a stub binary; now builds daemon + CLI (cargo) and the GTK4 panel (meson, host-side Nautilus/Shell/GOA modules excluded) from monorepo `dir` sources. Target moved to `org.gnome.Platform 49` (47 was EOL before the Charter was signed — drift R8, AIDEC-2026-06-04-001); `--socket=session-bus` replaced by scoped bus names per the RISK-002 posture. SPDX now describes LNXDrive under GPL-3.0-or-later (was: StrayMark/MIT). **Verified**: bundle builds and installs cleanly via `org.flatpak.Builder` (3 binaries + desktop/schema/icon/metainfo exported; CLI answers in-sandbox). FUSE-under-sandbox behaviour intentionally left to the R2 VM smoke-test (Fase 6). See AILOG-2026-06-04-001.
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
| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` | Complete install stages, permissions, target `org.gnome.Platform 49` — original entry said 47, EOL since 2025 (drift R8, AIDEC-2026-06-04-001); scoped bus names replace `--socket=session-bus`; gnome module builds via meson with host-side extensions (Nautilus/Shell/GOA) disabled (Fase 4) |
| `lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in` | Full description, releases section, screenshot URLs — original entry misplaced the file under `lnxdrive-packaging/flatpak/`; it lives in the preferences meson tree (drift R8, AILOG-2026-06-04-001) (Fase 4) |
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
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
new file mode 100644
index 0000000..1171395
--- /dev/null
+++ b/.github/workflows/release.yml
@@ -0,0 +1,75 @@
+# Release pipeline (Charter-01 Fase 5)
+#
+# Trigger: push of a `v*` tag (e.g. v0.1.0-alpha.1).
+# Builds the Flatpak bundle from lnxdrive-packaging/flatpak/ and publishes a
+# GitHub Release (pre-release when the tag carries a pre-release suffix) with
+# the bundle + SHA256SUMS.
+#
+# Production smoke (Charter-01 §Verification) installs from:
+#   https://github.com/StrangeDaysTech/lnxdrive/releases/download/<tag>/lnxdrive.flatpak
+
+name: Release
+
+on:
+  push:
+    tags:
+      - 'v*'
+
+permissions:
+  contents: write
+
+jobs:
+  flatpak-bundle:
+    name: Build Flatpak bundle & publish release
+    runs-on: ubuntu-latest
+    env:
+      APP_ID: com.strangedaystech.LNXDrive
+      MANIFEST: lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
+    steps:
+      - name: Checkout
+        uses: actions/checkout@v4
+
+      - name: Install flatpak-builder
+        run: |
+          sudo apt-get update
+          sudo apt-get install -y --no-install-recommends flatpak flatpak-builder
+
+      - name: Install GNOME runtime + Rust SDK extension
+        run: |
+          flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
+          flatpak install --user --noninteractive flathub \
+            org.gnome.Platform//49 \
+            org.gnome.Sdk//49 \
+            org.freedesktop.Sdk.Extension.rust-stable//25.08
+
+      - name: Verify tag matches the declared version
+        run: |
+          TAG="${GITHUB_REF_NAME#v}"
+          DECLARED=$(grep -m1 '^version = ' lnxdrive-engine/Cargo.toml | cut -d'"' -f2)
+          if [ "$TAG" != "$DECLARED" ]; then
+            echo "::error::Tag v$TAG does not match workspace version $DECLARED"
+            exit 1
+          fi
+
+      - name: Build bundle
+        run: |
+          flatpak-builder --user --force-clean --repo=repo build-dir "$MANIFEST"
+          flatpak build-bundle repo lnxdrive.flatpak "$APP_ID" \
+            --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo
+
+      - name: Generate SHA256SUMS
+        run: sha256sum lnxdrive.flatpak > SHA256SUMS
+
+      - name: Publish GitHub Release
+        env:
+          GH_TOKEN: ${{ github.token }}
+        run: |
+          PRERELEASE_FLAG=""
+          case "$GITHUB_REF_NAME" in
+            *-*) PRERELEASE_FLAG="--prerelease" ;;
+          esac
+          gh release create "$GITHUB_REF_NAME" \
+            --title "LNXDrive $GITHUB_REF_NAME" \
+            --generate-notes \
+            $PRERELEASE_FLAG \
+            lnxdrive.flatpak SHA256SUMS
diff --git a/.straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md b/.straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md
index f36f28b..09a2cc2 100644
--- a/.straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md
+++ b/.straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md
@@ -121,6 +121,42 @@ expected effort estimate is **L** (large, multi-week, multi-batch).
   SHA256SUMS, announcement on r/linux, r/gnome, r/onedrive and
   StrangeDaysTech Mastodon. Charter closes with telemetry.
 
+## Batch Ledger
+
+> Backfilled on 2026-06-04 (Fase 5 PR): the Charter §Tasks mandated
+> `straymark charter batch-complete` after each phase merge, but this section
+> was never scaffolded at declaration time, so Fases 0–4 are recorded
+> retroactively from the merge history (process drift, documented in
+> AILOG-2026-06-04-002). Batch N = Fase N−1.
+
+### Batch 1 — Fase 0: governance foundation + public backlog
+
+Merged 2026-05-28 — PR #4 (declare Charter-01, archive non-MVP UIs, governance foundation). Part 2 (GitHub-side): 24 backlog issues created from risk-analysis on 2026-05-28; GitHub Project later removed as redundant (see CLAUDE.md §3).
+
+### Batch 2 — Fase 1: P0 risk mitigation + CI hardening
+
+Merged 2026-05-28→30 — PRs #32 (RISK-002 OAuth off D-Bus), #33 (RISK-003 FUSE write-during-hydration), #35 (RISK-001 D-Bus health monitor), #36 (ISSUE-002 YAML billion-laughs), #34 (config path fix), #39 (CI hardening + external audit, lands #37/#38), #40 (audit drafts). External pre-merge audit consolidated.
+
+### Batch 3 — Fase 2: engine polish (T101 + FUSE listing repair)
+
+Merged 2026-05-31 — PR #41: T101 closed via real-mount integration test (getattr 43.7µs, readdir 1.40ms/1000, RSS 37.9MB/10k); 4 FUSE listing bugs + inode-persistence defect fixed. See AILOG-2026-05-31-001.
+
+### Batch 4 — Fase 3: GTK4 preferences panel audit + remediation
+
+Merged 2026-05-31 — PR #42: panel audit (6 findings) + remediation; H1 RISK-002 drift fixed (CompleteAuthViaGOA); G1 System group deferred (AIDEC-2026-05-31-001). E2E verified in Nivel-5 VM. See AILOG-2026-05-31-002.
+
+### Batch 5 — Fase 4: Flatpak packaging + SPDX fix + metainfo
+
+Merged 2026-06-04 — PR #48: Flatpak manifest rewrite (runtime 49, dir sources, meson module, scoped bus), SPDX fix (LNXDrive/GPL-3.0-or-later), metainfo completed. Bundle builds+installs clean via org.flatpak.Builder. Drift R8 (runtime 47→49 EOL, metainfo path). See AILOG-2026-06-04-001 + AIDEC-2026-06-04-001.
+
+### Batch 6 — Fase 5: release infrastructure & public assets
+
+(pending)
+
+### Batch 7 — Fase 6: tag, release, announce + Charter close
+
+(pending)
+
 ## Out of scope (recorded ex-ante so the drift gate ignores them)
 
 - GTK4 preferences panel beyond the four basic groups → v0.2.
diff --git a/.straymark/07-ai-audit/agent-logs/packaging/AILOG-2026-06-04-001-fase-4-flatpak-packaging-manifest-rewrite-spdx-fix.md b/.straymark/07-ai-audit/agent-logs/packaging/AILOG-2026-06-04-001-fase-4-flatpak-packaging-manifest-rewrite-spdx-fix.md
new file mode 100644
index 0000000..ee62b3b
--- /dev/null
+++ b/.straymark/07-ai-audit/agent-logs/packaging/AILOG-2026-06-04-001-fase-4-flatpak-packaging-manifest-rewrite-spdx-fix.md
@@ -0,0 +1,195 @@
+---
+id: AILOG-2026-06-04-001
+title: "Fase 4 — Flatpak packaging: manifest rewrite, SPDX fix, metainfo completion"
+status: draft
+created: 2026-06-04
+agent: claude-opus-4-8-v1.0
+confidence: high
+review_required: true
+risk_level: medium
+eu_ai_act_risk: not_applicable
+nist_genai_risks: [information_security, value_chain]
+iso_42001_clause: [8]
+lines_changed: 510              # +447/-63 (git diff --shortstat del PR)
+files_modified:
+  - lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
+  - lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in
+  - lnxdrive.spdx
+  - .straymark/charters/01-road-to-v0-1-0-alpha-1.md
+observability_scope: none
+tags: [packaging, flatpak, spdx, metainfo, appstream, charter-01, phase-4]
+related:
+  - CHARTER-01-road-to-v0-1-0-alpha-1
+  - AIDEC-2026-06-04-001
+  - AILOG-2026-05-31-002
+---
+
+# AILOG: Fase 4 — Flatpak packaging: manifest rewrite, SPDX fix, metainfo completion
+
+## Summary
+
+Fase 4 del Charter-01 (scope item 5): el manifiesto Flatpak pasa de un esqueleto
+roto (repos git inexistentes, runtime 45 EOL, `command` apuntando a un stub, sin
+install stages) a un manifiesto funcional que construye daemon + CLI + panel
+GTK4 desde el monorepo, con sandbox scoped según la postura RISK-002.
+`lnxdrive.spdx` deja de describir StrayMark (proyecto equivocado, licencia MIT
+equivocada) y describe LNXDrive bajo GPL-3.0-or-later. El metainfo de
+Preferences se completa con descripción extensa, screenshots y release
+`0.1.0-alpha.1` (type development). **Verificado**: el bundle construye e
+instala limpio con `org.flatpak.Builder` y los tres binarios + assets quedan
+correctamente exportados en el sandbox.
+
+## Context
+
+El Charter-01 declara la Fase 4 como "Flatpak packaging + `lnxdrive.spdx` fix +
+metainfo completion". La exploración previa (agente Explore, mapeo del estado
+real antes de planificar) reveló que el manifiesto heredado no era completable
+de forma incremental: sus sources apuntaban a dos repos git separados con tag
+`v0.1.0` que nunca existieron (el proyecto es un monorepo sin tags), y su
+segundo módulo construía el binario stub `lnxdrive-gnome` ("Not yet
+implemented") en lugar del panel real `lnxdrive-preferences`. Se reescribió
+completo — decisiones de arquitectura en [[AIDEC-2026-06-04-001]].
+
+## Actions Performed
+
+1. **Manifiesto** (`lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml`):
+   - Runtime `org.gnome.Platform 45 → 49` (el "47" del Charter estaba EOL desde
+     2025 — drift R8, ver AIDEC).
+   - Sources `type: dir` relativos al manifiesto (monorepo local) con
+     `skip: [target]`, en lugar de repos git inexistentes.
+   - Módulo engine: `cargo build --release --locked -p lnxdrive-daemon -p
+     lnxdrive-cli` → instala `lnxdrived` y `lnxdrive`.
+   - Módulo gnome: buildsystem **meson** con `-Denable_nautilus=false
+     -Denable_shell=false -Denable_preferences=true -Denable_goa=false` — el
+     meson existente ya instala panel, iconos, `.desktop`, metainfo y schema
+     GSettings (los "install stages" del Charter sin duplicación manual). Las
+     extensiones Nautilus/Shell y el provider GOA son host-side y no pueden
+     vivir en el sandbox.
+   - `command: lnxdrive-preferences` (la GUI real; el daemon se lanza con
+     `flatpak run --command=lnxdrived`).
+   - Sandbox: se elimina `--socket=session-bus` en favor de
+     `--own-name=com.strangedaystech.LNXDrive` +
+     `--talk-name=org.freedesktop.secrets` +
+     `--talk-name=org.gnome.OnlineAccounts` (superficie D-Bus mínima,
+     coherente con RISK-002). `--filesystem=home` literal del Charter.
+     `--device=all` para `/dev/fuse` (files-on-demand; sin clase más fina).
+   - `build-args: --share=network` para cargo (ver Follow-ups: vendoring para
+     Flathub).
+2. **SPDX** (`lnxdrive.spdx`): reemplazo completo — describe LNXDrive (daemon,
+   FUSE, CLI, integración GNOME), `PackageLicenseConcluded/Declared:
+   GPL-3.0-or-later` (antes MIT, contradiciendo `LICENSE` y todos los crates),
+   `PackageVersion: 0.1.0` (antes 1.0.0; la unificación a `0.1.0-alpha.1` es
+   Fase 5), copyright alineado con `LICENSE`.
+3. **Metainfo**
+   (`lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in`):
+   descripción de 3 párrafos + lista de features por página del panel, URLs
+   `homepage`/`bugtracker`/`vcs-browser` apuntando al monorepo (antes:
+   `lnxdrive-gnome` repo separado), 3 screenshots con nombres canónicos que la
+   Fase 5 debe producir (`preferences-window.png`, `onboarding-wizard.png`,
+   `conflict-dialog.png` bajo `docs/screenshots/`), release `0.1.0-alpha.1`
+   `type="development"` con changelog (sustituye la entrada `0.1.0 /
+   2026-02-05` que nunca correspondió a un release publicado).
+4. **Atomic update del Charter** (formato v4): filas Fase 4 de la tabla
+   `## Files to modify` — runtime 47→49 y ruta real del metainfo (la tabla lo
+   ubicaba bajo `lnxdrive-packaging/flatpak/`; vive en el árbol meson de
+   preferences). Scope item 5 anotado con el resultado de la fase.
+
+## Risk
+
+- **R8 (drift, documentado aquí y en el Charter)**: dos desviaciones de la
+  declaración ex-ante — (a) runtime objetivo `org.gnome.Platform 47 → 49`
+  porque 47 alcanzó EOL en 2025, antes incluso de la firma del Charter
+  (2026-05-29); (b) la ruta del metainfo en la tabla Files-to-modify era
+  incorrecta. Ambas corregidas atómicamente en este mismo PR
+  ([[AIDEC-2026-06-04-001]]).
+- El riesgo R2 del Charter (comportamiento del bundle bajo sandbox ≠ `cargo
+  run`, en particular el mount FUSE) **sigue abierto por diseño**: su
+  mitigación es el smoke-test en VM Fedora/Ubuntu previo al release (Fase 6),
+  no esta fase.
+- **Salida del drift check** (`check-charter-drift.sh`, rango
+  `origin/main..HEAD`): los 26 archivos "declared but NOT modified" pertenecen
+  a las demás fases del Charter (el script compara la tabla completa contra un
+  PR de fase — esperado, no accionable). Los 3 "modified but NOT declared" son
+  falsos positivos: el metainfo **sí** está declarado (fila corregida por el
+  propio drift R8), `lnxdrive.spdx` **sí** está declarado (fila intacta de la
+  tabla; límite del parser heurístico), y `.straymark/follow-ups-backlog.md`
+  es el registro de governanza que debe viajar en el mismo commit que el AILOG
+  (AGENT-RULES §13), no scope de producto.
+
+## Modified Files
+
+| File | Lines Changed (+/-) | Change Description |
+|------|--------------------|--------------------|
+| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` | reescritura completa | Runtime 49, sources dir monorepo, módulo meson, sandbox scoped, command real |
+| `lnxdrive.spdx` | reescritura completa | Describe LNXDrive (antes StrayMark), GPL-3.0-or-later (antes MIT), versión 0.1.0 |
+| `lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in` | +60/-6 aprox | Descripción completa, URLs monorepo, screenshots, release 0.1.0-alpha.1 |
+| `.straymark/charters/01-road-to-v0-1-0-alpha-1.md` | ~6 líneas | Atomic update: filas Fase 4 + anotación scope item 5 (drift R8) |
+
+## Decisions Made
+
+Ver [[AIDEC-2026-06-04-001]] (runtime 49, sources dir, command, módulos
+host-side excluidos, bus scoped, red en build). Decisión menor sin AIDEC: la
+entrada de release `0.1.0 / 2026-02-05` del metainfo se sustituyó en lugar de
+conservarse — nunca hubo release público con esa versión y AppStream la
+mostraría como historial falso.
+
+## Impact
+
+- **Functionality**: primer artefacto de distribución construible del proyecto;
+  base directa para `release.yml` (Fase 5).
+- **Performance**: N/A.
+- **Security**: superficie D-Bus del sandbox reducida (own-name/talk-name en
+  vez de session-bus sin restricción); SPDX ahora declara la licencia real
+  (GPL-3.0-or-later) — relevante para compliance de distribución; tokens
+  siguen en keyring vía `--talk-name=org.freedesktop.secrets` (RISK-002).
+- **Privacy**: N/A (sin cambios en manejo de datos).
+- **Environmental**: N/A.
+
+## Verification
+
+- [x] `desktop-file-validate` sobre el `.desktop`: limpio (exit 0).
+- [x] `appstreamcli validate --no-net` sobre el metainfo: **pass** (1 aviso
+  pedante por mayúsculas en el app-id heredado, no accionable). Con red:
+  3 warnings esperados `screenshot-image-not-found` — los PNG llegan en
+  Fase 5 (nombres canónicos ya fijados, ver Follow-ups).
+- [x] Build de verificación del Charter: `flatpak run org.flatpak.Builder
+  --user --install --force-clean build-dir
+  lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` — **construye
+  e instala limpio** (commit `31106ca4`, 33.9 MB instalado, runtime
+  org.gnome.Platform/x86_64/49).
+- [x] Post-install en sandbox: `/app/bin/{lnxdrived,lnxdrive,
+  lnxdrive-preferences}` presentes; `flatpak run --command=lnxdrive … --version`
+  → `lnxdrive 0.1.0`; `.desktop`, `gschemas.compiled`, icono SVG y metainfo
+  exportados en `/app/share/`.
+- [x] `straymark validate` + `.straymark/scripts/check-charter-drift.sh`
+  pre-commit (resultados en la descripción del PR).
+
+## Follow-ups
+
+- **Vendoring de crates para Flathub**: el manifiesto usa `build-args:
+  --share=network` para que cargo descargue crates.io; una submission a
+  Flathub exige sources vendorizados (`flatpak-cargo-generator`). Aplica
+  cuando se decida publicar en Flathub (post-alpha, candidato v0.2).
+- **`lnxdrive-packaging/README.md` desactualizado**: promete subdirectorios
+  `rpm/`, `debian/`, `aur/`, `appimage/` que no existen (diferidos a
+  v0.2.0-beta por el Charter). Alinear el README con la realidad del alpha
+  (Flatpak only) — candidato a resolverse de paso en Fase 5 junto con el
+  README raíz.
+- **Nombres canónicos de screenshots para Fase 5**: el metainfo referencia
+  `docs/screenshots/{preferences-window,onboarding-wizard,conflict-dialog}.png`;
+  la Fase 5 debe producir exactamente esos nombres (más los 3 restantes del
+  Charter para el README).
+
+## Additional Notes
+
+- El bundle del alpha **no incluye** la extensión de Nautilus, la extensión de
+  Shell ni el provider GOA (componentes host-side). Documentarlo como
+  limitación conocida en README/release notes es trabajo de Fase 5.
+- Verificación de build ejecutada con `org.flatpak.Builder` (Flathub, user
+  install) — el binario `flatpak-builder` nativo no está empaquetado en la
+  máquina de verificación; el comando del Charter §Verification sigue siendo
+  válido sustituyendo el prefijo por `flatpak run org.flatpak.Builder`.
+
+---
+
+<!-- Template: StrayMark | https://strangedays.tech -->
diff --git a/.straymark/07-ai-audit/agent-logs/packaging/AILOG-2026-06-04-002-fase-5-release-infrastructure-public-assets.md b/.straymark/07-ai-audit/agent-logs/packaging/AILOG-2026-06-04-002-fase-5-release-infrastructure-public-assets.md
new file mode 100644
index 0000000..2684a93
--- /dev/null
+++ b/.straymark/07-ai-audit/agent-logs/packaging/AILOG-2026-06-04-002-fase-5-release-infrastructure-public-assets.md
@@ -0,0 +1,210 @@
+---
+id: AILOG-2026-06-04-002
+title: "Fase 5 — Release infrastructure & public assets"
+status: draft
+created: 2026-06-04
+agent: claude-opus-4-8-v1.0
+confidence: high
+review_required: true
+risk_level: low
+eu_ai_act_risk: not_applicable
+nist_genai_risks: [information_integrity, value_chain]
+iso_42001_clause: [8]
+lines_changed: 656              # +550/-106 (git diff --shortstat del PR, sin contar el commit del Batch Ledger)
+files_modified:
+  - .github/workflows/release.yml
+  - SECURITY.md
+  - CHANGELOG.md
+  - README.md
+  - docs/screenshots/README.md
+  - lnxdrive-packaging/README.md
+  - lnxdrive-engine/Cargo.toml
+  - lnxdrive-engine/Cargo.lock
+  - lnxdrive-gnome/Cargo.toml
+  - lnxdrive-gnome/Cargo.lock
+  - lnxdrive-gnome/meson.build
+  - lnxdrive-gnome/preferences/Cargo.toml
+  - lnxdrive-gnome/preferences/Cargo.lock
+  - lnxdrive.spdx
+  - .straymark/07-ai-audit/agent-logs/guide/AILOG-2026-05-29-001-roadmap-v0-1-0-alpha-foundation.md
+  - .straymark/follow-ups-backlog.md
+observability_scope: none
+tags: [release, ci, packaging, security-policy, changelog, readme, versioning, charter-01, phase-5]
+related:
+  - CHARTER-01-road-to-v0-1-0-alpha-1
+  - AILOG-2026-06-04-001
+  - AIDEC-2026-06-04-001
+---
+
+# AILOG: Fase 5 — Release infrastructure & public assets
+
+## Summary
+
+Fase 5 del Charter-01 (scope item 6): infraestructura de release y assets
+públicos. Se crea el pipeline tag→bundle→Release (`release.yml`), `SECURITY.md`
+(reporte privado + disclosure coordinado), `CHANGELOG.md` (Keep a Changelog,
+entrada `0.1.0-alpha.1`), se unifica la versión a `0.1.0-alpha.1` en los 5
+puntos de declaración + 3 Cargo.lock, y se actualiza el README raíz con
+instalación real por Flatpak, galería de screenshots (6 nombres canónicos) y
+tabla comparativa verificada contra el estado 2026 de jstaf/onedriver y
+abraunegg/onedrive. Los 6 PNG los captura el operador en la VM Nivel-5
+(decisión de esta sesión); el PR no se mergea sin ellos.
+
+Adicionalmente se **backfillea el Batch Ledger** del Charter (process drift:
+el Charter §Tasks mandaba `batch-complete` tras cada merge de fase, pero la
+sección nunca se scaffoldeó en el AILOG de origen — Fases 0–4 registradas
+retroactivamente desde el historial de merges con `--note`).
+
+## Context
+
+Con Fases 0–4 mergeadas, el proyecto tiene bundle construible pero cero
+infraestructura de publicación: sin workflow de release, sin política de
+seguridad, sin changelog, versiones `0.1.0` planas inconsistentes con el tag
+objetivo `v0.1.0-alpha.1`, y un README raíz con secciones obsoletas
+(instalación "Planned", quick-start de CLI con comandos inexistentes, roadmap
+pre-Charter con todo "Planned").
+
+## Actions Performed
+
+1. **`.github/workflows/release.yml`** (nuevo): trigger en tags `v*`;
+   construye con `flatpak-builder` nativo (apt, ubuntu-latest) + runtime 49 +
+   rust-stable 25.08; verifica que el tag coincide con la versión del
+   workspace (gate anti-desincronización); `flatpak build-bundle` →
+   `lnxdrive.flatpak` + `SHA256SUMS`; publica GitHub Release con `gh` CLI
+   (`--prerelease` automático si el tag lleva sufijo). **Sin actions de
+   terceros** (solo `actions/checkout`) — postura supply-chain.
+2. **`SECURITY.md`** (nuevo): reporte privado vía GitHub Security Advisories
+   (primario) o `contact@strangedays.tech`; ack ≤7 días; disclosure
+   coordinado ≤90 días; tabla de versiones soportadas (solo último alpha);
+   resumen de postura (keyring, bus scoped, YAML hardening, cargo
+   audit/deny).
+3. **`CHANGELOG.md`** (nuevo): Keep a Changelog 1.1.0; entrada
+   `0.1.0-alpha.1` con Added/Security/Known limitations; la fecha se fija al
+   taggear (Fase 6).
+4. **Unificación de versión** `0.1.0` → `0.1.0-alpha.1`: workspace del engine,
+   `lnxdrive-gnome/Cargo.toml`, `preferences/Cargo.toml`, `meson.build`,
+   `lnxdrive.spdx`; los tres `Cargo.lock` regenerados (`cargo update
+   --workspace --offline`) — crítico porque el build Flatpak usa `--locked`.
+   El lock del crate stub raíz (`lnxdrive-gnome`) se ajustó a mano: su
+   git-dependency apunta a un remoto que no resuelve y nada lo construye
+   (entrada idéntica a la que escribiría cargo). Verificado: `cargo check -p
+   lnxdrive-daemon -p lnxdrive-cli` limpio; sin asserts de versión en
+   tests/CI.
+5. **`README.md` raíz**: badges (alpha + release), instalación real
+   (`flatpak install --user <URL del bundle>` + verificación SHA256SUMS),
+   galería de 6 screenshots con nombres canónicos, first-steps reales
+   (wizard → GOA/browser → selective sync → daemon), quick-start del CLI con
+   **comandos reales** (status/auth/mount/pin/dehydrate/explain — los
+   anteriores `account add`/`ls`/`log` no existen; verificado contra el
+   binario del Flatpak instalado), tabla comparativa vs jstaf/onedriver
+   (v0.15.0, ene-2026, activo) y abraunegg/onedrive (v2.5.10, ene-2026,
+   activo) con maturity honesta (alpha vs stable), limitaciones del alpha
+   (componentes host-side), roadmap actualizado a milestones reales,
+   diferenciador multi-provider re-calificado como *(roadmap)*.
+6. **`docs/screenshots/README.md`** (nuevo): los 6 nombres canónicos +
+   instrucciones de captura para el operador (VM Nivel-5, cuenta de prueba).
+7. **`lnxdrive-packaging/README.md`**: realineado con la realidad del alpha
+   (Flatpak only; rpm/debian/aur/appimage diferidos a v0.2.0-beta; comandos
+   de build desde la raíz del monorepo; nota org.flatpak.Builder). Cierra
+   **FU-004** del registro (recount en el mismo commit).
+8. **Batch Ledger backfill** (commit previo de esta rama): sección añadida al
+   AILOG de origen del Charter con Batches 1–7 (= Fases 0–6); Batches 1–5
+   completados retroactivamente vía `straymark charter batch-complete
+   --note` con PRs y fechas del historial.
+
+## Risk
+
+- **R9 (process drift, new, not in Charter)**: el mecanismo de Batch Ledger
+  declarado en Charter §Tasks nunca se materializó en Fases 0–4 (la sección
+  no existía en el AILOG de origen). Corregido con backfill retroactivo desde
+  el historial de merges. Causa raíz: el scaffold del AILOG de origen
+  (formato pre-v4) no incluía la sección y ningún gate lo detectó hasta el
+  primer `batch-complete` real. El gate de cierre (`straymark charter drift`
+  rechaza `(pending)`) protegerá las Fases 5–6.
+- **R10 (scope note)**: el README raíz necesitó más que la "install section +
+  comparison" declarada — quick-start con comandos inexistentes y roadmap
+  pre-Charter eran contenido público falso a fecha de release; se corrigieron
+  en esta fase como parte del espíritu "public assets". El movimiento del
+  contenido previo no fue necesario (el README raíz ya era de producto; el
+  del engine ya existía por separado).
+- Los screenshots son **bloqueo de merge**: el PR queda en draft hasta que el
+  operador deposite los 6 PNG (FU-005 se cierra entonces).
+- **Salida del drift check** (rango `origin/main..HEAD`): los 21 "declared but
+  NOT modified" pertenecen a otras fases (esperado en PR de fase). Los 9
+  "modified but NOT declared" están todos cubiertos arriba: 3 `Cargo.lock` +
+  `meson.build` + `preferences/Cargo.toml` + `lnxdrive.spdx` = unificación de
+  versión (la fila del Charter "Every Cargo.toml with version=, all manifests,
+  metainfo XML" los declara en agregado); `docs/screenshots/README.md` =
+  soporte de la fila screenshots; `lnxdrive-packaging/README.md` = FU-004;
+  `follow-ups-backlog.md` = registro §13 en el mismo commit.
+
+## Modified Files
+
+| File | Lines Changed (+/-) | Change Description |
+|------|--------------------|--------------------|
+| `.github/workflows/release.yml` | +78 (nuevo) | Pipeline tag → bundle → Release + SHA256SUMS |
+| `SECURITY.md` | +55 (nuevo) | Política de seguridad y disclosure |
+| `CHANGELOG.md` | +60 (nuevo) | Entrada 0.1.0-alpha.1 |
+| `README.md` | ~150 modificadas | Install real, screenshots, comparativa, roadmap, CLI real |
+| `docs/screenshots/README.md` | +20 (nuevo) | Nombres canónicos para el operador |
+| `lnxdrive-packaging/README.md` | reescritura | Flatpak only, formatos diferidos (FU-004) |
+| 5 archivos de versión + 3 Cargo.lock | ~30 | Unificación 0.1.0-alpha.1 |
+| AILOG origen Charter | +40 | Batch Ledger backfill (R9) |
+| `.straymark/follow-ups-backlog.md` | ~10 | FU-004 closed + recount |
+
+## Decisions Made
+
+- **`gh` CLI + actions oficiales únicamente** en `release.yml` (sin
+  `softprops/action-gh-release` ni equivalentes): menor superficie
+  supply-chain; el runner ya trae `gh` autenticado con `github.token`.
+- **Gate tag↔versión** en el workflow: aborta si `v<tag>` ≠ versión del
+  workspace — previene releases con artefactos mal versionados.
+- **Comparativa honesta**: maturity "Alpha" vs "Stable" explícita y
+  recomendación directa de las alternativas para Business/SharePoint. Estado
+  de competidores verificado por web search (2026-06), no entrenamiento.
+- Screenshots: captura por el operador en VM Nivel-5 (opción elegida en
+  sesión frente a captura automatizada o PR aparte).
+
+## Impact
+
+- **Functionality**: con esto, `git tag v0.1.0-alpha.1 && git push --tags`
+  produce el release completo — Fase 6 queda reducida a tag + smoke + anuncio.
+- **Performance**: N/A.
+- **Security**: canal de reporte privado publicado; política de versiones
+  soportadas explícita; pipeline sin actions de terceros.
+- **Privacy**: screenshots con cuenta de prueba (instrucción explícita).
+- **Environmental**: N/A.
+
+## Verification
+
+- [x] `cargo check -p lnxdrive-daemon -p lnxdrive-cli --offline` limpio tras
+  el bump de versión + locks regenerados.
+- [x] Comandos del README quick-start contrastados contra `lnxdrive --help`
+  real (binario del Flatpak instalado en Fase 4).
+- [x] Estado de jstaf/onedriver y abraunegg/onedrive verificado por web
+  search (releases ene-2026, ambos activos).
+- [x] `straymark validate` 0 errores; `followups recount` tras cierre de
+  FU-004; drift check documentado en la descripción del PR.
+- [ ] `release.yml` se ejercita end-to-end recién en Fase 6 (primer tag) —
+  riesgo aceptado: el workflow replica los pasos ya verificados localmente
+  en Fase 4 (mismo manifiesto, mismo runtime).
+- [ ] 6 PNG del operador en `docs/screenshots/` (bloqueo de merge).
+
+## Follow-ups
+
+- **`lnxdrive-engine/config/lnxdrive-autostart.desktop` apunta a
+  `/usr/bin/lnxdrive-daemon`**: el binario real se llama `lnxdrived` y en
+  Flatpak vive en `/app/bin`. Sin efecto en el alpha (el autostart no se
+  instala desde el bundle), pero corregir antes de empaquetar formatos
+  nativos en v0.2.0-beta.
+
+## Additional Notes
+
+- FU-005 (nombres canónicos de screenshots) queda `open` hasta que los PNG
+  estén en el árbol; se cierra en el triage de esta fase.
+- La fecha del `CHANGELOG.md` y del `<release>` del metainfo se fijan al
+  taggear (Fase 6).
+
+---
+
+<!-- Template: StrayMark | https://strangedays.tech -->
diff --git a/.straymark/07-ai-audit/decisions/AIDEC-2026-06-04-001-flatpak-manifest-architecture.md b/.straymark/07-ai-audit/decisions/AIDEC-2026-06-04-001-flatpak-manifest-architecture.md
new file mode 100644
index 0000000..81e31e9
--- /dev/null
+++ b/.straymark/07-ai-audit/decisions/AIDEC-2026-06-04-001-flatpak-manifest-architecture.md
@@ -0,0 +1,109 @@
+---
+id: AIDEC-2026-06-04-001
+title: Arquitectura del manifiesto Flatpak para v0.1.0-alpha (runtime 49, sources dir, bus scoped)
+status: accepted
+created: 2026-06-04
+agent: claude-opus-4-8-v1.0
+confidence: high
+review_required: true
+risk_level: medium
+tags: [packaging, flatpak, runtime, sandbox, dbus, charter-01, phase-4]
+related:
+  - CHARTER-01-road-to-v0-1-0-alpha-1
+  - AILOG-2026-06-04-001
+---
+
+# AIDEC: Arquitectura del manifiesto Flatpak para v0.1.0-alpha
+
+## Context
+
+El Charter-01 (Fase 4, scope item 5) pide completar
+`lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` con install
+stages, permisos correctos y target `org.gnome.Platform 47`. El manifiesto
+heredado tenía cuatro defectos estructurales:
+
+1. Apuntaba a **dos repos git separados** (`lnxdrive.git`, `lnxdrive-gnome.git`
+   con tag `v0.1.0`) que no existen — el proyecto es un **monorepo** sin tags.
+2. `command: lnxdrive-gnome` ejecuta un **stub** (`src/main.rs` imprime
+   "Not yet implemented"); la GUI real es `lnxdrive-preferences`.
+3. Runtime `org.gnome.Platform 45` (EOL); el Charter declara 47, que también
+   alcanzó EOL en 2025 — antes incluso de la firma del Charter (2026-05-29).
+4. Sin install stages para iconos, `.desktop`, metainfo ni schema GSettings, y
+   con `--socket=session-bus` (bus de sesión sin restricción).
+
+## Problem
+
+Definir runtime objetivo, mecanismo de sources, comando principal, alcance de
+módulos y política de sandbox para el bundle del alpha, respetando el scope del
+Charter y la postura de seguridad de RISK-002 (superficie D-Bus mínima).
+
+## Alternatives Considered
+
+### Runtime objetivo
+- **A. `org.gnome.Platform 47`** (literal del Charter): EOL desde 2025 — sin
+  parches de seguridad; además libadwaita 1.6 justo en el límite del feature
+  gate `v1_6` del panel. Descartada: publicar un alpha sobre runtime EOL
+  contradice el espíritu de la Fase 1 (cierre de riesgos).
+- **B. `org.gnome.Platform 50`** (estable actual, mar 2026): válida, pero
+  reduce la ventana de compatibilidad para early adopters en distros LTS.
+- **C. `org.gnome.Platform 49`** ✅: el runtime soportado más antiguo en
+  jun 2026; satisface gtk4 `v4_14` + libadwaita `v1_6`; ya instalado en la
+  máquina de verificación Nivel-5.
+
+### Sources de los módulos
+- **A. Git remoto con tags** (heredado): los repos/tags no existen; rompería
+  el build local y el de release.
+- **B. `type: dir` relativo al manifiesto** ✅: construye siempre desde el
+  checkout local del monorepo — sirve igual para la verificación local del
+  operador y para `release.yml` (Fase 5). `skip: [target]` evita copiar
+  artefactos de cargo.
+- **C. `type: git` con `path:` local**: solo archivos commiteados — incómodo
+  para iterar (cada ajuste exige commit previo) sin aportar nada al alpha.
+
+### Comando principal y módulos
+- `command: lnxdrive-preferences` (la GUI real). El daemon se lanza con
+  `flatpak run --command=lnxdrived com.strangedaystech.LNXDrive`.
+- Módulo engine: `cargo build --release --locked -p lnxdrive-daemon -p
+  lnxdrive-cli` (los dos binarios reales: `lnxdrived`, `lnxdrive`).
+- Módulo gnome: **meson** con `-Denable_nautilus=false -Denable_shell=false`
+  — las extensiones de Nautilus/Shell y el provider GOA se cargan en procesos
+  del *host* y no pueden vivir dentro del sandbox; el meson existente ya
+  instala panel, iconos, `.desktop`, metainfo y schema (los "install stages"
+  que pedía el Charter, sin duplicarlos a mano).
+
+### Política de sandbox (finish-args)
+- Se elimina `--socket=session-bus` (acceso sin restricción) en favor de
+  **nombres scoped**: `--own-name=com.strangedaystech.LNXDrive` +
+  `--talk-name=org.freedesktop.secrets` + `--talk-name=org.gnome.OnlineAccounts`
+  — alineado con RISK-002 (superficie D-Bus mínima).
+- `--device=all` para `/dev/fuse` (no existe clase de device más fina en
+  Flatpak); el riesgo R2 del Charter ya prevé el smoke-test de FUSE bajo
+  sandbox en VM antes de publicar.
+- `--filesystem=home` literal del Charter (la raíz de sync vive en `$HOME`).
+
+### Red durante el build
+- cargo descarga crates.io vía `build-args: --share=network`. Una submission a
+  Flathub exigiría sources vendorizados (`flatpak-cargo-generator`); se difiere
+  — el alpha distribuye bundle por GitHub Releases (registrado como follow-up).
+
+## Decision
+
+Runtime **`org.gnome.Platform 49`** (drift documentado vs el "47" del Charter,
+con atomic update de la tabla Files-to-modify), sources **`type: dir`** por
+módulo con `skip`, `command: lnxdrive-preferences`, módulo gnome vía **meson**
+con extensiones host-side deshabilitadas, bus de sesión **scoped** (own-name +
+talk-names), `--device=all` para FUSE y red solo en build para cargo.
+
+## Consequences
+
+- El bundle construye reproduciblemente desde cualquier checkout del monorepo
+  sin depender de tags inexistentes; `release.yml` (Fase 5) lo reutiliza tal cual.
+- Early adopters necesitan `org.gnome.Platform//49` (descarga automática al
+  instalar el bundle).
+- La integración Nautilus/Shell/GOA queda fuera del Flatpak del alpha — se
+  documentará como limitación conocida en el README/release notes (Fase 5).
+- Flathub queda explícitamente fuera del alcance del alpha (follow-up FU en el
+  registro).
+- Si el smoke-test de R2 muestra que FUSE no monta bajo el sandbox ni con
+  `--device=all`, aplica la mitigación ya prevista: v0.1.0-alpha.2 con el
+  permiso/portal faltante.
diff --git a/.straymark/charters/01-road-to-v0-1-0-alpha-1.md b/.straymark/charters/01-road-to-v0-1-0-alpha-1.md
index 5d98456..91825e0 100644
--- a/.straymark/charters/01-road-to-v0-1-0-alpha-1.md
+++ b/.straymark/charters/01-road-to-v0-1-0-alpha-1.md
@@ -30,7 +30,7 @@ The lnxdrive monorepo finished its MVP implementation (SpecKit features `001-cor
    - `cargo audit` + `cargo deny` jobs in CI.
 3. **Engine polish** — close the one remaining task (T101 performance validation) in `lnxdrive-engine/specs/002-files-on-demand/tasks.md`. **Done** (Fase 2): T101 validated via a real-mount integration test — `getattr` 43.7µs, `readdir` 1.40ms/1000 entries, idle RSS 37.9MB/10k files (all under target). The test was the first real FUSE mount exercised in the codebase and surfaced four functional listing bugs (init runtime-context panic, root self-listing, unstable `readdir` order, `opendir` dir-cache) plus an inode-persistence defect, all fixed with regression tests — see AILOG-2026-05-31-001. The other three items this row originally listed (remove `todo!()/unimplemented!()`, remove debug `println!`, enable `cargo test --workspace` in CI) were **already completed during Fase 1** (verified against `main`: zero such sites in crates; `cargo test --workspace` live at `.github/workflows/engine-ci.yml:66`).
 4. **GTK4 preferences panel** — the panel already exists under `lnxdrive-gnome/preferences/` (the root `src/main.rs` stub is just a placeholder). Fase 3 audits it (`.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`) and fixes the findings. It ships **three** settings groups wired to the daemon — Account, Folders (Sync), Network (Advanced) — plus Conflicts. The fourth group, **System** (auto-start, cache, dehydration), is **deferred to a v0.2 Charter** because it needs new daemon D-Bus API and is post-alpha (see AIDEC-2026-05-31-001). Key fix: realign the panel with the Fase-1 RISK-002 daemon API (`CompleteAuthViaGOA`). **Verified end-to-end** in the Nivel-5 testing VM (real GNOME Wayland): all pages load, full D-Bus contract exercised with no failed calls, live `QuotaChanged`, and operator-confirmed visual render of every page (incl. nested selective-sync selection). External pre-merge audit consolidated in `review.md` (1 Medium, fixed). See AILOG-2026-05-31-002.
-5. **Flatpak packaging** — complete `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` with install stages (icons, `*.desktop`, metainfo XML), correct permissions (`--filesystem=home:rw`, `--talk-name=org.freedesktop.secrets`), and target `org.gnome.Platform 47`. Fix `lnxdrive.spdx` (currently describes StrayMark by mistake). Complete the metainfo XML with description, releases section, and screenshot URLs.
+5. **Flatpak packaging** — complete `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` with install stages (icons, `*.desktop`, metainfo XML), correct permissions (`--filesystem=home:rw`, `--talk-name=org.freedesktop.secrets`), and target `org.gnome.Platform 47`. Fix `lnxdrive.spdx` (currently describes StrayMark by mistake). Complete the metainfo XML with description, releases section, and screenshot URLs. **Done** (Fase 4): manifest rewritten — the inherited skeleton pointed at two non-existent git repos and a stub binary; now builds daemon + CLI (cargo) and the GTK4 panel (meson, host-side Nautilus/Shell/GOA modules excluded) from monorepo `dir` sources. Target moved to `org.gnome.Platform 49` (47 was EOL before the Charter was signed — drift R8, AIDEC-2026-06-04-001); `--socket=session-bus` replaced by scoped bus names per the RISK-002 posture. SPDX now describes LNXDrive under GPL-3.0-or-later (was: StrayMark/MIT). **Verified**: bundle builds and installs cleanly via `org.flatpak.Builder` (3 binaries + desktop/schema/icon/metainfo exported; CLI answers in-sandbox). FUSE-under-sandbox behaviour intentionally left to the R2 VM smoke-test (Fase 6). See AILOG-2026-06-04-001.
 6. **Release infrastructure & public assets** — `.github/workflows/release.yml` (tag → bundle → GitHub Release with SHA256SUMS); `SECURITY.md`; `CHANGELOG.md`; 6 UI screenshots in `docs/screenshots/`; version `0.1.0-alpha.1` consistent across every `Cargo.toml`, Flatpak manifest, and metainfo XML; README install section + competitive comparison vs `jstaf/onedriver` and `abraunegg/onedrive`.
 7. **Tag, release, announce** — signed tag `v0.1.0-alpha.1`, GitHub Pre-release with Flatpak bundle, posts on r/linux, r/gnome, r/onedrive, and StrangeDaysTech Mastodon.
 
@@ -65,8 +65,8 @@ This Charter spans many files across 7 phases. The table below names the load-be
 | `lnxdrive-engine/specs/002-files-on-demand/tasks.md` | Close the one remaining `[ ]` task (Fase 2) |
 | The ~4 engine files containing `todo!()/unimplemented!()` (incl. `audit.rs`, `filesystem.rs`) | Implement, remove, or feature-gate; replace ~10 debug `println!` with `tracing::debug!` (Fase 2) |
 | `lnxdrive-gnome/src/main.rs`, `lnxdrive-gnome/data/ui/preferences.ui` (new), `lnxdrive-gnome/Cargo.toml` | GTK4 prefs panel with 4 settings groups (Fase 3) |
-| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` | Complete install stages, permissions, target `org.gnome.Platform 47` (Fase 4) |
-| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in` | Full description, releases section, screenshot URLs (Fase 4) |
+| `lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml` | Complete install stages, permissions, target `org.gnome.Platform 49` — original entry said 47, EOL since 2025 (drift R8, AIDEC-2026-06-04-001); scoped bus names replace `--socket=session-bus`; gnome module builds via meson with host-side extensions (Nautilus/Shell/GOA) disabled (Fase 4) |
+| `lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in` | Full description, releases section, screenshot URLs — original entry misplaced the file under `lnxdrive-packaging/flatpak/`; it lives in the preferences meson tree (drift R8, AILOG-2026-06-04-001) (Fase 4) |
 | `lnxdrive.spdx` | Replace contents — currently describes StrayMark; should describe LNXDrive (Fase 4) |
 | `.github/workflows/release.yml` (new) | Tag → flatpak-builder bundle → GitHub Release + SHA256SUMS (Fase 5) |
 | `SECURITY.md` (new) | Disclosure policy, SLA, known limitations referencing risk-analysis docs (Fase 5) |
diff --git a/.straymark/follow-ups-backlog.md b/.straymark/follow-ups-backlog.md
index 5a57b96..f4c0049 100644
--- a/.straymark/follow-ups-backlog.md
+++ b/.straymark/follow-ups-backlog.md
@@ -1,9 +1,9 @@
 ---
 last_scan: 2026-06-04
 schema_version: v1
-total_open: 0
+total_open: 2
 total_promoted: 0
-total_closed_in_session: 2
+total_closed_in_session: 4
 total_phase_blocked: 0
 total_suspected_closed: 0
 buckets:
@@ -15,6 +15,8 @@ buckets:
 fully_extracted_ailogs:
   - AILOG-2026-05-28-001
   - AILOG-2026-05-28-002
+  - AILOG-2026-06-04-001
+  - AILOG-2026-06-04-002
 ---
 
 # Follow-ups Backlog
@@ -58,6 +60,38 @@ Entry shape (v1 — optional fields marked):
 - **Destination**: CHARTER-01
 - **Cost**: 0 (resolved at source)
 - **Notes**: Charter drift already remediated atomically in the source PR — the `## Files to modify` row for RISK-001 (only `health.rs`) was extended to `lnxdrive-daemon/src/main.rs` and cross-crate `lnxdrive-ipc/src/service.rs` (`dbus_health` state field + property). Closed at registry adoption (2026-06-04); no pending work.
+
+### FU-003 — **Vendoring de crates para Flathub**: el manifiesto usa `build-args:
+- **Origin**: AILOG-2026-06-04-001 §Follow-ups
+- **Status**: open
+- **Trigger**: TBD
+- **Destination**: TBD
+- **Cost**: TBD
+- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04.
+
+### FU-004 — **`lnxdrive-packaging/README.md` desactualizado**: promete subdirectorios
+- **Origin**: AILOG-2026-06-04-001 §Follow-ups
+- **Status**: closed
+- **Trigger**: resolved
+- **Destination**: CHARTER-01
+- **Cost**: S
+- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04. Resuelto en Charter-01 Fase 5 (AILOG-2026-06-04-002): README de packaging realineado con la realidad del alpha (Flatpak only, formatos diferidos a v0.2.0-beta).
+
+### FU-005 — **Nombres canónicos de screenshots para Fase 5**: el metainfo referencia
+- **Origin**: AILOG-2026-06-04-001 §Follow-ups
+- **Status**: closed
+- **Trigger**: Fase 5 release assets
+- **Destination**: docs/screenshots/ (PR #49)
+- **Cost**: S
+- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04. Closed 2026-06-17: los 6 PNG con nombres canónicos (preferences-window, onboarding-wizard, conflict-dialog, shell-indicator, status-menu, nautilus-overlays) capturados en VM Nivel-5 (GNOME Wayland, mock daemon) y añadidos a `docs/screenshots/`; coinciden con README raíz y metainfo AppStream.
+
+### FU-006 — **`lnxdrive-engine/config/lnxdrive-autostart.desktop` apunta a
+- **Origin**: AILOG-2026-06-04-002 §Follow-ups
+- **Status**: open
+- **Trigger**: TBD
+- **Destination**: TBD
+- **Cost**: TBD
+- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-06-04.
 ## Bucket: time-triggered
 
 ## Bucket: charter-triggered
diff --git a/CHANGELOG.md b/CHANGELOG.md
new file mode 100644
index 0000000..5811007
--- /dev/null
+++ b/CHANGELOG.md
@@ -0,0 +1,59 @@
+# Changelog
+
+All notable changes to LNXDrive are documented in this file.
+
+The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
+and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
+
+## [0.1.0-alpha.1] — date set at tag time (Charter-01 Fase 6)
+
+First public alpha, aimed at Linux power users and GNOME enthusiasts willing
+to report bugs. GNOME-only by design — KDE Plasma, GTK3 (XFCE/MATE) and COSMIC
+front-ends are archived under `experimental/` until after v1.0.0.
+
+### Added
+
+- **Sync engine** (`lnxdrived`, Rust, 12 crates): Microsoft OneDrive via the
+  Graph API with delta synchronization, conflict detection, local file
+  watching (inotify) and a D-Bus control interface
+  (`com.strangedaystech.LNXDrive`).
+- **Files-on-demand**: FUSE filesystem — cloud files are visible locally and
+  hydrate on first access; validated with a real-mount performance test
+  (`getattr` 43.7 µs, `readdir` 1.40 ms/1000 entries, 37.9 MB idle RSS with
+  10k files).
+- **CLI** (`lnxdrive`): auth, sync control, status and config commands.
+- **GNOME integration**: GTK4/libadwaita preferences panel (Account, Folders,
+  Network, Conflicts — wired live to the daemon over D-Bus), GNOME Shell
+  status indicator, Nautilus sync-state overlay icons, GNOME Online Accounts
+  single sign-on.
+- **Flatpak packaging** (`com.strangedaystech.LNXDrive`, GNOME 49 runtime):
+  ships daemon + CLI + preferences panel; published as a bundle on GitHub
+  Releases with SHA256SUMS via the tag-triggered release workflow.
+- `SECURITY.md` with private vulnerability reporting and coordinated
+  disclosure policy.
+
+### Security
+
+- OAuth tokens stored in the system keyring (Secret Service) and never sent
+  raw over D-Bus — the API exposes only opaque session handles (RISK-002,
+  CVSS 9.1, closed).
+- FUSE write-during-hydration race serialized with per-inode locking + `EBUSY`
+  (RISK-003, closed).
+- D-Bus session-bus health monitor with automatic reconnect and interface
+  re-registration (RISK-001, mitigated; full Unix-socket fallback deferred
+  to v0.2).
+- YAML config parser hardened against billion-laughs expansion (ISSUE-002,
+  closed).
+- `cargo audit` + `cargo deny` enforced in CI.
+
+### Known limitations
+
+- The Flatpak bundle does **not** include the Nautilus extension, the GNOME
+  Shell extension or the GOA provider — they load into host processes and
+  cannot live inside the sandbox.
+- FUSE under the Flatpak sandbox requires `--device=all`; behaviour is
+  smoke-tested in VMs before each release.
+- System settings group (auto-start, cache, dehydration policy) deferred to
+  v0.2 (needs new daemon D-Bus API).
+
+[0.1.0-alpha.1]: https://github.com/StrangeDaysTech/lnxdrive/releases/tag/v0.1.0-alpha.1
diff --git a/README.md b/README.md
index fbc5db7..7b7bf66 100644
--- a/README.md
+++ b/README.md
@@ -10,7 +10,8 @@
 
 <p align="center">
   <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg" alt="License"></a>
-  <a href="https://github.com/StrangeDaysTech/lnxdrive"><img src="https://img.shields.io/badge/status-early%20development-orange.svg" alt="Status"></a>
+  <a href="https://github.com/StrangeDaysTech/lnxdrive/releases"><img src="https://img.shields.io/badge/status-alpha-orange.svg" alt="Status"></a>
+  <a href="https://github.com/StrangeDaysTech/lnxdrive/releases"><img src="https://img.shields.io/badge/release-v0.1.0--alpha.1-blue.svg" alt="Release"></a>
   <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust"></a>
 </p>
 
@@ -25,7 +26,7 @@ LNXDrive is a cloud storage synchronization client for Linux designed from scrat
 - **Explainable sync** -- You always know *why* something failed or didn't sync. No more cryptic "sync error" messages.
 - **Files-on-Demand** -- A robust FUSE-based virtual filesystem with clear file states (online-only, locally available, always-keep). See your cloud files without downloading them.
 - **Native desktop integration** -- Purpose-built UI for GNOME, KDE Plasma, COSMIC, and GTK3-based desktops (XFCE, MATE). Not a generic tray icon -- real shell extensions, file manager overlays, and system settings panels.
-- **Multi-provider, multi-account** -- Connect OneDrive, Google Drive, Dropbox, and more. Unlimited account namespaces (`onedrive:work`, `gdrive:personal`).
+- **Multi-provider, multi-account** *(roadmap)* -- The provider port is designed for OneDrive, Google Drive, Dropbox, and more. The alpha ships **Microsoft OneDrive** support.
 - **Declarative configuration** -- Versionable YAML config, not scattered dotfiles and CLI flags.
 - **Full observability** -- Structured JSON logs, Prometheus metrics, and a complete audit trail.
 
@@ -46,56 +47,106 @@ Cloud Providers          LNXDrive Engine              Desktop
 
 ---
 
+## Screenshots
+
+| | | |
+|:---:|:---:|:---:|
+| ![Preferences window](docs/screenshots/preferences-window.png) | ![Onboarding wizard](docs/screenshots/onboarding-wizard.png) | ![Conflict dialog](docs/screenshots/conflict-dialog.png) |
+| *Preferences: account & quota* | *Onboarding wizard* | *Conflict resolution* |
+| ![Shell indicator](docs/screenshots/shell-indicator.png) | ![Status menu](docs/screenshots/status-menu.png) | ![Nautilus overlays](docs/screenshots/nautilus-overlays.png) |
+| *GNOME Shell indicator* | *Status menu* | *Nautilus sync overlays* |
+
+---
+
 ## Getting Started
 
-> LNXDrive is in **early development**. The instructions below describe the intended installation flow. Pre-built packages are not yet available.
+> LNXDrive is in **alpha** (`v0.1.0-alpha.1`), aimed at Linux power users and GNOME enthusiasts willing to [report bugs](https://github.com/StrangeDaysTech/lnxdrive/issues). The alpha is **GNOME-only** and supports **Microsoft OneDrive**.
 
 ### Requirements
 
-- Linux with kernel 5.15+ and FUSE 3 support
-- systemd (user session)
-- A supported desktop environment (GNOME 45+, KDE Plasma 6, COSMIC, XFCE 4.18+, or MATE 1.26+)
+- Linux with FUSE 3 support and Flatpak ≥ 1.12
+- GNOME (Wayland recommended) — the bundle pulls the `org.gnome.Platform//49` runtime automatically
 
-### Installation
+### Installation (Flatpak)
+
+```bash
+flatpak install --user \
+  https://github.com/StrangeDaysTech/lnxdrive/releases/download/v0.1.0-alpha.1/lnxdrive.flatpak
+```
 
-Pre-built packages will be available for:
+Verify the download against the `SHA256SUMS` file published with each
+[release](https://github.com/StrangeDaysTech/lnxdrive/releases).
 
-| Format | Desktop | Status |
-|--------|---------|--------|
-| Flatpak | All | Planned |
-| AppImage | All | Planned |
-| `.deb` | Debian/Ubuntu | Planned |
-| AUR | Arch Linux | Planned |
+Other formats (RPM, DEB, AUR, AppImage, Flathub) are planned for `v0.2.0-beta`.
 
 ### First steps
 
-1. **Install LNXDrive** using your preferred package format
-2. **Launch the setup wizard** from your application menu or run `lnxdrive setup`
-3. **Add a cloud account** -- the wizard will guide you through OAuth authentication
-4. **Choose your sync mode** -- select which folders to sync and whether to use files-on-demand
-5. **Start syncing** -- LNXDrive runs automatically as a systemd user service
+1. **Launch LNXDrive** from your application menu (or `flatpak run com.strangedaystech.LNXDrive`) — the onboarding wizard opens on first run
+2. **Sign in to OneDrive** — via GNOME Online Accounts or the browser OAuth flow; tokens are stored in the system keyring, never on disk
+3. **Choose what to sync** — pick OneDrive folders (nested selective sync supported)
+4. **Start the daemon**: `flatpak run --command=lnxdrived com.strangedaystech.LNXDrive`
+
+> **Alpha limitations**: the Flatpak bundle does not include the Nautilus
+> overlay extension, the GNOME Shell indicator, or the GOA provider — these
+> load into host processes and can't ship inside the sandbox. They are
+> available when [building from source](#building-from-source). See
+> [CHANGELOG.md](CHANGELOG.md) for the full list.
 
 ### CLI quick start
 
 ```bash
-# Check daemon status
+alias lnxdrive='flatpak run --command=lnxdrive com.strangedaystech.LNXDrive'
+
+# Check daemon and sync status
 lnxdrive status
 
-# Add a OneDrive account
-lnxdrive account add onedrive:work
+# Authenticate with OneDrive
+lnxdrive auth login
 
-# List synced files
-lnxdrive ls /
+# Mount the files-on-demand filesystem
+lnxdrive mount
 
-# Force sync a specific path
-lnxdrive sync ~/OneDrive/Documents
+# Keep a folder always available offline
+lnxdrive pin ~/OneDrive/Documents
 
-# View sync activity
-lnxdrive log --follow
+# Free local space (file stays visible, downloads on next open)
+lnxdrive dehydrate ~/OneDrive/Videos
+
+# Why is this file in its current state?
+lnxdrive explain ~/OneDrive/report.xlsx
 ```
 
 ---
 
+## How does it compare?
+
+Both alternatives are mature, actively maintained projects — credit where due.
+LNXDrive is the **alpha** newcomer betting on deep desktop integration and
+explainability:
+
+| | **LNXDrive** (alpha) | [jstaf/onedriver](https://github.com/jstaf/onedriver) | [abraunegg/onedrive](https://github.com/abraunegg/onedrive) |
+|---|---|---|---|
+| Approach | Background sync **+** FUSE files-on-demand | Pure on-demand FUSE filesystem (no offline sync) | Full bidirectional sync client |
+| Maturity | **Alpha** | Stable | Stable, very featureful |
+| Language | Rust | Go | D |
+| Files-on-demand | ✅ FUSE, with pin/unpin/hydrate states | ✅ FUSE (cache-based) | ❌ (downloads everything selected) |
+| Selective sync | ✅ nested folder picker (GUI) | — (on-demand by nature) | ✅ (config file: `sync_list`) |
+| Native settings GUI | ✅ GTK4/libadwaita panel | Minimal launcher GUI | ❌ CLI (third-party GUIs exist) |
+| File manager integration | ✅ Nautilus sync-state overlays* | ❌ | ❌ |
+| Desktop SSO | ✅ GNOME Online Accounts | ❌ (own OAuth flow) | ❌ (own OAuth flow) |
+| Explainability | ✅ `lnxdrive explain <file>`, audit log | ❌ | Verbose logs |
+| Token storage | System keyring (Secret Service) | System keyring | Config dir file |
+| OneDrive Business / SharePoint | Not yet (alpha) | ✅ | ✅ |
+| Packaging | Flatpak bundle | distro packages (COPR, etc.) | distro packages, Docker |
+
+<sub>*From source on the host; not included in the Flatpak sandbox (alpha).</sub>
+
+If you need OneDrive **Business/SharePoint today**, use one of the
+alternatives. If you want native GNOME integration with files-on-demand and
+explainable sync — that's the gap LNXDrive exists to fill.
+
+---
+
 ## For Developers
 
 ### Project Structure
@@ -157,23 +208,14 @@ This project uses [StrayMark](https://github.com/StrangeDaysTech/straymark) to m
 
 ## Roadmap
 
-LNXDrive development is organized in phases:
-
-| Phase | Milestone | Status |
-|-------|-----------|--------|
-| 0 | Testing infrastructure (containers, CI/CD, mock servers) | In progress |
-| 1 | Core engine + CLI (sync, delta, rate limiting, systemd service) | Planned |
-| 2 | Files-on-Demand (FUSE, placeholders, hydration) | Planned |
-| 3 | GNOME integration (Shell extension, Nautilus overlays, GOA) | Planned |
-| 4 | Conflict resolution UI and declarative policies | Planned |
-| 5 | KDE Plasma integration | Planned |
-| 6 | Multi-provider support (Google Drive, Dropbox) | Planned |
-| 7 | COSMIC desktop integration | Planned |
-| 8 | GTK3 integration (XFCE/MATE) | Planned |
-| 9 | Packaging and distribution | Planned |
-| 10 | Observability and advanced features | Planned |
-
-For the full roadmap with detailed deliverables, see [the roadmap document](lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md).
+| Milestone | Scope | Status |
+|-----------|-------|--------|
+| `v0.1.0-alpha.1` | OneDrive engine (sync, delta, FUSE files-on-demand), CLI, GNOME stack (GTK4 panel, Shell indicator, Nautilus overlays, GOA SSO), Flatpak bundle | **Current** |
+| `v0.2.0-beta` | System settings group (auto-start, cache, dehydration), RPM/DEB/AUR/AppImage, Flathub submission, D-Bus Unix-socket fallback, i18n structure, telemetry opt-in | Planned |
+| `v1.0.0` | KDE Plasma, COSMIC and GTK3 (XFCE/MATE) front-ends, multi-provider (Google Drive, Dropbox), 5+ languages | Planned |
+
+Work is tracked publicly in [GitHub Issues](https://github.com/StrangeDaysTech/lnxdrive/issues)
+and milestones; for detailed deliverables see [the roadmap document](lnxdrive-guide/09-Referencia/02-hoja-de-ruta.md).
 
 ---
 
diff --git a/SECURITY.md b/SECURITY.md
new file mode 100644
index 0000000..2f617e0
--- /dev/null
+++ b/SECURITY.md
@@ -0,0 +1,51 @@
+# Security Policy
+
+## Supported Versions
+
+LNXDrive is in **alpha**. Only the most recent alpha release receives security
+fixes; there are no backports.
+
+| Version | Supported |
+|---------|-----------|
+| 0.1.0-alpha.x (latest) | ✅ |
+| anything older | ❌ |
+
+## Reporting a Vulnerability
+
+**Please do not open a public issue for security vulnerabilities.**
+
+1. **Preferred**: use GitHub's private vulnerability reporting —
+   [Security → Report a vulnerability](https://github.com/StrangeDaysTech/lnxdrive/security/advisories/new).
+2. Alternatively, email **contact@strangedays.tech** with subject
+   `[SECURITY] lnxdrive: <short summary>`.
+
+Include: affected component (daemon / FUSE / CLI / GNOME panel / packaging),
+reproduction steps, impact assessment, and your environment (distro, GNOME
+version, install method).
+
+### What to expect
+
+- **Acknowledgement** within 7 days.
+- **Coordinated disclosure**: we ask for up to 90 days to ship a fix before
+  public disclosure. Single-maintainer project — complex fixes may need the
+  full window.
+- Credit in the release notes (opt-out if you prefer anonymity).
+
+## Security posture (alpha)
+
+Documented threat analysis lives in the repository's governance tree
+(`.straymark/02-design/risk-analysis/` and `.straymark/08-security/`).
+Highlights relevant to users:
+
+- **OAuth tokens never touch disk in cleartext and never travel raw over
+  D-Bus**: they live in the system keyring (Secret Service); the D-Bus API
+  exposes only opaque session handles.
+- The Flatpak sandbox uses **scoped D-Bus names** (no unrestricted
+  `--socket=session-bus`).
+- The config parser is hardened against YAML expansion attacks
+  (billion-laughs caps).
+- CI runs `cargo audit` and `cargo deny` on every change to the engine.
+
+Known limitations of the alpha (host-side components outside the Flatpak
+sandbox, FUSE device access) are documented in the release notes of each
+pre-release.
diff --git a/docs/screenshots/README.md b/docs/screenshots/README.md
new file mode 100644
index 0000000..ab1b1d8
--- /dev/null
+++ b/docs/screenshots/README.md
@@ -0,0 +1,16 @@
+# Screenshots (Charter-01 Fase 5)
+
+Capturas en la VM Nivel-5 (GNOME Wayland real). **Nombres canónicos** — ya
+referenciados por el README raíz y el metainfo de AppStream; no renombrar:
+
+| Archivo | Contenido | Referenciado por |
+|---------|-----------|------------------|
+| `preferences-window.png` | Panel de preferencias (página Account con quota) | README + metainfo (default) |
+| `onboarding-wizard.png` | Wizard de onboarding (conexión de cuenta) | README + metainfo |
+| `conflict-dialog.png` | Diálogo de resolución de conflictos lado a lado | README + metainfo |
+| `shell-indicator.png` | Indicador en GNOME Shell | README |
+| `status-menu.png` | Menú de estado desplegado | README |
+| `nautilus-overlays.png` | Nautilus con overlays de estado de sync | README |
+
+Formato: PNG, idealmente 16:9 o proporción de ventana natural, sin
+información personal (cuenta de prueba).
diff --git a/docs/screenshots/conflict-dialog.png b/docs/screenshots/conflict-dialog.png
new file mode 100644
index 0000000..ec93e8b
Binary files /dev/null and b/docs/screenshots/conflict-dialog.png differ
diff --git a/docs/screenshots/nautilus-overlays.png b/docs/screenshots/nautilus-overlays.png
new file mode 100644
index 0000000..773a6cf
Binary files /dev/null and b/docs/screenshots/nautilus-overlays.png differ
diff --git a/docs/screenshots/onboarding-wizard.png b/docs/screenshots/onboarding-wizard.png
new file mode 100644
index 0000000..815fa0e
Binary files /dev/null and b/docs/screenshots/onboarding-wizard.png differ
diff --git a/docs/screenshots/preferences-window.png b/docs/screenshots/preferences-window.png
new file mode 100644
index 0000000..1f959a8
Binary files /dev/null and b/docs/screenshots/preferences-window.png differ
diff --git a/docs/screenshots/shell-indicator.png b/docs/screenshots/shell-indicator.png
new file mode 100644
index 0000000..af2e26b
Binary files /dev/null and b/docs/screenshots/shell-indicator.png differ
diff --git a/docs/screenshots/status-menu.png b/docs/screenshots/status-menu.png
new file mode 100644
index 0000000..3d4c75d
Binary files /dev/null and b/docs/screenshots/status-menu.png differ
diff --git a/lnxdrive-engine/Cargo.lock b/lnxdrive-engine/Cargo.lock
index 37a3745..cefefce 100644
--- a/lnxdrive-engine/Cargo.lock
+++ b/lnxdrive-engine/Cargo.lock
@@ -1507,7 +1507,7 @@ checksum = "6373607a59f0be73a39b6fe456b8192fcc3585f602af20751600e974dd455e77"
 
 [[package]]
 name = "lnxdrive-audit"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "serde",
  "serde_json",
@@ -1517,7 +1517,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-cache"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1534,7 +1534,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-cli"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "chrono",
@@ -1557,7 +1557,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-conflict"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "lnxdrive-core",
  "serde",
@@ -1567,7 +1567,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-core"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1583,7 +1583,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-daemon"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1605,7 +1605,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-fuse"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "chrono",
@@ -1632,7 +1632,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-graph"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1659,7 +1659,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-ipc"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1674,7 +1674,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-sync"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1691,7 +1691,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-telemetry"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "prometheus",
  "reqwest",
diff --git a/lnxdrive-engine/Cargo.toml b/lnxdrive-engine/Cargo.toml
index 435c0d1..909fc23 100644
--- a/lnxdrive-engine/Cargo.toml
+++ b/lnxdrive-engine/Cargo.toml
@@ -15,7 +15,7 @@ members = [
 ]
 
 [workspace.package]
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 edition = "2021"
 authors = ["Strange Days Tech, S.A.S. <contact@strangedays.tech>"]
 license = "GPL-3.0-or-later"
diff --git a/lnxdrive-gnome/Cargo.lock b/lnxdrive-gnome/Cargo.lock
index 65fec11..80ca897 100644
--- a/lnxdrive-gnome/Cargo.lock
+++ b/lnxdrive-gnome/Cargo.lock
@@ -1005,7 +1005,7 @@ dependencies = [
 
 [[package]]
 name = "lnxdrive-gnome"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "anyhow",
  "gio",
diff --git a/lnxdrive-gnome/Cargo.toml b/lnxdrive-gnome/Cargo.toml
index 5fabf69..0ad7cff 100644
--- a/lnxdrive-gnome/Cargo.toml
+++ b/lnxdrive-gnome/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "lnxdrive-gnome"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 edition = "2021"
 authors = ["Strange Days Tech, S.A.S. <contact@strangedays.tech>"]
 license = "GPL-3.0-or-later"
diff --git a/lnxdrive-gnome/meson.build b/lnxdrive-gnome/meson.build
index 4ff27eb..c20a64c 100644
--- a/lnxdrive-gnome/meson.build
+++ b/lnxdrive-gnome/meson.build
@@ -1,7 +1,7 @@
 project(
   'lnxdrive-gnome',
   'c',
-  version: '0.1.0',
+  version: '0.1.0-alpha.1',
   license: 'GPL-3.0-or-later',
   meson_version: '>= 0.62.0',
   default_options: [
diff --git a/lnxdrive-gnome/preferences/Cargo.lock b/lnxdrive-gnome/preferences/Cargo.lock
index 2a8a07e..8d30643 100644
--- a/lnxdrive-gnome/preferences/Cargo.lock
+++ b/lnxdrive-gnome/preferences/Cargo.lock
@@ -1170,7 +1170,7 @@ checksum = "6373607a59f0be73a39b6fe456b8192fcc3585f602af20751600e974dd455e77"
 
 [[package]]
 name = "lnxdrive-preferences"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 dependencies = [
  "futures-util",
  "gettext-rs",
diff --git a/lnxdrive-gnome/preferences/Cargo.toml b/lnxdrive-gnome/preferences/Cargo.toml
index a8a4fd3..6bb5480 100644
--- a/lnxdrive-gnome/preferences/Cargo.toml
+++ b/lnxdrive-gnome/preferences/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "lnxdrive-preferences"
-version = "0.1.0"
+version = "0.1.0-alpha.1"
 edition = "2021"
 license = "GPL-3.0-or-later"
 description = "LNXDrive preferences and onboarding application for GNOME"
diff --git a/lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in b/lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in
index 83f541c..9a42e7f 100644
--- a/lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in
+++ b/lnxdrive-gnome/preferences/data/com.strangedaystech.LNXDrive.Preferences.metainfo.xml.in
@@ -9,9 +9,27 @@
     <p>
       LNXDrive Preferences lets you manage your cloud synchronization settings
       directly from GNOME. Configure your OneDrive accounts, choose which folders
-      to sync, set bandwidth limits, and control files-on-demand behavior -- all
+      to sync, set bandwidth limits, and control files-on-demand behavior — all
       from a single, native desktop application.
     </p>
+    <p>
+      LNXDrive is a OneDrive synchronization client for Linux built around a
+      background daemon with files-on-demand support: your cloud files appear in
+      the file manager and download only when you open them. The preferences
+      panel talks to the daemon over D-Bus and covers the full day-to-day
+      configuration surface:
+    </p>
+    <ul>
+      <li>Account: sign in with Microsoft via GNOME Online Accounts or the browser flow, see quota usage live, and sign out safely</li>
+      <li>Folders: pick which OneDrive folders sync to your computer, including nested selective sync</li>
+      <li>Network: cap upload and download bandwidth so syncing never gets in your way</li>
+      <li>Conflicts: review files changed both locally and in the cloud, and resolve them side by side</li>
+    </ul>
+    <p>
+      OAuth tokens are stored in the system keyring (Secret Service), never on
+      disk in cleartext. This alpha release targets GNOME on Wayland and ships
+      as a Flatpak.
+    </p>
   </description>
 
   <project_license>GPL-3.0-or-later</project_license>
@@ -21,12 +39,40 @@
     <name>Strange Days Tech</name>
   </developer>
 
-  <url type="homepage">https://github.com/strangedaystech/lnxdrive-gnome</url>
+  <url type="homepage">https://github.com/StrangeDaysTech/lnxdrive</url>
+  <url type="bugtracker">https://github.com/StrangeDaysTech/lnxdrive/issues</url>
+  <url type="vcs-browser">https://github.com/StrangeDaysTech/lnxdrive</url>
 
   <content_rating type="oars-1.1" />
 
+  <!-- Screenshot PNGs land in docs/screenshots/ in Charter-01 Fase 5; the
+       file names below are the canonical ones Fase 5 must produce. -->
+  <screenshots>
+    <screenshot type="default">
+      <caption>Preferences window with account and quota overview</caption>
+      <image>https://raw.githubusercontent.com/StrangeDaysTech/lnxdrive/main/docs/screenshots/preferences-window.png</image>
+    </screenshot>
+    <screenshot>
+      <caption>Onboarding wizard: connect your OneDrive account</caption>
+      <image>https://raw.githubusercontent.com/StrangeDaysTech/lnxdrive/main/docs/screenshots/onboarding-wizard.png</image>
+    </screenshot>
+    <screenshot>
+      <caption>Side-by-side conflict resolution dialog</caption>
+      <image>https://raw.githubusercontent.com/StrangeDaysTech/lnxdrive/main/docs/screenshots/conflict-dialog.png</image>
+    </screenshot>
+  </screenshots>
+
   <releases>
-    <release version="0.1.0" date="2026-02-05" />
+    <release version="0.1.0-alpha.1" date="2026-06-04" type="development">
+      <description>
+        <p>
+          First public alpha for GNOME early adopters: daemon with
+          files-on-demand (FUSE), CLI, GTK4 preferences panel, GNOME Shell
+          indicator, Nautilus sync-state overlays and GOA single sign-on.
+          Distributed as a Flatpak bundle on GitHub Releases.
+        </p>
+      </description>
+    </release>
   </releases>
 
   <launchable type="desktop-id">com.strangedaystech.LNXDrive.Preferences.desktop</launchable>
diff --git a/lnxdrive-packaging/README.md b/lnxdrive-packaging/README.md
index 672d772..ae7b1a3 100644
--- a/lnxdrive-packaging/README.md
+++ b/lnxdrive-packaging/README.md
@@ -2,68 +2,58 @@
 
 Scripts y configuraciones de empaquetamiento para LNXDrive.
 
-## Descripción
+## Estado actual (v0.1.0-alpha)
 
-Este repositorio centraliza todo el empaquetamiento de LNXDrive para diferentes formatos y distribuciones:
+El alpha distribuye **únicamente Flatpak** (decisión de alcance del
+Charter-01). Los demás formatos están diferidos al milestone `v0.2.0-beta`:
 
-- **Flatpak**: Distribución universal (recomendado)
-- **RPM**: Fedora, RHEL, openSUSE
-- **DEB**: Debian, Ubuntu, Linux Mint
-- **AUR**: Arch Linux
-- **AppImage**: Ejecutable portable
+| Formato | Estado |
+|---------|--------|
+| **Flatpak** (`flatpak/`) | ✅ Activo — bundle publicado en GitHub Releases |
+| RPM (Fedora, RHEL, openSUSE) | Diferido a `v0.2.0-beta` |
+| DEB (Debian, Ubuntu, Mint) | Diferido a `v0.2.0-beta` |
+| AUR (Arch Linux) | Diferido a `v0.2.0-beta` |
+| AppImage | Diferido a `v0.2.0-beta` |
+| Flathub | Diferido a `v0.2.0-beta` (requiere vendoring de crates) |
 
 ## Estructura
 
 ```
 lnxdrive-packaging/
-├── flatpak/          # Manifiestos Flatpak
-├── rpm/              # Especificaciones RPM
-├── debian/           # Empaquetamiento Debian
-├── aur/              # PKGBUILD para AUR
-├── appimage/         # Configuración AppImage
-└── scripts/          # Scripts de build y release
+├── flatpak/          # Manifiesto Flatpak (com.strangedaystech.LNXDrive.yaml)
+└── scripts/          # Scripts de build y release (placeholder)
 ```
 
-## Uso
+Los subdirectorios `rpm/`, `debian/`, `aur/` y `appimage/` se crearán cuando
+sus formatos se activen en `v0.2.0-beta`.
 
-### Flatpak
+## Build local (Flatpak)
 
-```bash
-cd flatpak
-flatpak-builder --user --install builddir com.strangedaystech.LNXDrive.yaml
-```
-
-### RPM (Fedora)
-
-```bash
-cd rpm
-rpmbuild -ba lnxdrive.spec
-```
-
-### DEB (Debian/Ubuntu)
+Desde la **raíz del monorepo** (las sources del manifiesto son rutas `dir`
+relativas):
 
 ```bash
-cd debian
-dpkg-buildpackage -us -uc
+flatpak-builder --user --install --force-clean build-dir \
+  lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
 ```
 
-### AUR
+O con el builder de Flathub si `flatpak-builder` no está empaquetado en tu
+distro:
 
 ```bash
-cd aur
-makepkg -si
+flatpak install --user flathub org.flatpak.Builder
+flatpak run org.flatpak.Builder --user --install --force-clean build-dir \
+  lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
 ```
 
-### AppImage
-
-```bash
-cd appimage
-./build-appimage.sh
-```
+El bundle incluye `lnxdrived` (daemon), `lnxdrive` (CLI) y
+`lnxdrive-preferences` (panel GTK4). Las extensiones de Nautilus/Shell y el
+provider GOA son componentes host-side y quedan fuera del sandbox.
 
 ## Release
 
-El script `scripts/release.sh` automatiza la creación de todos los paquetes para una nueva versión.
+El workflow `.github/workflows/release.yml` (raíz del monorepo) construye el
+bundle y publica el GitHub Release con `SHA256SUMS` al pushear un tag `v*`.
 
 ## Licencia
 
diff --git a/lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml b/lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
index 7a2e459..8380f82 100644
--- a/lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
+++ b/lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
@@ -1,46 +1,88 @@
-# Flatpak manifest for LNXDrive
-# Build with: flatpak-builder --user --install builddir com.strangedaystech.LNXDrive.yaml
+# Flatpak manifest for LNXDrive (v0.1.0-alpha — Charter-01 Fase 4)
+#
+# Build & install (from the monorepo root):
+#   flatpak run org.flatpak.Builder --user --install --force-clean build-dir \
+#     lnxdrive-packaging/flatpak/com.strangedaystech.LNXDrive.yaml
+#
+# Ships: lnxdrived (daemon), lnxdrive (CLI), lnxdrive-preferences (GTK4 panel),
+# app icons, desktop entry, AppStream metainfo and GSettings schema.
+# NOT shipped (host-side components, cannot live inside the sandbox):
+#   - Nautilus extension (loads into the host Nautilus process)
+#   - GNOME Shell extension (loads into the host gnome-shell process)
+#   - GOA provider (system-level GOA backend)
+#
+# Sources are `dir` type (relative to this manifest) so the bundle always
+# builds from the local monorepo checkout — both for the operator's local
+# verification and for the release workflow (Charter-01 Fase 5).
+#
+# NOTE (Flathub): cargo fetches crates.io during build via
+# `build-args: --share=network`. A Flathub submission would require vendored
+# sources (flatpak-cargo-generator); deferred — GitHub Releases bundle only
+# for the alpha.
 
 app-id: com.strangedaystech.LNXDrive
 runtime: org.gnome.Platform
-runtime-version: '45'
+# GNOME 47 (Charter target) reached EOL in 2025; 49 is the oldest runtime that
+# is both supported in mid-2026 and satisfies the panel's gtk4 v4_14 +
+# libadwaita v1_6 feature gates. See AIDEC-2026-06-04-001.
+runtime-version: '49'
 sdk: org.gnome.Sdk
 sdk-extensions:
   - org.freedesktop.Sdk.Extension.rust-stable
-command: lnxdrive-gnome
+command: lnxdrive-preferences
 
 finish-args:
+  # Wayland first, X11 fallback
+  - --socket=wayland
+  - --socket=fallback-x11
   - --share=ipc
+  # Sync engine talks to Microsoft Graph
   - --share=network
-  - --socket=fallback-x11
-  - --socket=wayland
-  - --socket=session-bus
+  # Sync root lives in the user's home (Charter-01 scope item 5)
   - --filesystem=home
+  # The daemon owns the well-known bus name; the panel and CLI talk to it.
+  # Scoped names instead of --socket=session-bus, in line with the RISK-002
+  # posture of keeping the D-Bus surface minimal.
+  - --own-name=com.strangedaystech.LNXDrive
+  # OAuth tokens at rest in the keyring (RISK-002 mitigation)
   - --talk-name=org.freedesktop.secrets
+  # GOA single sign-on (CompleteAuthViaGOA)
   - --talk-name=org.gnome.OnlineAccounts
+  # /dev/fuse for the files-on-demand mount (no finer-grained device class
+  # exists; Charter-01 R2 plans the VM smoke-test of FUSE under the sandbox)
+  - --device=all
+
+build-options:
+  append-path: /usr/lib/sdk/rust-stable/bin
+  build-args:
+    - --share=network
 
 modules:
-  - name: lnxdrive
+  # Core engine: daemon + CLI (Cargo workspace, 12 crates)
+  - name: lnxdrive-engine
     buildsystem: simple
-    build-options:
-      append-path: /usr/lib/sdk/rust-stable/bin
     build-commands:
-      - cargo build --release
+      - cargo build --release --locked -p lnxdrive-daemon -p lnxdrive-cli
       - install -Dm755 target/release/lnxdrived -t /app/bin
       - install -Dm755 target/release/lnxdrive -t /app/bin
     sources:
-      - type: git
-        url: https://github.com/strangedaystech/lnxdrive.git
-        tag: v0.1.0
+      - type: dir
+        path: ../../lnxdrive-engine
+        skip:
+          - target
 
+  # GNOME integration: GTK4/libadwaita preferences panel + icons + i18n.
+  # Nautilus/Shell/GOA modules are host-side and stay disabled (see header).
   - name: lnxdrive-gnome
-    buildsystem: simple
-    build-options:
-      append-path: /usr/lib/sdk/rust-stable/bin
-    build-commands:
-      - cargo build --release
-      - install -Dm755 target/release/lnxdrive-gnome -t /app/bin
+    buildsystem: meson
+    config-opts:
+      - -Denable_nautilus=false
+      - -Denable_shell=false
+      - -Denable_preferences=true
+      - -Denable_goa=false
     sources:
-      - type: git
-        url: https://github.com/strangedaystech/lnxdrive-gnome.git
-        tag: v0.1.0
+      - type: dir
+        path: ../../lnxdrive-gnome
+        skip:
+          - target
+          - preferences/target
diff --git a/lnxdrive.spdx b/lnxdrive.spdx
index 3a7c61c..9e15041 100644
--- a/lnxdrive.spdx
+++ b/lnxdrive.spdx
@@ -2,44 +2,20 @@ SPDXVersion: SPDX-2.3
 DataLicense: CC0-1.0
 SPDXID: SPDXRef-DOCUMENT
 DocumentName: lnxdrive
-DocumentNamespace: https://github.com/StrangeDaysTech/lnxdrive
+DocumentNamespace: https://github.com/StrangeDaysTech/lnxdrive/spdx/lnxdrive-0.1.0
 Creator: Organization: Strange Days Tech, S.A.S.
-Created: 2025-01-27T00:00:00Z
+Created: 2026-06-04T00:00:00Z
 
 PackageName: lnxdrive
 SPDXID: SPDXRef-Package
-PackageVersion: 1.0.0
+PackageVersion: 0.1.0-alpha.1
+PackageSupplier: Organization: Strange Days Tech, S.A.S.
 PackageDownloadLocation: https://github.com/StrangeDaysTech/lnxdrive
 PackageHomePage: https://github.com/StrangeDaysTech/lnxdrive
-PackageLicenseConcluded: MIT
-PackageLicenseDeclared: MIT
-PackageCopyrightText: Copyright (c) 2025 Strange Days Tech, S.A.S. and LNXDrive Contributors
-PackageSummary: Documentation governance framework for AI-assisted software development.
-PackageDescription: StrayMark provides a documentation governance system that ensures traceability for all significant changes in software projects, whether made by humans or AI agents. Includes templates, validation scripts, and multi-agent support.
-
-LicenseID: LicenseRef-MIT
-ExtractedText: <text>
-MIT License
-
-Permission is hereby granted, free of charge, to any person obtaining a copy
-of this software and associated documentation files (the "Software"), to deal
-in the Software without restriction, including without limitation the rights
-to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
-copies of the Software, and to permit persons to whom the Software is
-furnished to do so, subject to the following conditions:
-
-The above copyright notice and this permission notice shall be included in all
-copies or substantial portions of the Software.
-
-THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
-IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
-FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
-AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
-LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
-OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
-SOFTWARE.
-</text>
-LicenseName: MIT License
-LicenseCrossReference: https://opensource.org/licenses/MIT
+PackageLicenseConcluded: GPL-3.0-or-later
+PackageLicenseDeclared: GPL-3.0-or-later
+PackageCopyrightText: Copyright (C) 2025-2026 Strange Days Tech, S.A.S. and LNXDrive Contributors
+PackageSummary: OneDrive synchronization client for Linux with files-on-demand support.
+PackageDescription: LNXDrive is a cloud storage synchronization client for Linux, targeting Microsoft OneDrive via the Microsoft Graph API. It provides a background daemon (lnxdrived) with a D-Bus control interface, files-on-demand through a FUSE filesystem, a command-line client (lnxdrive), and native GNOME desktop integration: a GTK4/libadwaita preferences panel, a GNOME Shell status indicator, a Nautilus extension with sync-state overlay icons, and GNOME Online Accounts single sign-on. OAuth tokens are stored in the system keyring via the Secret Service API. Distributed as a Flatpak (com.strangedaystech.LNXDrive) targeting the org.gnome.Platform runtime.
 
 Relationship: SPDXRef-DOCUMENT DESCRIBES SPDXRef-Package

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
git_range: "31482c7..ae5a27d"
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
- **DO NOT audit, and DO NOT read for cross-reference, the audit folders** (`audit/` or `.straymark/audits/`). They hold other auditors' reports and prior analyses — neither project code for you to audit, nor input to your findings. In particular, do not open this cycle's sibling `report-*.md` files (see the ABSOLUTE RULE on independence): your audit must stand on the code you read yourself.
- **DO NOT run** destructive or generative commands. Only read/verify commands (`go vet`, `go build`, `go test`; `cargo check`, `cargo test --no-run`; `npm run lint`, `npm test`; or their equivalents).
- **DO NOT consult external sources** beyond what is provided in this prompt and the repository files you open via tool call. The audit must be reproducible from the prompt + the repo + the available read tools.

---

*StrayMark unified audit template v1. The seven universal sections (ABSOLUTE RULE, Your role, Scope rules, Step 2 mandatory verification, Step 5 severity calibration, What you must NOT do, Output format) come from the `audit/SKILL.md` skill mature pre-StrayMark in Sentinel, contributed via issue #102 by José Villaseñor Montfort (StrangeDaysTech). Sentinel-specific hardcodes (spec paths, Etapa headings, internal modules) were parameterized against the Charter doc, originating AILOGs, git range, and project context.*
