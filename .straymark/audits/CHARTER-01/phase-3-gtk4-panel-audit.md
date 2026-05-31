---
audit_role: internal-calibrated-audit
calibrator: claude-opus-4-8
charter_id: CHARTER-01-road-to-v0-1-0-alpha-1
phase: "Fase 3 — GTK4 preferences panel"
component: lnxdrive-gnome/preferences
audited_at: 2026-05-31
method: 3 parallel Explore agents (D-Bus contract / UI logic / async+build), reconciled and code-verified by the calibrator
findings_consolidated: 6
findings_by_severity:
  high: 1
  medium: 3
  low: 1
  gap: 1
false_positives_rejected: 4
verdict: FUNCTIONAL_WITH_DRIFT
---

# Internal audit — Fase 3 GTK4 preferences panel

**Reviewer:** claude-opus-4-8
**Date:** 2026-05-31
**Confidence:** High
**Component:** `lnxdrive-gnome/preferences/` (binary `lnxdrive-preferences`)

## 1. Executive summary

Fase 3 of Charter-01 is scoped as "implement the GTK4 preferences panel
(currently a `println!("not yet implemented")` stub)". That stub is only the
placeholder `lnxdrive-gnome/src/main.rs`; the **real panel already exists and is
~95% built** under `lnxdrive-gnome/preferences/` — an `adw::Application` with a
typed zbus client, an onboarding wizard, and four pages (Account, Sync,
Conflicts, Advanced). It **compiles** (`cargo check` clean, 12 warnings) and,
unlike the FUSE crate audited in Fase 2, it **does not have a fatal runtime trap**
(async is correctly `async-io` + glib `spawn_local`, no stray `block_on`/
`tokio::spawn`; GSettings schema id/keys, app-id, and build wiring are
consistent).

The audit was run because "compiles" is not "works" — the panel had never been
exercised against a real daemon, and the zbus proxy contract is validated at
**runtime**, not compile time. Three Explore agents (D-Bus contract / UI logic /
async+build) produced findings that the calibrator reconciled and verified
against source, rejecting four agent over-classifications.

