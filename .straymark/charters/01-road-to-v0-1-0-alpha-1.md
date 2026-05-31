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
4. **GTK4 preferences panel** — implement four basic settings groups (Account, Folders, Network, System) in `lnxdrive-gnome/src/main.rs` (currently a `println!("not yet implemented")` stub) wired to the existing D-Bus daemon API.
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
