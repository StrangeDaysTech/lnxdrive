---
id: AILOG-2026-05-31-002
title: Fase 3 — GTK4 preferences panel audit + findings remediation
status: draft
created: 2026-05-31
agent: claude-opus-4-8-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [gnome, preferences, gtk4, dbus, goa, risk-002, charter-01, phase-3, audit]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - phase-3-gtk4-panel-audit
  - AIDEC-2026-05-31-001
  - AILOG-2026-05-29-002
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security]
iso_42001_clause: [8]
---

# AILOG: Fase 3 — GTK4 panel audit + remediation

## Summary

Fase 3 was scoped as "implement the GTK4 preferences panel (currently a stub)".
The stub is only `lnxdrive-gnome/src/main.rs`; the real panel already exists under
`lnxdrive-gnome/preferences/` and compiles. Per the operator, the work became a
**deep audit** of that panel (3 parallel Explore agents, calibrated against
source — `.straymark/audits/CHARTER-01/phase-3-gtk4-panel-audit.md`) followed by
remediation of the findings.

Six findings (1 High, 3 Medium, 1 Low, 1 gap); four agent over-classifications
rejected. All resolved: **H1–H5 fixed, G1 deferred** ([[AIDEC-2026-05-31-001]]).
`cargo check` and `cargo clippy -- -D warnings` are clean for the panel (the
latter for the first time).

## The findings & fixes

