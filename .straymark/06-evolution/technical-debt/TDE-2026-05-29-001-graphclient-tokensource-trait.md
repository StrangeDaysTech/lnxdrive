---
id: TDE-2026-05-29-001
title: Refactor GraphClient to forbid raw access_token in constructor (TokenSource trait)
status: identified
created: 2026-05-29
agent: claude-opus-4-7-v1.0
confidence: high
review_required: false
risk_level: low
type: architecture
impact: low
effort: medium
iso_42001_clause: [8]
tags: [security, defense-in-depth, refactor, graphclient, tokensource]
related:
  - AILOG-2026-05-29-002
  - ETH-2026-05-29-001
  - CHARTER-01-road-to-v0-1-0-alpha-1
priority: medium
assigned_to: null
promoted_from_followup: null
---

# TDE: Refactor `GraphClient` to forbid raw `access_token` in constructor (TokenSource trait)

> **IDENTIFIED BY AGENT**: Prioritization and assignment require human decision.

## Summary

`lnxdrive_graph::client::GraphClient::new(access_token)` and
`GraphClient::with_base_url(access_token, base_url)` accept the OAuth
access token as a constructor argument. Production callers
(`lnxdrive-daemon/src/main.rs`, `lnxdrive-cli/src/commands/sync.rs`)
correctly load the token from the system keyring before constructing
the client, so the invariant "tokens come only from the keyring" is
respected at runtime — but it is not enforced at compile time. A
malicious refactor (or a careless future change) could re-introduce a
code path that constructs `GraphClient` from a token obtained over an
unsafe channel.

The mitigation for **RISK-002** that landed in `AILOG-2026-05-29-002`
removed the D-Bus method that accepted tokens. This TDE proposes the
compile-time complement: replace the `access_token: String` constructor
argument with a `Box<dyn TokenSource>` (or `Arc<dyn TokenSource>`) so
that all GraphClient instances trace their token to a typed source.

## Context

During the audit that produced `AILOG-2026-05-29-001` and the
implementation in `AILOG-2026-05-29-002` we explicitly discussed
whether to introduce the `TokenSource` trait now or defer it. The
operator chose **defer**: the production callers already load tokens
from the keyring, so the additional refactor does not close any new
attack surface today. Filing this TDE is the bookkeeping of that
decision.

The audit found 19 callsites of `GraphClient::{new, with_base_url}`,
of which 17 are tests using hard-coded fake tokens (`"test-token"`,
`"old-token"`, `"expired-token"`, …). The remaining 2 are production
callers in the daemon and CLI.

## Proposed remediation

1. Add a `TokenSource` trait in `lnxdrive-graph` (or `lnxdrive-core`):

   ```rust
   #[async_trait]
   pub trait TokenSource: Send + Sync {
       async fn access_token(&self) -> Result<String>;
   }
   ```

2. Provide two concrete implementations in `lnxdrive-graph`:
   - `KeyringTokenSource { username: String }` — loads the access token
     from the keyring on demand, transparently refreshes via
     `OAuth2Provider::refresh` or `refresh_via_goa` when expired.
   - `StaticTokenSource { token: String }` — `#[cfg(test)]` (or
     gated behind a `unstable-test-utils` feature) for tests.

3. Change `GraphClient::new(access_token)` → `GraphClient::new(source: Arc<dyn TokenSource>)`.
   Update the 2 production callsites; provide a `GraphClient::with_static_token(token)`
   helper to keep the 17 test callsites short.

4. Internally, `GraphClient` calls `source.access_token().await` lazily
   inside `execute_with_retry` rather than caching the string in a
   field. This also closes a smaller debt: the current `GraphClient`
   never refreshes its `access_token` field across the lifetime of an
   instance.

## Why it is debt, not a bug

The runtime invariant "tokens come only from the keyring" holds today.
A future bug that violates it would need to either
(a) ship a new D-Bus method that accepts a raw token (the
`leak-test-dbus-tokens.sh` integration test would fail), or
(b) introduce a new in-process code path that constructs `GraphClient`
from an unsafe string (compile would succeed, no test fails).

This TDE addresses (b) by removing the `String`-accepting constructor
entirely. It is **defense in depth**, not a primary mitigation.

## Why not now

- The current scope of `CHARTER-01-road-to-v0-1-0-alpha-1` is
  release-blocker work. The refactor touches 19 callsites and exercises
  the auth flow end-to-end; a regression in that path is harder to
  debug late in the release cycle than a regression in a D-Bus
  surface change.
- All 17 test callsites would need a parallel migration. With the
  current architecture they pass `"test-token"` as a plain string;
  after the refactor they would either use a helper or wire a
  `StaticTokenSource`. The behaviour is identical, but the churn is
  meaningful for `git blame` clarity.
- The runtime safety we want for v0.1.0-alpha.1 is already provided by
  the keyring + D-Bus method removal. The compile-time enforcement is
  strictly an improvement, not a precondition.

## Activation triggers for this TDE

Any one of the following should promote this debt to active work in a
new Charter:

- A second mitigation finds that `GraphClient` is being constructed
  from a non-keyring source somewhere (proves the runtime invariant
  was already broken).
- A new cloud provider lands (Google Drive, Dropbox) that introduces
  its own client class. Sharing a `TokenSource` trait across providers
  becomes valuable for shared refresh logic.
- The `lnxdrive-graph` crate is split into smaller crates for
  v1.0.0 — natural moment to introduce the abstraction.

## Suggested milestone

`v0.2.0-beta` of LNXDrive. Coupled with the broader CI hardening Fase
(which adds `cargo audit` / `cargo deny` / coverage), since they share
the "defense-in-depth" theme.
