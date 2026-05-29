---
id: AILOG-2026-05-29-002
title: Mitigate RISK-002 — move OAuth tokens off the public D-Bus surface
status: accepted
created: 2026-05-29
agent: claude-opus-4-7-v1.0
confidence: high
review_required: true
risk_level: high
tags: [security, dbus, auth, oauth, keyring, goa, charter-01, risk-002]
related:
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - AILOG-2026-05-29-001
  - AILOG-2026-02-03-006
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security, data_privacy]
iso_42001_clause: [8]
---

# AILOG: Mitigate RISK-002 — move OAuth tokens off the public D-Bus surface

## Summary

Replaces the vulnerable `Auth.CompleteAuthWithTokens(access_token,
refresh_token, expires_at_unix)` method on the D-Bus interface
`com.strangedaystech.LNXDrive.Auth` with `Auth.CompleteAuthViaGOA(goa_account_path)`,
which only accepts the non-sensitive GNOME Online Accounts D-Bus path.
The daemon now resolves the path to a Microsoft account internally,
fetches tokens from `org.gnome.OnlineAccounts.OAuth2Based.GetAccessToken`,
and persists them in the system keyring through the pre-existing
`KeyringTokenStorage` adapter. Tokens never travel as D-Bus method
arguments anymore.

This closes the highest-severity item in
`.straymark/02-design/risk-analysis/RISK-002-security-vulns.md`
(CVSS 9.1, P0) and is the first batch of `CHARTER-01-road-to-v0-1-0-alpha-1`.

## Context

`RISK-002` documented that the D-Bus `Auth` interface accepted raw OAuth
tokens as method arguments, meaning any local process listening on the
session bus could read `Bearer …` strings with `dbus-monitor`.
`AILOG-2026-02-03-006` had landed the keyring-based storage design
correctly (`KeyringTokenStorage::{store,load,clear}` in
`lnxdrive-graph/src/auth.rs`), but the GOA integration shipped in
PR #2 took a shortcut: it added `CompleteAuthWithTokens` accepting raw
tokens as D-Bus parameters and **never called `KeyringTokenStorage::store`**
— the only side effect was setting `state.is_authenticated = true`.

The four pre-existing unit tests for that method (named
`test_auth_complete_with_tokens_*`) validated the *vulnerable* behaviour
by asserting that the method accepted token strings and updated the
state. They are removed in this change.

The mitigation strategy was chosen with the operator on 2026-05-29:
**minimum viable** — break the public D-Bus surface (the project is
pre-release alpha, no external consumers exist), redirect through a
new method that only takes the GOA account path, and reuse the keyring
storage that is already implemented and exercised by the CLI flow. A
broader refactor (introducing a `TokenSource` trait inside `GraphClient`
so that `access_token` is no longer accepted as a constructor argument
at all) was scoped out of this AILOG and recorded as TDE-001 for a
future iteration. See the "Out of scope" section below.

## Actions performed

### 1. New trait boundary in `lnxdrive-ipc`

Created `lnxdrive-engine/crates/lnxdrive-ipc/src/auth_backend.rs`
defining `trait AuthBackend` (async, Send + Sync) with a single method
`complete_auth_via_goa(&self, goa_account_path: &str) -> AuthBackendResult`.
The error type `AuthBackendError` enumerates the four coarse-grained
failure modes (`InvalidAccount`, `GoaCallFailed`, `KeyringStoreFailed`,
`Internal`); no sensitive material is ever carried in the error.

`AuthInterface` (the zbus `#[interface]` implementation) now holds an
`Option<Arc<dyn AuthBackend>>`. Two constructors:

- `AuthInterface::new(state)` — backend `None`, used by unit tests that
  do not exercise the GOA path.
- `AuthInterface::with_backend(state, backend)` — production wiring.

`DbusService` gained a fluent setter `with_auth_backend(Arc<dyn AuthBackend>)`
and threads the backend through the interface registration in
`DbusService::start()`. When the backend is absent the service still
starts but `CompleteAuthViaGOA` returns `false` and a warning is logged.

### 2. Public D-Bus method swap

`lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs`:

- **Removed** `async fn complete_auth_with_tokens(access_token, refresh_token, expires_at_unix)`.
  This is a deliberate breaking change to the D-Bus contract. No
  external consumers existed before v0.1.0-alpha.1 ships; the GOA
  integration in PR #2 is the only known caller and is updated in this
  change.
- **Added** `async fn complete_auth_via_goa(goa_account_path: String) -> bool`.
  The method validates the path prefix locally, then delegates to the
  configured `AuthBackend`. On success it updates `state.is_authenticated`,
  `state.account_email`, and `state.auth_source = Some("goa")`; on failure
  it logs the backend error (which does not carry tokens) and returns
  `false`. No payload of the call is logged at info level.

### 3. Production backend in `lnxdrive-daemon`

Created `lnxdrive-engine/crates/lnxdrive-daemon/src/goa_auth_backend.rs`.
`GoaAuthBackend` implements `AuthBackend` by:

1. Validating that the path begins with `/org/gnome/OnlineAccounts/Accounts/`.
2. Calling `org.gnome.OnlineAccounts.OAuth2Based.GetAccessToken` on the
   account path to obtain the access token + `expires_in` directly from
   GOA — no caller passes the token.
3. Calling `org.freedesktop.DBus.Properties.Get` on the
   `org.gnome.OnlineAccounts.Account` interface to read the
   `PresentationIdentity` property (the user e-mail).
4. Building a `lnxdrive_core::ports::Tokens { access_token, refresh_token: None, expires_at }`
   and persisting it through
   `lnxdrive_graph::auth::KeyringTokenStorage::store(&email, &tokens)`.
5. Returning the e-mail to the caller. **No tokens are returned, logged
   at info level, or sent back over D-Bus.**

`lnxdrive-daemon/src/main.rs` wires the backend at daemon startup:

```rust
let dbus_service = DbusService::new(Arc::clone(&self.daemon_state))
    .with_auth_backend(Arc::new(GoaAuthBackend::new()));
```

### 4. Tests

`lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs` test module:

- **Removed** the four vulnerable tests
  `test_auth_complete_with_tokens_*` that asserted the bug behaviour
  (accepting tokens as parameters and toggling state without keyring
  persistence).
- **Added** four new tests against `complete_auth_via_goa`:
  - `test_auth_complete_via_goa_succeeds_when_backend_returns_email`
  - `test_auth_complete_via_goa_rejects_invalid_path_before_calling_backend`
  - `test_auth_complete_via_goa_without_backend_returns_false`
  - `test_auth_complete_via_goa_propagates_backend_failure`
- Introduced a `MockAuthBackend` in the test module that captures the
  last call (so we can assert that backend invocation is skipped for
  invalid paths) and returns a configurable `Ok`/`Err`.

Test totals after the change:

- `cargo test -p lnxdrive-ipc` → **71 passed, 0 failed** (was 71 passed before, but 4 of them were validating the bug; net is the same count, with the 4 vulnerable tests replaced by 4 secure ones).
- `cargo test -p lnxdrive-daemon` → **6 passed** (5 pre-existing + 1 new `goa_auth_backend::tests::rejects_non_goa_path`).
- `cargo test --workspace` → 218 passed, 1 pre-existing failure unrelated to this change (`lnxdrive-core::config::tests::default_path_ends_with_config_yaml` fails on `main` too, sensitive to the cwd of `cargo test`; tracked as a separate TDE).

### 5. Integration leak test

Added `lnxdrive-testing/scripts/leak-test-dbus-tokens.sh`. The script
launches the daemon inside a `dbus-run-session`, captures all session
bus traffic with `dbus-monitor` while exercising `Auth.StartAuth`,
`Auth.CompleteAuthWithTokens` (which now returns `UnknownMethod` — the
positive regression signal) and `Auth.CompleteAuthViaGOA` with a fake
account path, then `grep`s the trace for `Bearer `, JWT-shaped strings
(`eyJ[A-Za-z0-9_\-]{20,}`), `refresh_token` and `access_token`.

Token-shaped strings the operator's own calls send (as request arguments)
are filtered out before the assertion; the assertion runs only on
*reply* messages (signals, method_return, error). That way the
regression signal is "the daemon parrots tokens back" rather than
"the operator sent strings that look like tokens", which is the bug we
actually care about.

The script is invoked manually for now (`bash lnxdrive-testing/scripts/leak-test-dbus-tokens.sh`)
and is wired into CI in a follow-up PR alongside `cargo test --workspace`
(part of Fase 2 of `CHARTER-01`).

## Out of scope (recorded ex-ante so the drift gate ignores them)

- **`TokenSource` trait for `GraphClient`.** During scoping with the
  operator we considered making `GraphClient::new` resolve the token
  internally through a trait abstraction so that callers cannot pass a
  raw token at all. The audit revealed that `GraphClient::new(&token)`
  is only called from production code that has already loaded the token
  from the keyring (`daemon/main.rs:183`, `cli/commands/sync.rs:101`),
  so the additional refactor does not close any new attack surface —
  it would only enforce the invariant at compile time. It is recorded
  as **TDE-001** in `.straymark/06-evolution/technical-debt/` and as
  a separate GitHub issue under milestone `v0.2.0-beta`.

- **CI integration of the leak test.** Wiring
  `leak-test-dbus-tokens.sh` into `lnxdrive-engine/.github/workflows/ci.yml`
  lands with the broader CI hardening of Fase 2 (which also turns on
  `cargo test --workspace`, `cargo audit` and `cargo deny`). Doing it
  here would bloat this PR with workflow plumbing.

- **`AGENT-RULES.md` § Identity update for `agent: claude-opus-4-7-v1.0`.**
  The current StrayMark template still suggests `claude-code-v1.0` as
  the canonical identifier. Continuing to use the actual model
  identifier per Anthropic's guidance; alignment with the template is
  cosmetic.