- **H1 (High) — RISK-002 drift.** Fase 1 removed `Auth.CompleteAuthWithTokens`
  from the daemon and shipped `CompleteAuthViaGOA` (tokens off the bus), but the
  panel still called the removed method, and its GOA code sat behind a `goa`
  feature that `Cargo.toml` never defined → compiled out. This is the **third
  occurrence (N=3)** of the "declared but not wired" pattern reported upstream to
  StrayMark (#205), and the first that is a *regression* of a shipped Fase-1
  mitigation. Fix: define the `goa` feature (default on); add the
  `complete_auth_via_goa` proxy and drop `complete_auth_with_tokens`; hand the
  GOA account object-path to the daemon (tokens never client-side). This also
  surfaced and fixed a latent type error in `goa_sso` that had **never compiled**
  (the feature was always off) — concrete evidence the GOA path was dead code.
  The manual browser auth path (`start_auth` + `AuthStateChanged`) was unaffected.
- **H2 (Medium) — daemon state not consumed.** Added the missing Sync/Status
  properties+signals and `Settings.config_changed` to the proxies, and wired a
  real consumer (AccountPage refreshes quota on `QuotaChanged`).
- **H3 (Medium) — silent errors.** `folder_tree`, `sync_page`, and the onboarding
  pages now surface load/save failures in the UI (inline error, error group,
  toast/banner) instead of stderr; `folder_tree` distinguishes a parse error from
  an empty tree.
- **H4 (Medium) — folder_tree load race.** Merged the two independent load tasks
  into one ordered task (selections first, then populate) so selections can no
  longer apply to an empty tree.
- **H5 (Low) — lint debt.** `cargo check` warnings cleared (unused imports,
  deprecated `ActionRow::icon_name` → `add_prefix`); the audit also surfaced 145
  pre-existing `needless_borrow` clippy lints across the panel (the panel had
  never passed clippy `-D warnings`), auto-fixed in this pass.
- **G1 (gap) — "System" settings group.** Deferred to a v0.2 Charter
  ([[AIDEC-2026-05-31-001]]): cache/dehydration need new daemon D-Bus API and are
  post-alpha. Fase 3 ships three wired groups (Account, Folders, Network) +
  Conflicts.

## Rejected (calibration)

The `.expect()` cascades in GTK factories (idiomatic, type-guaranteed), the
missing `Files` interface (Nautilus' concern, not the prefs UI), the
`STRATEGY_VALUES[i]` index (equal-length consts), and a FUSE-style async deadlock
(zbus uses async-io + glib `spawn_local`, no `block_on`) were verified and
rejected as not-a-bug for this codebase.

## Verification

```bash
cd lnxdrive-gnome/preferences
cargo check                              # clean
cargo clippy --all-targets -- -D warnings  # clean (first time for the panel)
```

### Runtime verification (Nivel-5 VM, 2026-05-31)

Run end-to-end in the `lnxdrive-testing` QEMU/libvirt VM (Fedora + GNOME Wayland),
which compiled the daemon and panel from this branch over the 9p mount and ran
the mock daemon (`--authenticated`, updated this session with `CompleteAuthViaGOA`
— it carried the same RISK-002 drift). Captured over SSH:

- **Panel launches and stays alive** in real GNOME Wayland (no panic; survives a
  5s liveness probe). The `libEGL/MESA ZINK` warnings are the VM's lack of a GPU
  (software render), not a panel fault.
- **All pages load and exercise the full D-Bus contract with zero failed calls**
  (no `UnknownMethod`): `IsAuthenticated`, `GetAccountInfo`, `GetQuota`,
  `GetConfig`, `GetSelectedFolders`, `GetRemoteFolderTree`, `GetExclusionPatterns`,
  `Conflicts.List` — confirming the H1/H2 client↔daemon contract is sound at
  runtime, not just at compile time.
- **H2 confirmed live:** the panel received a `QuotaChanged` signal (the
  AccountPage subscription added in H2 fires against a real bus).

A screenshot was not capturable by the agent (GNOME 4x blocks the D-Bus
screenshot method; no screenshot tool installed), so the operator connected via
`virt-viewer` and captured the UI directly (8 screenshots, 2026-05-31).

**Visual verification (operator, via virt-viewer) — all pages render correctly:**

- Shell indicator shows live daemon state — Idle, 7 pending changes, last sync,
  2 conflicts (budget.xlsx, team-notes.docx), Online, 5.0/15.0 GB quota bar
  (confirms H2 end-to-end into the Shell extension too).
- Account page: email, display name, quota bar, Sign Out.
- Sync page: auto-sync, conflict-resolution combo, interval, and the selective
  folder tree with **Photos expanded → "Vacation" subfolder shown checked** —
  the nested-selection display that M1 fixes.
- Conflicts page + "Resolve Conflict" dialog (local vs remote size/mtime/hash,
  Keep Local/Remote).
- Advanced page: exclusion patterns + Add, bandwidth limits.

**Minor cosmetic observation (non-blocking, logged, not fixed):** the "Conflicts"
view-switcher tab is truncated to "Conflicts …" while the other tabs fit — a
`ViewSwitcher` width issue, not a functional defect. Candidate follow-up polish.

Still not exercised (needs non-authenticated mock + a real GOA account): the GOA
onboarding flow `CompleteAuthViaGOA`. Contract is verified (mock + panel + real
daemon agree); the interactive flow is a future check.

## Drift

- Fase 3 scope as written ("implement from a stub") did not match reality (panel
  ~95% built). Re-framed as audit + remediation; Charter row updated.
- G1 dropped from the alpha (deferred to v0.2), reducing "four groups" to three +
  Conflicts. Documented in the AIDEC and Charter.
- An external pre-merge audit of this phase is planned before merge, per the
  operator's phase-scoped external-audit workflow.

## Risk

All changes are in the GTK client; no daemon code changed. H1 realigns the panel
with the (already shipped, audited) RISK-002 daemon API, so it cannot reintroduce
the token-on-bus exposure — it removes the client-side token fetch entirely. The
proxy additions (H2) are declarative. Error surfacing (H3) and the load
reordering (H4) only change UI behaviour. The clippy auto-fix (H5) is mechanical.
No tests broken; the panel has no unit tests (UI), so runtime behaviour rests on
the planned manual verification + external audit.

## Telemetry

| Metric | Value |
|---|---|
| Findings (audit) | 6 (1 High, 3 Medium, 1 Low, 1 gap) + 4 rejected |
| Findings resolved | H1–H5 fixed, G1 deferred |
| Files changed | ~13 (panel) + 3 governance docs |
| New docs | audit, AIDEC, this AILOG |
| clippy lints cleared | 145 needless_borrow + others |
| Daemon code changed | 0 |
| Pre-commit hook failures | none |
