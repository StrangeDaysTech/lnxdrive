# LNXDrive Testing Infrastructure

Test infrastructure for the LNXDrive monorepo. Orchestrates build verification, D-Bus integration tests, and visual testing across multiple isolation levels using Podman containers, GNOME nested sessions, and QEMU/libvirt VMs.

## Prerequisites

### Required

- **Podman** (rootless, with `--systemd=always` support)
- **Python 3** with `dbus-next` (`pip install dbus-next`)

### Optional (per test level)

| Dependency | Required for | Install (Fedora) |
|------------|-------------|------------------|
| GJS | D-Bus / Shell extension tests | `sudo dnf install gjs` |
| gnome-shell | Nested sessions | `sudo dnf install gnome-shell` |
| mutter | Nested sessions (GNOME 49+) | `sudo dnf install mutter` |
| qemu-kvm, libvirt | VM testing | `sudo dnf install qemu-kvm libvirt virt-install virt-viewer` |
| cloud-localds or genisoimage | VM cloud-init ISO | `sudo dnf install cloud-utils-growpart` |

Run `make check-deps` to verify what's available on your system.

## Quick Start

```bash
cd lnxdrive-testing

# 1. Check what's available on your host
make check-deps

# 2. Run the full build + test suite in a container (~10-15 min first time)
make build-test

# 3. Or run only D-Bus integration tests
make test-dbus

# 4. For visual testing, open a GNOME nested session
make gnome-nested
```

## Test Levels

The infrastructure provides five progressively more isolated test environments. Choose the level that fits your needs:

### Level 1 — Container Build + Unit Tests

```bash
make build-test            # Full build + unit tests + D-Bus tests in Podman
make build-test-rebuild    # Same, but forces container image rebuild
```

Compiles the Rust daemon (`lnxdrive-engine/`) and GNOME integration (`lnxdrive-gnome/`) inside a Podman container with systemd as PID 1. Runs unit tests, Clippy, and D-Bus integration tests.

### Level 2 — D-Bus Integration Tests

```bash
make test-dbus             # D-Bus tests only
make test-dbus-rebuild     # Same, with container image rebuild
```

Focused D-Bus testing: starts the real daemon via systemd, then runs Shell extension (GJS) and Nautilus extension (Python) tests against both real and mock D-Bus daemons.

### Level 3 — GNOME Nested Session (Host)

```bash
make gnome-nested          # Opens a GNOME Shell window on your desktop
make gnome-nested-build    # Same, also builds extensions before launching
```

Lightweight visual testing. Opens a nested GNOME Shell window on your current desktop with the LNXDrive indicator extension loaded and a mock daemon providing simulated sync statuses. Requires a running GNOME session on the host.

### Level 4 — GNOME Desktop Container (VNC)

```bash
make gnome-container         # Start GNOME desktop in container, VNC on port 5900
make gnome-container-stop    # Stop the container
make gnome-container-status  # Check status
make gnome-container-rebuild # Rebuild and relaunch
```

Full GNOME desktop inside a Podman container, accessible via VNC at `localhost:5900`. More isolated than the nested session — doesn't require GNOME on the host.

Connect: `vncviewer localhost:5900` (password: `testuser`)

### Level 5 — QEMU/libvirt VM (Maximum Isolation)

```bash
make vm-create    # Download Fedora Cloud image, provision VM with cloud-init
make vm-status    # Check VM status
make vm-destroy   # Remove VM and all disk images
```

Creates a full Fedora VM with GNOME desktop, compiles LNXDrive from source via cloud-init, and auto-configures the mock daemon and extensions. The host project directory is shared into the VM via 9p filesystem.

**First-time setup takes ~10-15 minutes** (downloads ~400 MB Fedora Cloud image, installs GNOME desktop, compiles Rust code).

**VM specifications:**
- RAM: 4 GB, CPUs: 2, Disk: 20 GB
- Default credentials: `testuser` / `testuser`

**Connecting to the VM:**

```bash
virt-viewer lnxdrive-test-gnome    # Graphical console
ssh -p 2222 testuser@localhost     # SSH access
```

**How it works:**

1. `create-test-vm.sh` downloads a Fedora Cloud base image (cached for reuse)
2. Creates a qcow2 disk backed by the base image
3. Generates a cloud-init ISO from `vm/cloud-init/user-data` and `vm/cloud-init/meta-data`
4. Creates and starts a QEMU VM via libvirt (user session, no root needed)
5. cloud-init provisions the VM: installs GNOME, Rust, compiles LNXDrive, configures auto-login
6. On reboot, the VM starts a GNOME session with the LNXDrive extension and mock daemon active

## Logs

Test runs produce logs in `logs/` (gitignored). Use the following targets to inspect them:

```bash
make logs          # Summary across all test runs
make logs-latest   # Only the latest run
make logs-detail   # Full detailed output
```

## Cleanup

```bash
make clean               # Remove everything: containers, images, logs, build output
make clean-containers    # Remove test containers only
make clean-images        # Remove container images only
make clean-logs          # Remove test logs only
make clean-build-output  # Remove extracted build artifacts
```

## Project Structure

```
lnxdrive-testing/
├── Makefile                           # Orchestrator — all targets documented above
├── README.md                          # This file
├── scripts/
│   ├── 00-check-deps.sh               # Host dependency checker
│   ├── 01-build-and-test.sh           # Full build + test launcher (Level 1)
│   ├── 02-test-dbus-integration.sh    # D-Bus focused tests (Level 2)
│   ├── 03-gnome-nested-session.sh     # GNOME nested session (Level 3)
│   ├── 04-gnome-desktop-container.sh  # GNOME desktop + VNC (Level 4)
│   ├── container-test-runner.sh       # Entrypoint inside build container
│   └── collect-logs.sh               # Log aggregator
├── containers/
│   ├── Containerfile.build-test       # Fedora + systemd + Rust + meson + GJS
│   └── Containerfile.gnome-desktop    # Full GNOME desktop + VNC
├── vm/
│   ├── create-test-vm.sh              # QEMU VM provisioner (Level 5)
│   └── cloud-init/                    # Cloud-init configuration
│       ├── user-data                  # Provisioning: packages, compilation, services
│       └── meta-data                  # Instance metadata
└── logs/                              # Test run output (gitignored)
    └── .gitkeep
```

## License

GPL-3.0-or-later
