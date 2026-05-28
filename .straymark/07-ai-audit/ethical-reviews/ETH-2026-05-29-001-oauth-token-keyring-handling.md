---
id: ETH-2026-05-29-001
title: OAuth token handling — moving credentials off the public D-Bus surface
status: draft
created: 2026-05-29
agent: claude-opus-4-7-v1.0
confidence: high
review_required: true
risk_level: high
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_security, data_privacy]
iso_42001_clause: [8]
gdpr_legal_basis: contract
fria_required: false
tags: [security, credentials, oauth, keyring, dbus, gdpr]
related:
  - AILOG-2026-05-29-002
  - CHARTER-01-road-to-v0-1-0-alpha-1
  - RISK-002-security-vulns
approved_by: null
approved_date: null
---

# ETH: OAuth token handling — moving credentials off the public D-Bus surface

> **IMPORTANT**: This document is a DRAFT created by an AI agent.
> It requires human review and approval before merging the corresponding
> code change (see `AILOG-2026-05-29-002` for the implementation).

## Executive Summary

The lnxdrive daemon used to accept the user's Microsoft OAuth access
token and refresh token as plain D-Bus method arguments
(`Auth.CompleteAuthWithTokens(access_token, refresh_token, expires_at)`).
Any local process listening on the user's D-Bus session bus could read
those credentials with `dbus-monitor` and impersonate the user against
Microsoft Graph until the refresh token was revoked. The mitigation
landed in `AILOG-2026-05-29-002` removes that public surface, has the
daemon fetch tokens internally from GNOME Online Accounts, and persists
them in the system keyring (`secret-service` / GNOME Keyring / KDE
Wallet, whichever the user has configured).

This is an **ethical review** rather than a pure security review
because the issue concerns user credentials and personal data
(the e-mail address tied to the Microsoft account), so it falls under
GDPR Article 32 ("Security of processing") and the project's own
DOCUMENTATION-POLICY requirement that changes touching credentials
get a human ethical sign-off before merging.

## Context

The lnxdrive daemon is a desktop application that synchronises files
between the user's Microsoft OneDrive account and the local filesystem.
Authentication uses Microsoft's OAuth2 PKCE flow, either initiated by
the in-app browser (`Auth.StartAuth` / `Auth.CompleteAuth`) or
delegated to GNOME Online Accounts so the user can re-use credentials
already on file in the desktop session.

Before this change, the GOA path was implemented by the UI obtaining
the access and refresh tokens from GOA and then passing them as
arguments to the daemon's D-Bus method `Auth.CompleteAuthWithTokens`.
On a Linux session bus those arguments are visible to any process the
user runs (the session bus is intra-user, not inter-user, but any
program the user starts — including malicious ones — can subscribe).
A pre-existing risk analysis recorded the issue as RISK-002 with
CVSS 9.1.

## Ethical concerns and how the change addresses them

### 1. Confidentiality of authentication credentials

**Concern.** OAuth refresh tokens are long-lived bearer credentials.
Their disclosure is materially equivalent to disclosure of the user's
password for the duration of the refresh window. Storing them in cleartext
on the wire — even an intra-user wire — is incompatible with the
"appropriate technical measures" obligation of GDPR Art. 32 and with
the StrayMark `AGENT-RULES.md` rule "Never document credentials,
tokens, API keys, or PII in document content" (interpreted broadly:
the rule's spirit is "never expose them where they don't strictly
need to be").

**Mitigation.** The D-Bus method no longer accepts tokens. The new
method takes only the GOA account D-Bus path (a non-secret identifier).
The daemon fetches the token internally from GOA and immediately
persists it in the system keyring, which is accessible only to the
calling user (via `org.freedesktop.secrets` ACLs).

### 2. Personal data — user e-mail

**Concern.** The new flow returns the user's e-mail address (read from
`org.gnome.OnlineAccounts.Account.PresentationIdentity`) and stores it
both in the daemon's in-memory state (`DaemonState::account_email`)
and as the keyring entry's username key. This is personal data under
GDPR.

