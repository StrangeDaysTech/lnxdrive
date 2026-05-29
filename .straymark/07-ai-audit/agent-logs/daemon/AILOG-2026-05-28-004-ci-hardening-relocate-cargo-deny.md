---
id: AILOG-2026-05-28-004
title: CI hardening — relocate dead engine workflow to repo root, add cargo-deny, remediate advisories
status: accepted
created: 2026-05-28
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [ci, supply-chain, cargo-deny, rustsec, clippy, charter-01, fase-1]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-05-28-002
  - AILOG-2026-05-28-003
  - TDE-2026-05-28-001
  - TDE-2026-05-28-002
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security]
iso_42001_clause: [8]
---

# AILOG: CI hardening — relocate workflow + cargo-deny + advisory remediation

## Summary

Closes the final Fase-1 item of Charter-01 ("cargo audit + cargo deny jobs in
CI"). An audit-before-acting pass found the premise was wrong in an important
way: **the engine CI workflow never ran.** It lived at
`lnxdrive-engine/.github/workflows/ci.yml`, but GitHub Actions only executes
workflows under the **repository-root** `.github/workflows/`; files in
subdirectories are ignored. `gh run list --workflow=ci.yml` returns 404 — there
is no run history. So the fmt / clippy / build / test / audit gates that
`ci.yml` defined had **never been enforced** on any PR.

This PR therefore does more than "add cargo deny":

1. **Relocate** the workflow to `.github/workflows/engine-ci.yml` (repo root)
   with `defaults.run.working-directory: lnxdrive-engine` and a path filter
   scoped to `lnxdrive-engine/**`, so it actually runs. Deletes the dead
   subdirectory `ci.yml`.
2. **Add a `cargo-deny` job** (EmbarkStudios/cargo-deny-action@v2) + a curated
   `lnxdrive-engine/deny.toml` (advisories / licenses / bans / sources). This
   **replaces** the old `rustsec/audit-check` job: cargo-deny's `advisories`
   check uses the same RustSec DB and subsumes cargo-audit, avoiding two
   overlapping ignore-lists.
3. **Make the now-live gates pass** — fix the pre-existing clippy debt and
   remediate the supply-chain advisories the gate surfaced (below).

## Context

Because the gates never ran, debt had accumulated undetected:

- **Clippy** (`--workspace --all-targets -D warnings`) failed on 5 pre-existing
  lints: 3 `assert_eq!(bool, literal)` in `config.rs` tests
  (`clippy::bool_assert_comparison`) and 2 `Iterator::last` on a
  `DoubleEndedIterator` in `lnxdrive-cache` tests.
- **`cargo deny`** surfaced several RUSTSEC advisories and a license-policy gap.

The operator chose (this session) "relocate + leave green" over a literal
minimal add. Per [[feedback-minimum-viable-plus-tde]], cheap fixes were applied
in-tree and genuinely out-of-scope fixes (breaking major bumps, a workspace-wide
reformat) were deferred to TDEs rather than bloating this PR or conflicting with
the open Fase-1 PRs (#35, #36).

## Change

### CI workflow

- **`.github/workflows/engine-ci.yml`** (new, root) — relocated `check` job
  (toolchains, system deps, fmt, clippy, build, test) with
  `working-directory: lnxdrive-engine`, `Swatinem/rust-cache` scoped to the
  workspace, and a `paths:` filter. New `deny` job runs cargo-deny against
  `lnxdrive-engine/Cargo.toml`.
- **`lnxdrive-engine/.github/workflows/ci.yml`** — deleted (dead path).
- The **fmt step is non-blocking** (`continue-on-error: true`) due to ~48 files
  of pre-existing rustfmt debt (TDE-2026-05-28-001); the bulk reformat lands in
  a dedicated chore PR after #35/#36 merge, then the step flips to blocking.

### Supply-chain (`lnxdrive-engine/deny.toml`, new)

- Advisories **resolved** via `cargo update`: `quinn-proto 0.11.13→0.11.14`,
  `rustls-webpki 0.103.9→0.103.13`, `rand 0.9.2→0.9.4` (cleared 5 advisories).
- `protobuf 2.28` (RUSTSEC-2024-0437 recursion vuln + unmaintained) **removed at
  the root** by setting `prometheus = { default-features = false }` in the
  workspace `Cargo.toml` — `lnxdrive-telemetry` only uses the text registry, not
  the protobuf push gateway.
- Advisories **deferred** (allow-listed with justification, tracked in
  TDE-2026-05-28-002): `RUSTSEC-2024-0363` (sqlx 0.7.4 — fix is the breaking
  sqlx 0.8 bump; SQLite not exploitable per the advisory) and
  `RUSTSEC-2024-0436` (paste — unmaintained, no known vuln, ubiquitous).
- Licenses: allow-list includes the project's own `GPL-3.0-or-later` plus the
  permissive licenses present (MIT, Apache-2.0, BSD-*, ISC, Unicode-*, MPL-2.0,
  CC0-1.0, CDLA-Permissive-2.0). Bans: `multiple-versions = warn`. Sources:
  crates.io only.

### Clippy debt fixed

- `crates/lnxdrive-core/src/config.rs` — 3× `assert_eq!(x, true/false)` →
  `assert!(x)` / `assert!(!x)`.
- `crates/lnxdrive-cache/tests/repository_tests.rs` — 2× `.split('/').last()` →
  `.next_back()`.
- `crates/lnxdrive-fuse/src/hydration.rs` — `manual_checked_ops`: a
  `total_size == 0` guard followed by `/ total_size` rewritten to
  `(range_end * 100).checked_div(total_size).map_or(100, …)`. Surfaced only by
  the first real CI run: GitHub's stable clippy was **1.96.0** while the local
  pinned-`stable` toolchain was **1.93.0**, and `manual_checked_ops` did not
  exist in 1.93. Fixing it keeps both versions green. The underlying fragility —
  a floating `stable` toolchain with `-D warnings` re-breaks on any new clippy
  release — is flagged to the operator; pinning `rust-toolchain.toml` to an
  exact version is a project-policy decision left to them.

## Verification

```bash
cd lnxdrive-engine
cargo clippy --workspace --all-targets -- -D warnings   # clean
cargo test --workspace                                  # all green
cargo deny check                                        # advisories ok, bans ok, licenses ok, sources ok
# fmt is intentionally non-blocking (TDE-2026-05-28-001):
cargo +nightly fmt --all -- --check                     # reports ~48 files (expected, deferred)
```

```bash
straymark validate
```

After merge, confirm the workflow is recognised (it wasn't before):
`gh run list --workflow=engine-ci.yml` should show runs.

## Drift

- **Premise correction (major)** — the Charter assumed jobs would be *added to*
  `lnxdrive-engine/.github/workflows/ci.yml`. That file never ran (wrong
  location). The real fix is **relocation** to the repo root; documented in the
  updated Charter row.
- **Consolidation** — replaced the planned separate `cargo audit` job with
  cargo-deny (which subsumes it), rather than maintaining two advisory configs.
- **Scope deferrals (TDEs)** — workspace rustfmt (TDE-2026-05-28-001, fmt step
  non-blocking) and breaking advisory fixes (TDE-2026-05-28-002, sqlx/paste
  allow-listed) deferred to keep this PR focused and conflict-free with #35/#36.
- **Cross-cutting files touched** beyond the Charter's `ci.yml` entry:
  `.github/workflows/engine-ci.yml` (new), workspace `Cargo.toml` + `Cargo.lock`
  (dep updates + prometheus features), `deny.toml` (new), and the two
  clippy-debt files. Charter row updated atomically.
- A dead duplicate `lnxdrive-engine/.github/workflows/docs-validation.yml`
  remains (a docs workflow, also ignored by GitHub); left untouched as
  out-of-scope for engine CI.

## Risk

- **R1 — Relocated workflow misconfigured (working-directory / path filter).**
  Medium-low. Verified locally that every command runs from `lnxdrive-engine`;
  the first PR run will confirm end-to-end. If the path filter is too narrow,
  the failure mode is "doesn't run", caught immediately on the PR.
- **R2 — Allow-listing real advisories (sqlx).** Accepted: SQLite backend is
  non-exploitable per the advisory; tracked in TDE-2026-05-28-002 with explicit
  justification, not silently ignored.
- **R3 — fmt left non-blocking.** Accepted: documented in TDE-2026-05-28-001
  with a concrete activation trigger (merge of #35/#36) and a flip-to-blocking
  step. Not silent — the step still annotates.
- **R4 — `prometheus default-features = false` drops a feature.** Low:
  `lnxdrive-telemetry` uses no protobuf/push API (grep-verified); it builds and
  its tests pass.

## Telemetry

| Metric | Estimated | Actual |
|---|---|---|
| Effort | 0.5 day (per Charter "add jobs") | ~0.7 day (scope grew: relocate + debt) |
| Lines added | ~60 | ~180 (workflow + deny.toml + 2 TDEs + AILOG) |
| Lines removed | ~5 | ~55 (old ci.yml + lint fixes) |
| New files | 1 (deny.toml) | 4 (workflow, deny.toml, 2 TDEs) |
| Advisories resolved | n/a | 6 (5 via update + protobuf removed) |
| Advisories deferred (justified) | n/a | 2 (sqlx, paste) |
| Existing tests broken | 0 | 0 |
| Pre-commit hook failures | n/a | none |