**The one serious finding is a cross-component governance drift (H1):** Fase 1
(RISK-002) removed `Auth.CompleteAuthWithTokens` from the daemon and replaced it
with `Auth.CompleteAuthViaGOA` to keep OAuth tokens off the D-Bus surface, but
the **panel was never updated** — it still declares/calls `complete_auth_with_tokens`
(now nonexistent), and that GOA code is behind `#[cfg(feature = "goa")]` while
`Cargo.toml` defines **no `goa` feature**, so GOA SSO is compiled out entirely.
This is the **third occurrence (N=3)** of the "declared but not wired" pattern
already reported upstream to StrayMark (#205) — and the first one that is a
*regression* of a shipped Fase-1 mitigation rather than an original gap.

Mitigating fact: the **manual browser auth path works** (`start_auth()` +
`AuthStateChanged` signal, both present on the daemon — `auth_page.rs:238-295`),
so the panel can still authenticate; only the GOA "use your existing Microsoft
account" path (FR-019–023) is broken. Hence H1 is **High, not Critical**.

**Overall verdict: FUNCTIONAL_WITH_DRIFT.** The panel runs and mostly works; the
material work is fixing the RISK-002 drift, three medium robustness items, lint
cleanup, and the absent "System" group.

## 2. Scope

Audited: every Rust source under `lnxdrive-gnome/preferences/src/`, the zbus
client contract against the daemon's interfaces in
`lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs`, plus `Cargo.toml`,
`meson.build`, the GSettings schema, and the desktop/metainfo files. Not run:
live execution against a mounted daemon (no authenticated account available in
this environment) — deferred to manual verification.

## 3. Findings (calibrated)

### H1 — RISK-002 drift: GOA auth broken & compiled out — **HIGH**

- **Client** declares and calls `complete_auth_with_tokens(access_token,
  refresh_token, expires_at_unix)` — `dbus_client.rs:88,235-243` and
  `onboarding/auth_page.rs:362`.
- **Daemon** removed that method in Fase 1; only `complete_auth(code, state)`
  (`service.rs:873`) and `complete_auth_via_goa(goa_account_path)`
  (`service.rs:917`) exist. `service.rs:902` states it "replaces the historical
  `CompleteAuthWithTokens`"; the tests at `service.rs:2004` confirm it was
  deleted.
- The GOA UI is gated on `#[cfg(feature = "goa")]` (`auth_page.rs:20,35,143,338`)
  but `preferences/Cargo.toml` defines **no `[features]`** → the gate is always
  false → GOA SSO is compiled out (also the source of the `unexpected cfg value
  'goa'` warnings).
- **Impact:** GOA SSO (FR-019–023) is non-functional and, if re-enabled as-is,
  would call a method the daemon no longer exposes (`UnknownMethod` at runtime).
  Manual browser auth is unaffected.
- **Remediation:** (a) add `[features] goa = []` (decide default on/off) to
  `Cargo.toml`; (b) add a `complete_auth_via_goa(goa_account_path)` proxy method
  to `dbus_client.rs` and drop/deprecate `complete_auth_with_tokens`;
  (c) rewrite `auth_page.rs::on_goa_sign_in_clicked` to pass the GOA account
  object path to `complete_auth_via_goa` instead of fetching tokens client-side.

### H2 — Daemon state not consumed (no live status) — **MEDIUM**

- The client proxies omit several daemon-exposed properties/signals: `Sync`
  `sync_status`/`last_sync_time`/`pending_changes` + `sync_started`/
  `sync_completed`/`sync_progress` (`service.rs:652-705`); `Status`
  `connection_status`/`dbus_health` + `quota_changed`/`connection_changed`
  (`service.rs:762-790`); `Settings.config_changed` (`service.rs:1066`).
- **Impact:** the panel shows no live sync/connection status and does not refresh
  on external changes. Functional gap, not a crash.
- **Remediation:** add the missing properties/signals to the proxies and wire a
  minimal set (sync status + quota refresh) into the relevant pages.

### H3 — Silent error handling — **MEDIUM**

- D-Bus call failures go to `eprintln!`/stderr, not the UI (e.g.
  `sync_page.rs:200`, `account_page.rs`), so a dead daemon leaves the panel
  showing default values as if loaded.
- JSON parsing uses `unwrap_or_default()` (`folder_tree.rs:416`), so a malformed
  `GetRemoteFolderTree` response renders an **empty tree indistinguishable from
  "no folders"**.
- **Impact:** silent degradation; user operates on stale/empty UI believing it
  loaded.
- **Remediation:** surface load/save failures via an `adw::Toast`/banner;
  distinguish parse-error from empty in `folder_tree`.

### H4 — `folder_tree` load race — **MEDIUM**

- `FolderTree::new` fires `load_remote_tree()` and `load_selected_folders()` as
  two independent `spawn_local` tasks (`folder_tree.rs:205-206`); `apply_selections`
  can run before the tree is populated, dropping the selection highlight. A
  related issue: `apply_selections` only walks root-level nodes, so lazily-loaded
  children are not marked.
- **Impact:** selective-sync selections may not display correctly.
- **Remediation:** chain selections after the tree populates (await both, or
  apply selections in the populate continuation); apply recursively as nodes
  expand.

### H5 — Compiler warnings — **LOW**

- 12 warnings: unused `gtk4::prelude` imports (`sync_page.rs:11`,
  `onboarding/mod.rs:17`, `app.rs:12`), `unexpected cfg value 'goa'` (resolved by
  H1's feature definition), deprecated `ActionRowBuilder::icon_name`
  (`confirm_page.rs:90,96`).
- **Remediation:** remove unused imports; migrate the deprecated builder call;
  the `cfg` warnings disappear once `goa` is a declared feature.

### G1 — "System" settings group absent — **GAP**

- The Charter names four groups (Account, Folders, Network, System). The panel
  has Account, Sync (≈Folders), Advanced (≈Network), Conflicts — but **no
  "System" group**, and the daemon exposes **no D-Bus API** for its candidate
  settings (auto-start, cache, dehydration policy).
- **Remediation (decision required):** auto-start is implementable without new
  D-Bus API (manage a systemd user unit / autostart `.desktop`); cache and
  dehydration controls need daemon API and are **deferred to v0.2**. Either ship
  a "System" page with only the implementable controls, or document the group as
  deferred. To be decided during remediation.

## 4. Rejected (agent over-classifications)

The calibrator verified and **rejected** these as not-a-bug for this codebase:

- **`.expect()` cascade in GTK factories** (`folder_tree.rs:227,260-318`) — flagged
  CRITICAL by the UI agent, but these are idiomatic gtk4-rs factory closures where
  the item type is guaranteed by construction (`TreeListModel`/`ListItem` always
  yield the registered type). The async+build agent correctly rated them low.
  **Rejected as CRITICAL; at most a stylistic LOW.**
- **`Files` interface missing from client** — the panel is the *preferences* UI;
  pin/unpin/file-status is Nautilus' concern, not this binary's. **Not applicable.**
- **`conflict_list.rs:296` `STRATEGY_VALUES[i]` index** — the two arrays are
  fixed-size consts of equal length; no runtime risk exists today. **LOW, not
  CRITICAL.**
- **async-runtime deadlock (FUSE-style)** — verified absent: zbus uses `async-io`
  (not tokio), all D-Bus calls run via `glib spawn_local`, no `block_on`. **No bug.**

## 5. Remediation plan (→ Fase 3 implementation)

Ordered, each on the `feat/charter-01-phase-3-*` branch with regression coverage
where testable and a closing AILOG:

1. **H1 (High):** define the `goa` feature; replace the client/`auth_page` token
   path with `complete_auth_via_goa`. Backport a governance note (this is a
   Fase-1 RISK-002 regression) and feed the N=3 "declared but not wired" data
   point into the upstream-feedback drafts.
2. **H3 (Medium):** toast/banner on D-Bus errors; parse-error vs empty in
   `folder_tree`.
3. **H4 (Medium):** fix the `folder_tree` load ordering + recursive selection.
4. **H2 (Medium):** extend proxies and wire live sync/quota status.
5. **G1 (Gap):** decide System-group scope; implement auto-start or document
   deferral.
6. **H5 (Low):** clear warnings + deprecation.

Verification: `cargo clippy -p lnxdrive-preferences -- -D warnings` clean; unit
tests for any non-GTK logic added; manual run against a live daemon recorded in
the closing AILOG (the panel cannot be exercised end-to-end in CI — same
`/dev/fuse`/display constraint class as the T101 mount test).
