# LNXDrive Testing

Test infrastructure for the [LNXDrive](https://github.com/Strange Days Tech/lnxdrive) ecosystem — a Linux OneDrive sync client.

Orchestrates build verification, D-Bus integration tests, and visual testing across multiple isolation levels using Podman containers, GNOME nested sessions, and QEMU VMs.

## Prerequisites

- **Podman** (rootless, with `--systemd=always` support)
- **Python 3** with `dbus-next` (`pip install dbus-next`)
- **GJS** (GNOME JavaScript runtime)
- **gnome-shell** (for nested sessions)
- **mutter-devel** (GNOME 49+ — provides `mutter-devkit` for nested sessions)

Run `make check-deps` to verify what's available on your system.

## Quick Start

```bash
cd lnxdrive-testing
make check-deps       # Verify host dependencies
make build-test       # Full build + tests in container (~10-15 min first run)
make test-dbus        # D-Bus integration tests only
make gnome-nested     # GNOME nested session for visual testing
```

## Make Targets

### Build & Test

| Target | Description |
|--------|-------------|
| `make build-test` | Full compilation + unit tests + D-Bus tests in Podman container |
| `make build-test-rebuild` | Same as above, but forces container image rebuild |
| `make test-dbus` | D-Bus integration tests only (mock daemon + real daemon) |
| `make test-dbus-rebuild` | Same as above, with container image rebuild |

### Visual Testing

| Target | Description |
|--------|-------------|
| `make gnome-nested` | GNOME Shell nested session on host with mock daemon and test files |
| `make gnome-nested-build` | Same as above, also builds extensions before launching |
| `make gnome-container` | Full GNOME desktop in container with VNC on port 5900 |
| `make gnome-container-stop` | Stop the GNOME desktop container |
| `make gnome-container-status` | Check GNOME desktop container status |
| `make gnome-container-rebuild` | Rebuild and launch GNOME desktop container |

### VM (Maximum Isolation)

| Target | Description |
|--------|-------------|
| `make vm-create` | Create QEMU VM with Fedora Cloud + GNOME + LNXDrive |
| `make vm-destroy` | Destroy the test VM |
| `make vm-status` | Check VM status |

### Logs & Cleanup

| Target | Description |
|--------|-------------|
| `make logs` | Show log summary across all test runs |
| `make logs-latest` | Show only the latest test run logs |
| `make logs-detail` | Detailed log output |
| `make clean` | Remove all containers, images, logs, and build output |
| `make clean-containers` | Remove test containers only |
| `make clean-images` | Remove container images only |
| `make clean-logs` | Remove test logs only |

## Project Structure

```
lnxdrive-testing/
├── Makefile                          # Main orchestrator
├── scripts/
│   ├── 00-check-deps.sh             # Host dependency checker
│   ├── 01-build-and-test.sh         # Full build + test launcher
│   ├── 02-test-dbus-integration.sh  # D-Bus focused tests
│   ├── 03-gnome-nested-session.sh   # GNOME nested session (host)
│   ├── 04-gnome-desktop-container.sh # GNOME desktop + VNC
│   ├── container-test-runner.sh     # Entrypoint inside build container
│   └── collect-logs.sh              # Log aggregator
├── containers/
│   ├── Containerfile.build-test     # Fedora + systemd + Rust + meson + GJS
│   └── Containerfile.gnome-desktop  # Full GNOME desktop + VNC
├── vm/
│   ├── create-test-vm.sh            # QEMU VM provisioner
│   └── cloud-init/                  # Cloud-init user-data and meta-data
└── logs/                            # Test run logs (gitignored)
```

## How It Works

The testing infrastructure validates three sibling repositories:

- **[lnxdrive](https://github.com/Strange Days Tech/lnxdrive)** — Rust daemon (`lnxdrived`) and CLI
- **[lnxdrive-gnome](https://github.com/Strange Days Tech/lnxdrive-gnome)** — GNOME Shell extension, Nautilus extension, and preferences app

### Test Levels

1. **Container tests** (`make build-test`) — Compiles the Rust daemon and runs unit tests, Clippy, and cargo-deny inside a Podman container with systemd. Then runs D-Bus integration tests with mock and real daemons.

2. **D-Bus tests** (`make test-dbus`) — Focused D-Bus testing: starts the real daemon via systemd, runs Shell extension (GJS) and Nautilus extension (Python) tests against both real and mock D-Bus daemons.

3. **GNOME nested session** (`make gnome-nested`) — Opens a GNOME Shell window on your desktop with the LNXDrive indicator extension loaded and a mock daemon providing simulated sync statuses.

4. **GNOME container** (`make gnome-container`) — Full GNOME desktop inside a container, accessible via VNC.

5. **VM** (`make vm-create`) — Maximum isolation using QEMU/libvirt with a full Fedora installation.

## License

GPL-3.0-or-later
