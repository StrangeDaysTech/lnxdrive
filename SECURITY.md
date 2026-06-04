# Security Policy

## Supported Versions

LNXDrive is in **alpha**. Only the most recent alpha release receives security
fixes; there are no backports.

| Version | Supported |
|---------|-----------|
| 0.1.0-alpha.x (latest) | ✅ |
| anything older | ❌ |

## Reporting a Vulnerability

**Please do not open a public issue for security vulnerabilities.**

1. **Preferred**: use GitHub's private vulnerability reporting —
   [Security → Report a vulnerability](https://github.com/StrangeDaysTech/lnxdrive/security/advisories/new).
2. Alternatively, email **contact@strangedays.tech** with subject
   `[SECURITY] lnxdrive: <short summary>`.

Include: affected component (daemon / FUSE / CLI / GNOME panel / packaging),
reproduction steps, impact assessment, and your environment (distro, GNOME
version, install method).

### What to expect

- **Acknowledgement** within 7 days.
- **Coordinated disclosure**: we ask for up to 90 days to ship a fix before
  public disclosure. Single-maintainer project — complex fixes may need the
  full window.
- Credit in the release notes (opt-out if you prefer anonymity).

## Security posture (alpha)

Documented threat analysis lives in the repository's governance tree
(`.straymark/02-design/risk-analysis/` and `.straymark/08-security/`).
Highlights relevant to users:

- **OAuth tokens never touch disk in cleartext and never travel raw over
  D-Bus**: they live in the system keyring (Secret Service); the D-Bus API
  exposes only opaque session handles.
- The Flatpak sandbox uses **scoped D-Bus names** (no unrestricted
  `--socket=session-bus`).
- The config parser is hardened against YAML expansion attacks
  (billion-laughs caps).
- CI runs `cargo audit` and `cargo deny` on every change to the engine.

Known limitations of the alpha (host-side components outside the Flatpak
sandbox, FUSE device access) are documented in the release notes of each
pre-release.
