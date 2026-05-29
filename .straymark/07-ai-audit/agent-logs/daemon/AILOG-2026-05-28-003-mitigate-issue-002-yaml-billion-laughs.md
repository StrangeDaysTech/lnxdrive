---
id: AILOG-2026-05-28-003
title: Mitigate ISSUE-002 — harden YAML config parser against billion-laughs
status: accepted
created: 2026-05-28
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [security, dos, yaml, config, billion-laughs, charter-01, issue-002, sim-l4-003]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AIDEC-2026-05-28-001
  - AILOG-2026-05-28-002
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security]
iso_42001_clause: [8]
---

# AILOG: Mitigate ISSUE-002 — YAML billion-laughs hardening

## Summary

Mitigates ISSUE-002 (alias D5 / SIM-L4-003, P0) — the configuration loader
`Config::load` called `serde_yaml::from_str` directly, with no size or alias
limits, on `serde_yaml 0.9.34+deprecated` (an archived crate with **no**
defense against the billion-laughs alias-expansion bomb). A crafted config could
exhaust memory/CPU at parse time.

Two enforcement layers:

1. **Dependency migration `serde_yaml` → `serde_norway`** (workspace-wide). The
   chosen replacement (decision recorded in [[AIDEC-2026-05-28-001]]) ships
   **built-in, on-by-default DoS limits** — recursion depth 128 and an
   alias-repetition cap (`events.len() * 100`) that reject billion-laughs
   bombs (`RecursionLimitExceeded` / `RepetitionLimitExceeded`). API-compatible
   with serde_yaml (`from_str` / `to_string`), so call sites are a 1:1 swap.

2. **Input size cap** — `Config::load` now delegates to a new
   `Config::from_yaml_str`, which rejects any config larger than
   `MAX_CONFIG_BYTES` (1 MiB) **before** parsing. Defense in depth, independent
   of the YAML library; satisfies the "size cap" arm of the Charter's ISSUE-002
   entry. The default config is ~1.4 KB, so 1 MiB is generous headroom.

## Context

`BACKLOG-simulation-issues.md` / `RISK-002-security-vulns.md` document D5: the
YAML parser expands aliases recursively without limit, so a small input
(`&a [..]`, `&b [*a,*a,..]`, …) explodes to ~10^9 nodes. The audit on 2026-05-28
(per [[feedback-validate-before-security-code]]) confirmed:

- `Config::load` (`lnxdrive-core/src/config.rs`) was `read_to_string` +
  `serde_yaml::from_str`, with **no** pre-parse validation.
- `serde_yaml 0.9` is deprecated/archived and offers no configurable
  alias/depth/size limits — migrating to it was a dead end, and migrating to the
  API-compatible `serde_yaml_ng` would **not** add protection (same
  `unsafe-libyaml` backend). See [[AIDEC-2026-05-28-001]] for the full
  six-crate comparison and why `serde_norway` was chosen over the
  hardened-but-supply-chain-risky `serde_yaml_bw`.

Operator decision (per [[feedback-minimum-viable-plus-tde]]): adopt a maintained
fork whose built-in limits mitigate the bomb by default, rather than write and
maintain a bespoke alias-counting pre-scanner.

## Change

### Code

- **`lnxdrive-engine/Cargo.toml`** — workspace dependency `serde_yaml = "0.9"`
  → `serde_norway = "0.9"` (with a comment explaining the security rationale).
- **`crates/lnxdrive-core/Cargo.toml`**, **`crates/lnxdrive-cli/Cargo.toml`** —
  `serde_yaml.workspace = true` → `serde_norway.workspace = true`.
- **`crates/lnxdrive-core/src/config.rs`**:
  - New `Config::MAX_CONFIG_BYTES = 1 << 20` (1 MiB).
  - New `pub fn from_yaml_str(&str) -> anyhow::Result<Self>` enforcing the size
    cap then deferring to `serde_norway::from_str`. `Config::load` delegates to
    it (so the hardening is exercisable without disk I/O).
  - Internal test call site migrated to `serde_norway::from_str`.
- **`crates/lnxdrive-cli/src/commands/config.rs`** — two `serde_yaml::to_string`
  call sites migrated to `serde_norway::to_string`.

### Tests

