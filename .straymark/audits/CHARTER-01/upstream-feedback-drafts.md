# Upstream feedback drafts — StrayMark (from Charter-01 Fase 1)

> Adopter feedback that LNXDrive owes upstream to `StrangeDaysTech/straymark`,
> referencing the adoption discussion
> [#205](https://github.com/StrangeDaysTech/straymark/discussions/205).
>
> **Status of the four findings from the Fase-1 work:**
>
> | # | Finding | Channel | State |
> |---|---|---|---|
> | 2a | `charter drift` rejects the range its Charter template ships | CLI/format friction (ad-hoc) | ✅ filed — [straymark#207](https://github.com/StrangeDaysTech/straymark/issues/207) |
> | 2b | `charter audit --prepare` default range under-covers phase audits | Documentation gap (ad-hoc) | ✅ filed — [straymark#208](https://github.com/StrangeDaysTech/straymark/issues/208) |
> | 1 | "declared but not wired" — now N=3 (cross-component regression of a shipped mitigation, found in Fase 3) | Pattern candidate | ✅ filed — [straymark#209](https://github.com/StrangeDaysTech/straymark/issues/209) (advanced from Charter-close cadence: the Fase-3 panel audit produced the N=3 data point) |
> | 4 | Charter scope declared against assumed (un-read) code → code-reconnaissance gate at creation | Process / methodology gap | ✅ filed — [straymark#210](https://github.com/StrangeDaysTech/straymark/issues/210) |
> | 3 | External-audit calibration results (dual-model + calibrator-hunts-missed) | External audit results / pattern | 🕓 draft below — file at Charter close |
>
> The cadence committed in #205 is **per Charter close** for telemetry + audit
> results, so drafts (1) and (3) wait until `straymark charter close
> CHARTER-01-road-to-v0-1-0-alpha-1`. The `external_audit:` array itself lives in
> `external-audit-pending.yaml` in this directory.

---

## Draft (1) — Pattern candidate: "declared but not wired" at N=2 across crate/D-Bus

Use the `Adopter feedback / upstream finding` issue template. Paste field values:

```markdown
### Adopter project
lnxdrive

### Adoption discussion link
https://github.com/StrangeDaysTech/straymark/discussions/205

### Finding type
Pattern candidate — recurring anti-pattern or discipline worth documenting

### Charter / telemetry reference
CHARTER-01-road-to-v0-1-0-alpha-1

### N-context
N=2 for the "Polish Charter as debt-detection / declared-but-not-wired"
pattern (Sentinel = N=1), now in a Rust FUSE/D-Bus/multi-crate domain.

### Description
The Fase-1 external audit reproduced the surface-declaration anti-pattern at
N=2, and — as predicted in the adoption discussion — with MORE surface than the
Go single-service case: the gap spanned a crate boundary AND a D-Bus method
signature, not one file.

RISK-002 was declared in the Charter as "opaque `SessionHandle` exposed by a new
`lnxdrive-daemon/src/dbus_iface.rs`". The shipped mitigation is
`Auth.CompleteAuthViaGOA(goa_account_path) -> bool` in `lnxdrive-ipc/src/service.rs`
+ `goa_auth_backend.rs` (security-equivalent, a deliberate minimum-viable
decision). No `SessionHandle` type exists; `dbus_iface.rs` was never created.
The decision was recorded in the AILOG's Context, but the Charter
`## Files to modify` table was never atomically backported — exactly the
"declared but not wired (in the doc)" gap.

Two heterogeneous auditors flagged it independently: gemini-3-1-pro-high via the
stale table row, gpt-5.2-codex via the scope-vs-implementation mismatch. A
post-hoc review would likely have missed it; the ex-ante Charter declaration is
what made the divergence legible.

### Proposed upstream change
Crystallize the pattern as validated at N=2. Concrete CLI idea surfaced by this
case: `charter drift` could flag `Files to modify` rows whose paths do not exist
in the tree (here `dbus_iface.rs` never existed) — a cheap mechanical signal for
"declared but not wired" that needs no semantic analysis.
```

---

## Draft (3) — External-audit results: heterogeneity + calibrator-hunts-missed

Use the `Adopter feedback / upstream finding` issue template. Paste field values:

```markdown
### Adopter project
lnxdrive

### Adoption discussion link
https://github.com/StrangeDaysTech/straymark/discussions/205

### Finding type
Pattern candidate — recurring anti-pattern or discipline worth documenting

### Charter / telemetry reference
CHARTER-01-road-to-v0-1-0-alpha-1

### N-context
N=2 for the external-audit calibration discipline (dual/multi-model) — first
real dual-model run outside Sentinel.

### Description
First end-to-end run of `straymark-audit-prompt -> -execute -> -review` on a real
phase, pre-merge, with two auditors of different families. Two observations the
telemetry supports:

1. Heterogeneity paid off — the families caught DIFFERENT real things:
   gemini-3-1-pro-high found the RISK-002 governance drift; gpt-5.2-codex found a
   dead `--capture-seconds` flag in a test script that gemini missed. Convergence
   AND divergence were both signal.
2. The calibrator must hunt, not just reconcile — the single most material
   finding (a daemon silently masking config-parse errors via `load_or_default`,
   undermining ISSUE-002 observability) was found by the review step's own code
   sweep, NOT by either auditor. This validates the skill's "find what the
   auditors missed" instruction as load-bearing, not optional.

### Telemetry excerpt (optional)
(YAML — see external-audit-pending.yaml in this directory)

external_audit:
  - auditor: "gemini-3-1-pro-high"
    findings_total: 1
    findings_by_category: {hallucination: 0, implementation_gap: 1, real_debt: 0, false_positive: 0}
    audit_quality: "high"
  - auditor: "gpt-5.2-codex"
    findings_total: 2
    findings_by_category: {hallucination: 0, implementation_gap: 1, real_debt: 1, false_positive: 0}
    audit_quality: "high"
# calibrator (claude-opus-4-8): 2 auditor findings consolidated (1 agreed, 1 unique),
# 0 false positives, +1 finding missed by all auditors. Ratings: gemini 7.0/10, gpt 8.0/10.

### Proposed upstream change
Document the calibrator's "missed-by-all" sweep as a required step (with this run
as evidence), and consider capturing per-auditor "unique vs agreed" counts in the
telemetry schema — the divergence count is what quantifies the value of adding a
second family.
```
