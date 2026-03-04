# LNXDrive Testing Makefile
#
# Orchestrates build, test, and visual testing across multiple isolation levels:
#   - Container (Podman + systemd) for compilation, unit tests, D-Bus tests
#   - GNOME nested session (host) for quick visual testing
#   - GNOME desktop container (Podman + VNC) for robust visual testing
#   - VM (QEMU/libvirt) for full end-to-end testing
#
# Usage:
#   make check-deps       # Verify host dependencies
#   make build-test       # Full build + tests in container
#   make test-dbus        # D-Bus integration tests only
#   make gnome-nested     # GNOME nested session on host
#   make gnome-container  # GNOME desktop via VNC
#   make vm-create        # Create QEMU VM with GNOME
#   make logs             # Show log summary
#   make clean            # Remove containers, images, logs

.PHONY: check-deps \
        build-test test-dbus \
        gnome-nested gnome-container gnome-container-stop \
        vm-create vm-destroy vm-status \
        logs logs-latest logs-detail \
        clean clean-containers clean-images clean-logs

SHELL := /bin/bash

# ==============================================================================
# Dependency Verification
# ==============================================================================

check-deps:
	@bash scripts/00-check-deps.sh

# ==============================================================================
# Container: Full Build + Tests (Level 1+2)
# ==============================================================================

build-test:
	@bash scripts/01-build-and-test.sh

build-test-rebuild:
	@bash scripts/01-build-and-test.sh --rebuild

# ==============================================================================
# Container: D-Bus Integration Tests Only
# ==============================================================================

test-dbus:
	@bash scripts/02-test-dbus-integration.sh

test-dbus-rebuild:
	@bash scripts/02-test-dbus-integration.sh --rebuild

# ==============================================================================
# GNOME Nested Session (Host - Lightweight)
# ==============================================================================

gnome-nested:
	@bash scripts/03-gnome-nested-session.sh

gnome-nested-build:
	@bash scripts/03-gnome-nested-session.sh --build

# ==============================================================================
# GNOME Desktop Container (VNC - Robust)
# ==============================================================================

gnome-container:
	@bash scripts/04-gnome-desktop-container.sh

gnome-container-stop:
	@bash scripts/04-gnome-desktop-container.sh --stop

gnome-container-status:
	@bash scripts/04-gnome-desktop-container.sh --status

gnome-container-rebuild:
	@bash scripts/04-gnome-desktop-container.sh --rebuild

# ==============================================================================
# VM: QEMU/libvirt (Maximum Isolation)
# ==============================================================================

vm-create:
	@bash vm/create-test-vm.sh

vm-destroy:
	@bash vm/create-test-vm.sh --destroy

vm-status:
	@bash vm/create-test-vm.sh --status

# ==============================================================================
# Logs
# ==============================================================================

logs:
	@bash scripts/collect-logs.sh

logs-latest:
	@bash scripts/collect-logs.sh --latest

logs-detail:
	@bash scripts/collect-logs.sh --detail

# ==============================================================================
# Cleanup
# ==============================================================================

clean-containers:
	@echo "Stopping and removing test containers..."
	@podman stop lnxdrive-build-test 2>/dev/null || true
	@podman rm lnxdrive-build-test 2>/dev/null || true
	@podman stop lnxdrive-dbus-test 2>/dev/null || true
	@podman rm lnxdrive-dbus-test 2>/dev/null || true
	@podman stop lnxdrive-build-extract 2>/dev/null || true
	@podman rm lnxdrive-build-extract 2>/dev/null || true
	@podman stop lnxdrive-gnome-desktop 2>/dev/null || true
	@podman rm lnxdrive-gnome-desktop 2>/dev/null || true
	@echo "Done."

clean-images:
	@echo "Removing container images..."
	@podman rmi localhost/lnxdrive-build-test 2>/dev/null || true
	@podman rmi localhost/lnxdrive-gnome-desktop 2>/dev/null || true
	@echo "Done."

clean-logs:
	@echo "Removing logs..."
	@rm -rf logs/build-* logs/dbus-* logs/gnome-nested-* logs/gnome-desktop-*
	@rm -f logs/summary-*.txt
	@echo "Done."

clean-build-output:
	@echo "Removing build output..."
	@rm -rf build-output/
	@echo "Done."

clean: clean-containers clean-images clean-logs clean-build-output
	@echo "All testing artifacts cleaned."