**Mitigation.** The processing is necessary for the contract the user
enters with the local application (they explicitly sign in to their
Microsoft account in order for the app to sync their files) — Art. 6(1)(b)
contract basis. The e-mail is not transmitted to any third party that
isn't already part of the OAuth flow (Microsoft Identity itself).
The keyring entry stays on the user's machine. No telemetry, no
analytics. The e-mail appears in logs only at `info!` level inside
the daemon's `tracing` output, which is local-only.

### 3. Resilience to malicious local processes

**Concern.** A malicious local process running under the same user
could still attempt to call `Auth.CompleteAuthViaGOA(arbitrary_path)`
to coerce the daemon into authenticating against an attacker-controlled
GOA account.

**Mitigation (partial).** Two layers reduce the impact: (a) the daemon
validates that the path starts with `/org/gnome/OnlineAccounts/Accounts/`,
so the attacker cannot trick it into talking to a different D-Bus
service; (b) the keyring entry is keyed by the e-mail returned by GOA
itself, so the worst the attacker can do is add a *new* keyring entry
for a *different* account (without affecting the legitimate user's
existing entry). This is acceptable for the alpha; a stronger guard
(D-Bus peer-credentials check restricting `Auth.*` calls to a known
set of UIDs / Flatpak app IDs) is a future hardening tracked outside
this ETH.

### 4. No telemetry, no exfiltration

**Concern.** Any code that handles credentials must be auditable not
to leak them via telemetry, crash dumps or analytics.

**Mitigation.** The daemon emits no telemetry and no crash reporting
in the v0.1.0-alpha cycle (both are explicitly out of scope per
`AILOG-2026-05-29-001`). The `tracing` output is structured but never
emits the access or refresh token (only the GOA path, the e-mail at
`info!`, and the keyring outcome). The `lnxdrive-testing/scripts/leak-test-dbus-tokens.sh`
script validates this at the wire level on every test run.

## GDPR fields

- **Legal basis** (Art. 6): `contract` — the user signs in to use the
  app's sync functionality.
- **Data minimisation** (Art. 5(1)(c)): only the e-mail (account
  identifier) and the access/refresh tokens are stored. No additional
  profile fields are read from GOA.
- **Storage limitation** (Art. 5(1)(e)): the access token has a TTL
  set by Microsoft (~1h); the refresh token is retained until the user
  signs out (`Auth.Logout`) at which point `KeyringTokenStorage::clear`
  removes the entry.
- **Integrity and confidentiality** (Art. 5(1)(f) / Art. 32): tokens
  live only in the system keyring at rest and in-memory at the
  daemon while a sync is active; they never traverse the D-Bus
  session bus as method arguments after this change.
- **DPIA** (Art. 35): not required — local processing, no large-scale
  monitoring, no special categories, single user per installation.

## Open questions for the reviewer

1. **Identity scope choice.** We picked the GOA `PresentationIdentity`
   property as the keyring username. On some GOA providers this is the
   e-mail, on others it may be a display name. Should we instead use
   the GOA `Identity` (UUID-shaped) field for stability across renames,
   even if it makes the keyring entries less human-readable? This
   choice affects how migrations would work later.

2. **Logging of GOA path at `info!`.** The new code logs the full
   GOA path (`/org/gnome/OnlineAccounts/Accounts/1234`) at `info!`
   level. It is not a credential but it does correlate to a specific
   account on the user's machine. Acceptable, or downgrade to `debug!`?

3. **D-Bus peer-credentials restriction.** The current contract lets
   any local process under the user call the `Auth` interface. Should
   v0.1.0-alpha already restrict callers to the lnxdrive UI binaries
   (Flatpak app ID match), or is that a v0.2 hardening?

## Approval

This ETH is `draft`. Approval workflow:

1. The reviewer reads the AILOG-2026-05-29-002 implementation alongside
   this ETH.
2. The reviewer either approves (set `status: approved`, fill
   `reviewed_by`, `reviewed_at`, `review_outcome`, `approved_by`,
   `approved_date`) or requests revisions.
3. The corresponding GitHub PR (closes issue #5) cannot be merged
   without an approved ETH per the project's `AGENT-RULES.md`.
