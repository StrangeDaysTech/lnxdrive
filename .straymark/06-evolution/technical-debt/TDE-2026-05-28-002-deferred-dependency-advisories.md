---
id: TDE-2026-05-28-002
title: Remediate deferred RUSTSEC advisories (sqlx 0.8 bump, paste unmaintained)
status: identified
created: 2026-05-28
agent: claude-opus-4-8-v1.0
confidence: high
review_required: false
risk_level: medium
type: dependencies
impact: medium
effort: medium
iso_42001_clause: [8]
tags: [security, supply-chain, cargo-deny, rustsec, sqlx, charter-01]
related:
  - AILOG-2026-05-28-004
  - CHARTER-01-road-to-v0-1-0-alpha-1
priority: medium
assigned_to: null
promoted_from_followup: null
---

# TDE: Remediate deferred RUSTSEC advisories

> **IDENTIFIED BY AGENT**: Prioritization and assignment require human decision.

## Summary

Wiring `cargo deny` in CI (AILOG-2026-05-28-004) surfaced advisories the
CI-hardening PR resolved where cheap, and **deferred** where the fix requires an
out-of-scope change. The deferred ones are allow-listed in
`lnxdrive-engine/deny.toml` with justifications; this TDE tracks their real fix.

| Advisory | Crate | Why deferred | Fix |
|---|---|---|---|
| RUSTSEC-2024-0363 (vuln) | `sqlx 0.7.4` | Fix is `sqlx 0.8.1+`, a **breaking** major bump rippling through `lnxdrive-cache`. The advisory states SQLite (our only backend) "does not appear to be exploitable". | Bump to `sqlx 0.8.x`, migrate `lnxdrive-cache` query/type API, run the cache suite. |
| RUSTSEC-2024-0436 (unmaintained) | `paste 1.x` | Ubiquitous transitive proc-macro; not directly removable. No known vulnerability. | Wait for downstream crates to drop `paste`, or vendor a maintained fork if a vuln is later filed. |

## Resolved in the CI-hardening PR (for the record, not debt)

- `quinn-proto 0.11.13 → 0.11.14`, `rustls-webpki 0.103.9 → 0.103.13`,
  `rand 0.9.2 → 0.9.4` via `cargo update` (cleared 5 advisories).
- `protobuf 2.28` (RUSTSEC-2024-0437 recursion vuln + unmaintained) removed at
  the root by disabling `prometheus` default features (`default-features =
  false`) — `lnxdrive-telemetry` only uses the text registry, not the protobuf
  push gateway.

## Also noted

`rand 0.8.5` (RUSTSEC-2026-0097, "unsound") is pulled by `zbus 4.4`. Not flagged
by cargo-deny's current DB (only cargo-audit's), and only exploitable with a
custom logger calling `rand::rng()` (which we do not). Clearing it needs a
`zbus 5.x` bump. Folded into this TDE.

## Why it is debt, not a bug

Each deferred advisory is either non-exploitable in our usage (sqlx/SQLite,
rand) or carries no known vulnerability (paste/unmaintained). The `deny.toml`
ignores are explicit and justified, so the supply-chain gate is green and
honest rather than silently passing.

## Activation triggers

- A new exploit demonstration is published for any deferred advisory.
- A `sqlx 0.8` migration is scheduled for another reason (e.g. Postgres support).
- `zbus` is bumped to 5.x for the D-Bus Unix-socket fallback (v0.2).

## Suggested milestone

`v0.2.0-beta` (engine-polish / dependency-hygiene pass).
