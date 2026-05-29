---
id: TDE-2026-05-28-001
title: Apply workspace-wide rustfmt (pre-existing formatting debt, ~48 files)
status: identified
created: 2026-05-28
agent: claude-opus-4-8-v1.0
confidence: high
review_required: false
risk_level: low
type: code-quality
impact: low
effort: small
iso_42001_clause: [8]
tags: [rustfmt, formatting, ci, charter-01]
related:
  - AILOG-2026-05-28-004
  - CHARTER-01-road-to-v0-1-0-alpha-1
priority: low
assigned_to: null
promoted_from_followup: null
---

# TDE: Apply workspace-wide rustfmt

> **IDENTIFIED BY AGENT**: Prioritization and assignment require human decision.

## Summary

The engine workspace carries pre-existing rustfmt debt: `cargo fmt --all --
--check` reports **~48 files** under stable rustfmt (and ~57 under the
nightly options in `.rustfmt.toml`: `imports_granularity = "Crate"`,
`group_imports = "StdExternalCrate"`). The code was never consistently
formatted because the engine CI workflow — which ran `cargo +nightly fmt --all
-- --check` — was located at `lnxdrive-engine/.github/workflows/ci.yml`, a path
GitHub Actions ignores, so it never executed (see AILOG-2026-05-28-004).

## Why it is debt, not a bug

Formatting has no runtime effect. The fmt gate is now wired (relocated workflow
in `.github/workflows/engine-ci.yml`) but kept **non-blocking**
(`continue-on-error: true`) precisely because of this backlog.

## Why not now

Reformatting ~48 files in the CI-hardening PR would (a) dwarf the actual CI
changes and (b) conflict with the other open Fase-1 PRs (RISK-001 #35 touches
daemon files; ISSUE-002 #36 touches `config.rs` / CLI) — all among the files
needing reformat. A bulk reformat must land **after** those PRs merge.

## Proposed remediation

1. After PRs #35 and #36 merge, open a dedicated `chore: apply workspace
   rustfmt` PR running `cargo +nightly fmt --all` (single mechanical commit).
2. Flip the `Check formatting` step in `engine-ci.yml` from
   `continue-on-error: true` to blocking in the same PR.

## Activation trigger

The merge of Fase-1 PRs #35 and #36 (clears the conflict surface).

## Suggested milestone

`v0.1.0-alpha.1` (housekeeping, immediately after the Fase-1 security PRs).