- **`lnxdrive-engine/tests/security/billion_laughs.yaml`** (new) — the canonical
  9-level alias bomb, at the Charter-specified workspace path.
- **`crates/lnxdrive-core/src/config.rs` (`#[cfg(test)]`)** — four tests:
  1. `test_billion_laughs_rejected` — the production path
     (`Config::from_yaml_str`) rejects the bomb and returns fast (a hang here
     would mean the cap regressed).
  2. `test_billion_laughs_trips_dos_limit` — parsing the same bomb to an untyped
     `serde_norway::Value` errors with a "limit"/"recursion"/"repetition"
     message, proving the rejection is the DoS guard and not typed-struct
     short-circuiting.
  3. `test_oversized_config_rejected` — a >1 MiB input fails the size cap before
     parsing.
  4. `test_default_config_still_parses` — the shipped `config/default-config.yaml`
     still parses, proving the hardening did not break valid input.

### Governance

- **[[AIDEC-2026-05-28-001]]** records the dependency decision with the full
  alternatives analysis (serde_yaml_ng / serde_yaml_bw / serde_norway + the
  low-level parsers), including the supply-chain reasoning that ruled out
  `serde_yaml_bw`'s three `0.0.x` author-maintained sub-crates.
- **Charter `## Files to modify`** — the ISSUE-002 row (which named the
  non-existent `lnxdrive-config/src/parser.rs`) is corrected atomically to the
  real path `lnxdrive-core/src/config.rs` and the actual mitigation shape
  (dependency swap + size cap, alias cap via the library).

## Verification

```bash
cd lnxdrive-engine

# The four ISSUE-002 tests
cargo test -p lnxdrive-core --lib config::tests::test_billion_laughs_rejected \
  config::tests::test_billion_laughs_trips_dos_limit \
  config::tests::test_oversized_config_rejected \
  config::tests::test_default_config_still_parses
# Expected: 4 passed.

# Full workspace — no regressions from the dependency swap
cargo test --workspace
# Expected: all green (lnxdrive-core: 223 passed, +4 vs. before).
```

Governance:

```bash
straymark validate
straymark charter status CHARTER-01-road-to-v0-1-0-alpha-1
```

## Drift

- **Charter path correction** — the Charter's ISSUE-002 entry named
  `lnxdrive-engine/crates/lnxdrive-config/src/parser.rs` "(or equivalent)"; no
  such crate exists. The real config parser is `lnxdrive-core/src/config.rs`.
  Row corrected atomically in this PR.
- **Mitigation shape vs. Charter wording** — the Charter said "size + alias
  caps". Final shape: **size cap implemented in-tree** (`MAX_CONFIG_BYTES`) +
  **alias cap delegated to `serde_norway`'s built-in limits** (rather than a
  hand-written alias pre-scanner). Recorded here and in [[AIDEC-2026-05-28-001]].
- **Cross-crate sweep** — the dependency rename also touched
  `lnxdrive-cli` (Cargo.toml + two `to_string` call sites), not enumerated in
  the Charter; listed in the updated row.

## Risk

Dependency migration of a parsing library plus an additive input cap.

- **R1 — Behavioural difference between serde_yaml and serde_norway.** Low.
  `serde_norway` is a direct serde-yaml fork with the same data model; the full
  workspace suite (223 core tests incl. the existing config round-trip tests)
  passes unchanged, and `test_default_config_still_parses` pins the real shipped
  config.
- **R2 — serde_norway maintenance (bus factor 1, ~17-month release gap).**
  Accepted in [[AIDEC-2026-05-28-001]]; mitigated by the permissive license
  (forkable) and the library-independent size cap.
- **R3 — Limits not configurable.** Accepted: the threat model is a local config
  file; the hardcoded recursion/alias caps are more than sufficient.

No emergent risks.

## Telemetry

| Metric | Estimated | Actual |
|---|---|---|
| Effort | 0.5 day | ~0.3 day |
| Lines added | ~80 | ~90 (incl. fixture + tests) |
| Lines removed | ~5 | ~5 |
| New files | 3 (fixture, AIDEC, AILOG) | 3 |
| Existing tests broken | 0 | 0 |
| Tests added | 1 (billion-laughs) | 4 |
| Pre-commit hook failures | n/a | none |
