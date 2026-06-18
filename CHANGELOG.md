# Changelog

All notable changes to LNXDrive are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.1] — date set at tag time (Charter-01 Fase 6)

First public alpha, aimed at Linux power users and GNOME enthusiasts willing
to report bugs. GNOME-only by design — KDE Plasma, GTK3 (XFCE/MATE) and COSMIC
front-ends are archived under `experimental/` until after v1.0.0.

### Added

- **Sync engine** (`lnxdrived`, Rust, 12 crates): Microsoft OneDrive via the
  Graph API with delta synchronization, conflict detection, local file
  watching (inotify) and a D-Bus control interface
  (`com.strangedaystech.LNXDrive`).
- **Files-on-demand**: FUSE filesystem — cloud files are visible locally and
  hydrate on first access; validated with a real-mount performance test
  (`getattr` 43.7 µs, `readdir` 1.40 ms/1000 entries, 37.9 MB idle RSS with
  10k files).
- **CLI** (`lnxdrive`): auth, sync control, status and config commands.
- **GNOME integration**: GTK4/libadwaita preferences panel (Account, Folders,
  Network, Conflicts — wired live to the daemon over D-Bus), GNOME Shell
  status indicator, Nautilus sync-state overlay icons, GNOME Online Accounts
  single sign-on.
- **Flatpak packaging** (`com.strangedaystech.LNXDrive`, GNOME 49 runtime):
  ships daemon + CLI + preferences panel; published as a bundle on GitHub
  Releases with SHA256SUMS via the tag-triggered release workflow.
- `SECURITY.md` with private vulnerability reporting and coordinated
  disclosure policy.

### Security

- OAuth tokens stored in the system keyring (Secret Service) and never sent
  raw over D-Bus — the API exposes only opaque session handles (RISK-002,
  CVSS 9.1, closed).
- FUSE write-during-hydration race serialized with per-inode locking + `EBUSY`
  (RISK-003, closed).
- D-Bus session-bus health monitor with automatic reconnect and interface
  re-registration (RISK-001, mitigated; full Unix-socket fallback deferred
  to v0.2).
- YAML config parser hardened against billion-laughs expansion (ISSUE-002,
  closed).
- `cargo audit` + `cargo deny` enforced in CI.

### Known limitations

- The Flatpak bundle does **not** include the Nautilus extension, the GNOME
  Shell extension or the GOA provider — they load into host processes and
  cannot live inside the sandbox.
- FUSE under the Flatpak sandbox requires `--device=all`; behaviour is
  smoke-tested in VMs before each release.
- System settings group (auto-start, cache, dehydration policy) deferred to
  v0.2 (needs new daemon D-Bus API).

[0.1.0-alpha.1]: https://github.com/StrangeDaysTech/lnxdrive/releases/tag/v0.1.0-alpha.1