## Drift

> Added 2026-05-28 during the Fase-1 external audit (two auditors independently
> flagged the scope-vs-implementation gap). Recorded here explicitly and the
> Charter `## Files to modify` RISK-002 row backported in the same change.

- **`SessionHandle` (scoped) → `CompleteAuthViaGOA` + internal keyring (built).**
  The Charter §Scope and its `## Files to modify` table specified that the D-Bus
  interface would expose "opaque `SessionHandle` IDs" via a new
  `lnxdrive-daemon/src/dbus_iface.rs`. The shipped design does **not** issue a
  handle: `Auth.CompleteAuthViaGOA(goa_account_path)` returns a `bool`, and the
  token is fetched from GOA and stored in the keyring by `GoaAuthBackend`
  (`goa_auth_backend.rs`) — the interface lives in the existing
  `lnxdrive-ipc/src/service.rs`, not a new `dbus_iface.rs`. This is
  **security-equivalent** to the SessionHandle design for the stated threat
  (tokens never cross the D-Bus surface) and was the deliberate minimum-viable
  choice recorded in the Context section above. No `SessionHandle` type exists
  in the codebase. The compile-time `TokenSource` hardening is deferred to
  TDE-2026-05-29-001. The Charter table row was not updated atomically at the
  time (the omission the audit caught); it is corrected in the audit-remediation
  change that adds this note.

## Risks

- **R1 — Breaking the public D-Bus contract.** Probability low,
  severity low.
  Mitigation: this is a pre-release alpha. No published consumer
  outside the monorepo exists. The release notes for `v0.1.0-alpha.1`
  will document the contract.
- **R2 — GOA `Properties.Get(PresentationIdentity)` may return a value
  that is not exactly the user e-mail on some providers (it can be
  `user@host` styled, or a friendly display name).** Probability
  medium, severity medium.
  Mitigation: the keyring entry is keyed by whatever GOA returns; the
  daemon stores and reads the same string consistently. If we later
  discover that GOA returns a non-email identity for some providers,
  we add a normalisation step (or switch to `Identity` / `Id` property)
  in a follow-up. The risk is captured because it can surface during
  the `lnxdrive-testing/` E2E smoke test, not during unit tests.
- **R3 — `Auth.CompleteAuthViaGOA` returning the same boolean shape as
  the old method may mask a backend failure as "completed but not
  signed-in".** Probability low, severity low.
  Mitigation: the new method updates `state.is_authenticated` *only*
  after the backend returns `Ok`, so a `false` return guarantees that
  `is_authenticated` was not toggled. UI callers that observed the old
  semantics will see `false` whenever GOA or the keyring fail — strictly
  better than the previous behaviour, which set
  `is_authenticated = true` regardless of token validity.

## Verification

### Local checks

```bash
# Workspace-level build
cargo build -p lnxdrive-ipc -p lnxdrive-daemon \
    --manifest-path lnxdrive-engine/Cargo.toml

# Unit tests for the touched crates
cargo test -p lnxdrive-ipc -p lnxdrive-daemon \
    --manifest-path lnxdrive-engine/Cargo.toml

# (Optional) full workspace — note the pre-existing
# config::tests::default_path_ends_with_config_yaml failure that fails
# on main too; not a regression of this change.
cargo test --workspace \
    --manifest-path lnxdrive-engine/Cargo.toml

# Static rejection of the removed method (with the daemon running):
gdbus introspect --session \
    --dest com.strangedaystech.LNXDrive \
    --object-path /com/strangedaystech/LNXDrive \
    | grep -E 'CompleteAuthWithTokens|CompleteAuthViaGOA'
# Expected: CompleteAuthViaGOA appears, CompleteAuthWithTokens does not.
```

### Production smoke (after deploy)

```bash
# Leak test — fails if any token-shaped string appears in D-Bus replies.
cargo build -p lnxdrive-daemon --manifest-path lnxdrive-engine/Cargo.toml
bash lnxdrive-testing/scripts/leak-test-dbus-tokens.sh

# Manual E2E with a real Microsoft account configured in GOA:
gdbus call --session \
    --dest com.strangedaystech.LNXDrive \
    --object-path /com/strangedaystech/LNXDrive \
    --method com.strangedaystech.LNXDrive.Auth.CompleteAuthViaGOA \
    "/org/gnome/OnlineAccounts/Accounts/<your-account-id>"
# Expected: returns true. Then:
secret-tool search --all service lnxdrive
# Expected: an entry under user@yourdomain with the token JSON.
```

## Follow-up

- **TDE-001**: refactor `GraphClient` to forbid raw `access_token` in
  the constructor and resolve tokens via a `TokenSource` trait.
  Milestone `v0.2.0-beta`. GitHub issue link to follow when the TDE
  is filed.
- **CI integration of the leak test**: lands with Fase 2 CI hardening
  PR.
- **Closes GitHub issue #5** (`OAuth tokens visible in DBus traffic`,
  `priority/P0`, milestone `v0.1.0-alpha`).
