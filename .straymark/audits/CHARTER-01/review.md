---
audit_role: calibrator-reconciler
calibrator: claude-opus-4-8
charter_id: CHARTER-01-road-to-v0-1-0-alpha-1
phase: "Fase 3 — GTK4 preferences panel"
git_range: "origin/main..HEAD"
prompt_used: ../audit-prompt.md
calibrated_at: 2026-05-31
auditors_reconciled:
  - report-gemini-3-1-pro-high.md
  - report-gpt-5-2-codex.md
findings_consolidated: 1
findings_by_status:
  agreed: 0
  disputed: 0
  unique_gpt-5-2-codex: 1
  unique_gemini-3-1-pro-high: 0
  missed_by_auditors: 0
  rejected: 0
---

# Consolidated audit review — CHARTER-01 (Fase 3)

**Reviewer:** claude-opus-4-8
**Date:** 2026-05-31
**Confidence:** High

## 1. Executive summary

Two heterogeneous auditors reviewed the Fase-3 GTK4 preferences-panel work over
`origin/main..HEAD` (the six panel-fix commits resolving audit findings H1–H5 and
the G1 deferral). One substantive finding survives calibration: **M1
(gpt-5.2-codex, Medium)** — nested/selective-sync folder selections are not
persisted or restored. It is **VALID** and confirmed against the code; notably it
is the *recursive* half of the internal-audit finding H4 that the remediation
fixed only partially (the load race was fixed; the recursive child selection was
not). No Critical/High findings; no security regressions — both auditors
independently confirm the Fase-1 RISK-002 realignment (`CompleteAuthViaGOA`,
tokens off the bus) is correctly wired in the panel.

The two auditors diverged sharply in quality this round (inverting their Fase-1
ranking). **gpt-5.2-codex** focused on the actual diff, traced UI→D-Bus, and
found the real bug. **gemini-3-1-pro-high** returned a clean verdict but (a)
audited well outside the Fase-3 diff (FUSE/RISK code from already-merged phases)
and (b) marked Fases 4–7 — Flatpak, release, tag — as "Implemented" with
"Implementation read: No / Assumed", which is **false**: those phases do not
exist yet. It missed M1 (deflation) and asserted state it never verified.

**Overall verdict: PASS_WITH_ONE_MEDIUM.** Fase 3 is sound and security-clean;
M1 should be fixed to complete H4 before the phase PR merges.

## 2. Scope definition

Audited unit: **Fase 3 only** — the `lnxdrive-gnome/preferences/` panel changes
in `origin/main..HEAD`. The Charter's closure criterion is the full v0.1.0-alpha.1
(Fases 0–6); Fases 0–2 are merged (outside this diff), Fases 4–6 are not yet
started. Findings are evaluated against the **Fase-3 panel scope**, not the whole
Charter. gpt's task table correctly reports the later phases as absent from this
range (it labels them "Not implemented", which reads as range-relative, not a
claim they were skipped); gemini's "Implemented/Assumed" for those same phases is
the inaccurate reading.

## 3. Per-auditor evaluation

### 3.1 gpt-5.2-codex (model: gpt-5.2-codex)

| # | Finding | Reported sev | Verdict | Justification |
|---|---------|:---:|:---:|---|
| M1 | Nested folder selections not persisted/restored (`folder_tree.rs`) | Medium | **VALID** | Confirmed: `:235` children inherit `parent_selected` not the selected list; `collect_selected` (`:477`) iterates only the root store despite its "recursively" doc-comment. Selective-sync (FR-014) drops subfolder choices on save/reload. Medium is correct (functional UX defect, no security/crash). |

**Summary:** Accurate, well-cited (16 citations), correctly severized, zero false
positives. Found the one real defect in the diff — the recursive gap H4's fix
left open. Minor weakness: the task table labels merged Fases 0–2 "Not
implemented", which is range-relative but could read as a coverage claim.

### 3.2 gemini-3-1-pro-high (model: gemini-3-1-pro-high)

| # | Finding | Reported sev | Verdict | Justification |
|---|---------|:---:|:---:|---|
| — | (none) | — | — | Clean verdict. Correctly confirmed the RISK-002/GOA realignment and `serde_norway`, but those are mostly **out of the Fase-3 diff** (already-merged phases). Missed M1 (deflation). Asserted Fases 4–7 "Implemented (Assumed)" — factually wrong; those phases do not exist. |

**Summary:** Disciplined tone but poor scope control: it audited beyond the diff,
verified nothing for Fases 4–7 yet declared them implemented, and missed the one
real bug a focused read of `folder_tree.rs` would have surfaced.

## 4. Remediation plan — VALID and PARTIALLY VALID findings

### P3 — Robustness
- **M1 — nested selective-sync selections lost** (completes H4)
  - **Files:** `lnxdrive-gnome/preferences/src/preferences/folder_tree.rs:235` (child init), `:477-489` (`collect_selected`)
  - **Problem:** child `FolderNode`s inherit `parent_selected` instead of being
    matched against `selected_folders`; `collect_selected` walks only the root
    store, so expanded-subfolder selections are neither restored on load nor
    gathered on save. Selective sync (FR-014) silently drops nested choices.
  - **Remediation:** in the `create_model` closure, set each child's selected
    state from membership in `selected_folders` (not the parent's flag); make
    `collect_selected` actually recurse into materialised child stores (or
    collect from the `TreeListModel` rows), and persist the full set.
  - **Complexity:** Low–Medium.
  - **Detected by:** gpt-5.2-codex.

No P0/P1/P2/P4 findings.

## 5. Discarded findings — misattributions and false positives

None. gemini reported no findings (so none to discard); its out-of-scope
"Implemented (Assumed)" task rows are evaluation noise, not findings.

## 6. Auditor ratings

| Auditor | Scope precision (25%) | Technical depth (25%) | Bug detection (30%) | False-positive rate (20%) | **Overall** |
|---|:-:|:-:|:-:|:-:|:-:|
| gpt-5.2-codex | 7/10 | 8/10 | 9/10 | 10/10 | **8.4/10** |
| gemini-3-1-pro-high | 3/10 | 5/10 | 2/10 | 7/10 | **3.9/10** |

### Justifications

**gpt-5.2-codex — 8.4/10:** Found the only real defect in the diff, cited it
precisely, severized it correctly, and reported zero false positives. Lost a
little on scope wording (merged phases labelled "Not implemented").

**gemini-3-1-pro-high — 3.9/10:** A clean verdict that was wrong — it missed a
confirmable Medium and, more seriously, asserted that unimplemented Fases 4–7 were
"Implemented" without reading them. Confident-but-unverified state claims are
worse than silence in an auditor. (Inverts its strong Fase-1 showing — a reminder
that per-run heterogeneity, not a fixed "best model", is what pays off.)

## 7. Conclusion

Fase 3 is **clean on security and correct on the RISK-002 realignment**, with one
VALID Medium functional defect (M1) that completes the H4 fix. Recommended next
step: fix M1 (low–medium effort, isolated to `folder_tree.rs`), then open the
Fase-3 PR. The external-audit telemetry for this phase is emitted to
`external-audit-pending.yaml` for Charter close.
