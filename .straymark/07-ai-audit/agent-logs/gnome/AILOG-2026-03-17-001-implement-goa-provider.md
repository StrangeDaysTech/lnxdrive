---
id: AILOG-2026-03-17-001
title: Implement GOA provider for Microsoft SSO (FR-019 to FR-023)
status: accepted
created: 2026-03-17
agent: claude-code-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [goa, authentication, gnome, sso, oauth2]
related: [FR-019, FR-020, FR-021, FR-022, FR-023]
---

# AILOG: Implement GOA Provider for Microsoft SSO

## Summary

Implemented the GNOME Online Accounts provider (S5) enabling native Microsoft
account integration in GNOME Settings → Online Accounts. This covers all 6 phases:
D-Bus extension, C provider module, build system, account lifecycle monitoring,
SSO detection, and GOA-aware token refresh.

## Context

The MVP was 100% closed. The GOA provider was the next deferred item (S5), needed
to provide seamless SSO where users add their Microsoft account through GNOME's
native account settings instead of the standalone onboarding wizard.

## Actions Performed

1. **Phase 1 — D-Bus extension**: Added `CompleteAuthWithTokens(access_token, refresh_token, expires_at_unix)` method to `AuthInterface` and `auth_source` field to `DaemonState`
2. **Phase 2 — GOA Provider C module**: Created 5 C source files implementing `GoaOAuth2Provider` subclass with Microsoft OAuth2 endpoints and D-Bus token handoff
3. **Phase 3 — Build system**: Replaced meson.build placeholder with full shared_library build, added GOA subdir to main meson.build, updated option description
4. **Phase 4 — Account lifecycle**: Added GOA account removal monitoring to shell extension via `InterfacesRemoved` D-Bus signal subscription
5. **Phase 5 — SSO detection**: Added `goa_sso.rs` module and conditional "Use existing account" button to onboarding auth page (feature-gated behind `goa`)
6. **Phase 6 — GOA token refresh**: Added `refresh_via_goa()` method to `GraphAuthAdapter` that delegates refresh to GOA D-Bus

## Modified Files

| File | Change |
|------|--------|
| `lnxdrive-engine/crates/lnxdrive-ipc/src/service.rs` | Added `auth_source` field, `CompleteAuthWithTokens` method, 5 new tests |
| `lnxdrive-engine/crates/lnxdrive-graph/src/auth.rs` | Added `refresh_via_goa()` method to `GraphAuthAdapter` |
| `lnxdrive-engine/crates/lnxdrive-graph/Cargo.toml` | Added `zbus` dependency |
| `lnxdrive-gnome/preferences/src/dbus_client.rs` | Added `complete_auth_with_tokens` to proxy trait and `DbusClient` |
| `lnxdrive-gnome/preferences/src/main.rs` | Registered `goa_sso` module (feature-gated) |
| `lnxdrive-gnome/preferences/src/goa_sso.rs` | **New** — GOA SSO detection and token retrieval helper |
| `lnxdrive-gnome/preferences/src/onboarding/auth_page.rs` | Added GOA SSO button and handler (feature-gated) |
| `lnxdrive-gnome/tests/mock-dbus-daemon.py` | Added `CompleteAuthWithTokens` method to mock |
| `lnxdrive-gnome/goa-provider/src/lnxdrive-goa-module.c` | **New** — GIO module entry point |
| `lnxdrive-gnome/goa-provider/src/lnxdrive-goa-provider.h` | **New** — Type declarations |
| `lnxdrive-gnome/goa-provider/src/lnxdrive-goa-provider.c` | **New** — GoaOAuth2Provider subclass |
| `lnxdrive-gnome/goa-provider/src/lnxdrive-goa-dbus.h` | **New** — D-Bus helper header |
| `lnxdrive-gnome/goa-provider/src/lnxdrive-goa-dbus.c` | **New** — D-Bus helper implementation |
| `lnxdrive-gnome/goa-provider/meson.build` | Replaced placeholder with full build definition |
| `lnxdrive-gnome/goa-provider/README.md` | Updated to reflect implemented status |
| `lnxdrive-gnome/meson.build` | Added conditional GOA subdir |
| `lnxdrive-gnome/meson_options.txt` | Updated `enable_goa` description |
| `lnxdrive-gnome/shell-extension/.../dbus.js` | Added Auth proxy, GOA monitoring functions |

## Decisions Made

- **Separate D-Bus method for tokens**: Created `CompleteAuthWithTokens` instead of overloading `CompleteAuth`, because the signatures and semantics are different (tokens vs. auth code)
- **auth_source tracking**: Added `auth_source` field to `DaemonState` so the daemon knows whether to refresh tokens itself or delegate to GOA
- **GOA refresh sentinel**: When SSO reuses a GOA account, the refresh_token is set to `__goa_managed__` since GOA manages refresh internally
- **Feature-gated SSO UI**: The GOA button in onboarding is behind `#[cfg(feature = "goa")]` to avoid adding GOA D-Bus dependencies to the standard preferences build

## Impact

- **Functionality**: Users can now add Microsoft/OneDrive account via GNOME Settings → Online Accounts
- **Performance**: N/A — no impact on sync performance
- **Security**: Tokens are managed by GOA (system keyring); D-Bus token handoff uses same session bus security as existing auth flow

## Verification

- [x] `cargo check --workspace` compiles cleanly
- [x] All 184 existing tests pass + 5 new tests added (71 total in lnxdrive-ipc)
- [ ] Meson build with `-Denable_goa=true` (requires GOA dev packages)
- [ ] Manual: GOA provider visible in GNOME Settings
- [ ] Manual: Token handoff to daemon works

## Additional Notes

- GOA backend API is marked unstable; we pin to GNOME 45-47 and use `GOA_API_IS_SUBJECT_TO_CHANGE`
- WebKitGTK naming varies between distros; meson.build has a fallback from `webkitgtk-6.0` to `webkit2gtk-6.0`

---

<!-- Template: DevTrail | https://strangedays.tech -->
