#!/usr/bin/env bash
# container-test-runner.sh - Runs inside the build-test container as root
#
# Sequentially: build lnxdrive, unit tests, build lnxdrive-gnome,
# install daemon, run functional tests, run D-Bus integration tests.
#
# This script runs as root. It uses `su - testuser` for operations that
# require the systemd user session (D-Bus, systemctl --user).
#
# Expected environment:
#   /src/lnxdrive       - mounted read-only
#   /src/lnxdrive-gnome - mounted read-only
#   /build              - writable build area
#   /logs               - writable log output
#   TESTUSER_UID        - UID of testuser (for D-Bus env)

set -uo pipefail

# --- Colors and formatting ---------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

PASS_COUNT=0
FAIL_COUNT=0
TOTAL_STEPS=0

step_pass() {
    echo -e "  ${GREEN}[PASS]${NC} $1"
    PASS_COUNT=$((PASS_COUNT + 1))
}

step_fail() {
    echo -e "  ${RED}[FAIL]${NC} $1"
    FAIL_COUNT=$((FAIL_COUNT + 1))
}

step_info() {
    echo -e "  ${YELLOW}[INFO]${NC} $1"
}

run_step() {
    local name="$1"
    local log_file="$2"
    shift 2
    TOTAL_STEPS=$((TOTAL_STEPS + 1))
    echo ""
    echo -e "${BOLD}Step ${TOTAL_STEPS}: ${name}${NC}"

    if "$@" > "$log_file" 2>&1; then
        step_pass "$name"
        return 0
    else
        step_fail "$name (see $log_file)"
        tail -20 "$log_file" 2>/dev/null || true
        return 1
    fi
}

# Run a command as testuser with D-Bus session environment
run_as_testuser() {
    local uid="${TESTUSER_UID:-1000}"
    su - testuser -c "
        export XDG_RUNTIME_DIR=/run/user/${uid}
        export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/${uid}/bus
        export PATH=/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin
        $*
    "
}

# --- Pre-flight ---------------------------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive Full Build & Test Runner${NC}"
echo "==================================="
echo ""
step_info "User: $(whoami)"
step_info "TESTUSER_UID: ${TESTUSER_UID:-not set}"
echo ""

# Ensure source directories exist
if [ ! -f /src/lnxdrive/Cargo.toml ]; then
    echo -e "${RED}ERROR: /src/lnxdrive not mounted or invalid${NC}"
    exit 1
fi
if [ ! -f /src/lnxdrive-gnome/meson.build ]; then
    echo -e "${RED}ERROR: /src/lnxdrive-gnome not mounted or invalid${NC}"
    exit 1
fi

# Prepare build directories (copy sources since cargo needs writable target/)
cp -a /src/lnxdrive /build/lnxdrive
cp -a /src/lnxdrive-gnome /build/lnxdrive-gnome
chown -R testuser:testuser /build

# Ensure /logs is writable
chmod 777 /logs 2>/dev/null || true

# Wait for testuser's systemd user session and D-Bus socket
wait_for_user_session() {
    local uid="${TESTUSER_UID:-1000}"
    local socket="/run/user/${uid}/bus"
    local max_wait=30
    local waited=0

    step_info "Waiting for testuser session bus at ${socket}..."
    while [ ! -S "$socket" ] && [ $waited -lt $max_wait ]; do
        sleep 1
        waited=$((waited + 1))
    done

    if [ -S "$socket" ]; then
        step_info "D-Bus session bus ready (${waited}s)"
        return 0
    else
        step_info "D-Bus session bus not found after ${max_wait}s"
        step_info "Attempting to start user session manually..."
        loginctl enable-linger testuser 2>/dev/null || true
        sleep 3
        if [ -S "$socket" ]; then
            step_info "D-Bus session bus ready (manual linger)"
            return 0
        fi
        step_info "WARNING: D-Bus session bus still not available"
        return 1
    fi
}

wait_for_user_session

# Crates excluded from default build (matching lnxdrive/Makefile, phase 4+)
EXCLUDE_STUBS="--exclude lnxdrive-conflict --exclude lnxdrive-audit --exclude lnxdrive-telemetry"

# ============================================================================
# PHASE 1: Build lnxdrive daemon
# ============================================================================

echo ""
echo -e "${BOLD}=== PHASE 1: Build lnxdrive ===${NC}"

run_step "Cargo build lnxdrive (workspace)" \
    "/logs/cargo-build-lnxdrive.log" \
    cargo build --manifest-path /build/lnxdrive/Cargo.toml --workspace $EXCLUDE_STUBS

# ============================================================================
# PHASE 2: Unit tests lnxdrive
# ============================================================================

echo ""
echo -e "${BOLD}=== PHASE 2: Unit tests lnxdrive ===${NC}"

run_step "Cargo test lnxdrive (workspace)" \
    "/logs/cargo-test-lnxdrive.log" \
    cargo test --manifest-path /build/lnxdrive/Cargo.toml --workspace $EXCLUDE_STUBS || true

# ============================================================================
# PHASE 3: Build lnxdrive-gnome
# ============================================================================

echo ""
echo -e "${BOLD}=== PHASE 3: Build lnxdrive-gnome ===${NC}"

run_step "Meson setup lnxdrive-gnome" \
    "/logs/meson-setup-gnome.log" \
    meson setup /build/gnome-builddir /build/lnxdrive-gnome --prefix=/usr

run_step "Meson compile lnxdrive-gnome" \
    "/logs/meson-compile-gnome.log" \
    meson compile -C /build/gnome-builddir

# ============================================================================
# PHASE 4: Build preferences app
# ============================================================================

echo ""
echo -e "${BOLD}=== PHASE 4: Build preferences ===${NC}"

run_step "Cargo build preferences" \
    "/logs/cargo-build-preferences.log" \
    cargo build --manifest-path /build/lnxdrive-gnome/preferences/Cargo.toml

# ============================================================================
# PHASE 5: Install & test daemon via systemd
# ============================================================================

echo ""
echo -e "${BOLD}=== PHASE 5: Daemon functional tests ===${NC}"

# Install daemon binaries (as root)
DAEMON_BIN="/build/lnxdrive/target/debug/lnxdrived"
CLI_BIN="/build/lnxdrive/target/debug/lnxdrive"

if [ -f "$DAEMON_BIN" ]; then
    cp "$DAEMON_BIN" /usr/local/bin/lnxdrived
    chmod +x /usr/local/bin/lnxdrived
    step_info "Installed lnxdrived to /usr/local/bin/"
else
    step_info "lnxdrived binary not found (build may have failed)"
fi

if [ -f "$CLI_BIN" ]; then
    cp "$CLI_BIN" /usr/local/bin/lnxdrive
    chmod +x /usr/local/bin/lnxdrive
    step_info "Installed lnxdrive CLI to /usr/local/bin/"
fi

# Copy default config
cp /src/lnxdrive/config/default-config.yaml /home/testuser/.config/lnxdrive/config.yaml
chown testuser:testuser /home/testuser/.config/lnxdrive/config.yaml

# Create systemd user service
cat > /home/testuser/.config/systemd/user/lnxdrive.service <<'EOF'
[Unit]
Description=LNXDrive OneDrive Sync Service
After=dbus.service

[Service]
Type=simple
ExecStart=/usr/local/bin/lnxdrived
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
EOF
chown testuser:testuser /home/testuser/.config/systemd/user/lnxdrive.service

# Enable the service via symlink
mkdir -p /home/testuser/.config/systemd/user/default.target.wants
ln -sf /home/testuser/.config/systemd/user/lnxdrive.service \
       /home/testuser/.config/systemd/user/default.target.wants/lnxdrive.service
chown -R testuser:testuser /home/testuser/.config/systemd

# Copy and run functional test script (runs as testuser for systemd --user)
if [ -f /src/lnxdrive/scripts/test-daemon-functional.sh ]; then
    cp /src/lnxdrive/scripts/test-daemon-functional.sh /usr/local/bin/test-daemon-functional.sh
    chmod +x /usr/local/bin/test-daemon-functional.sh
    run_step "Daemon functional tests (8 checks)" \
        "/logs/daemon-functional.log" \
        run_as_testuser /usr/local/bin/test-daemon-functional.sh
else
    step_info "test-daemon-functional.sh not found, skipping"
fi

# ============================================================================
# PHASE 6: D-Bus integration tests with mock daemon
# ============================================================================

echo ""
echo -e "${BOLD}=== PHASE 6: D-Bus integration tests (mock daemon) ===${NC}"

# Stop real daemon first (as testuser)
run_as_testuser "systemctl --user stop lnxdrive" 2>/dev/null || true
sleep 2

# The Nautilus test manages its own mock daemon (with its own sync root),
# so it must run BEFORE the runner starts the shared mock daemon.
MOCK_DAEMON="/src/lnxdrive-gnome/tests/mock-dbus-daemon.py"
NAUTILUS_TEST="/src/lnxdrive-gnome/tests/test-nautilus-extension.py"
SHELL_TEST="/src/lnxdrive-gnome/tests/test-shell-extension.js"

# Run Nautilus extension tests first (starts/stops its own mock daemon)
if [ -f "$NAUTILUS_TEST" ] && [ -f "$MOCK_DAEMON" ]; then
    run_step "Nautilus extension D-Bus tests (mock)" \
        "/logs/test-nautilus-mock.log" \
        run_as_testuser "python3 '$NAUTILUS_TEST'"
fi

# Start mock daemon for Shell extension tests
if [ -f "$MOCK_DAEMON" ]; then
    step_info "Starting mock D-Bus daemon..."
    run_as_testuser "python3 '$MOCK_DAEMON' --authenticated --signal-interval 999 \
        --sync-root /home/testuser/OneDrive \
        > /tmp/mock-daemon.log 2>&1 &"
    sleep 3

    # Copy mock daemon log to /logs
    cp /tmp/mock-daemon.log /logs/mock-daemon.log 2>/dev/null || true

    # Run Shell extension tests (as testuser for D-Bus access)
    if [ -f "$SHELL_TEST" ]; then
        run_step "Shell extension D-Bus tests (mock)" \
            "/logs/test-shell-mock.log" \
            run_as_testuser "gjs -m '$SHELL_TEST'"
    fi

    # Stop mock daemon
    run_as_testuser "pkill -f mock-dbus-daemon" 2>/dev/null || true
    sleep 1
    step_info "Mock daemon stopped"

    # Copy updated mock daemon log
    cp /tmp/mock-daemon.log /logs/mock-daemon.log 2>/dev/null || true

    # Run Shell extension tests without daemon (as testuser for D-Bus access)
    if [ -f "$SHELL_TEST" ]; then
        run_step "Shell extension tests (no daemon)" \
            "/logs/test-shell-nodaemon.log" \
            run_as_testuser "gjs -m '$SHELL_TEST' --no-daemon"
    fi
else
    step_info "mock-dbus-daemon.py not found, skipping D-Bus tests"
fi

# ============================================================================
# PHASE 7: Capture journal logs
# ============================================================================

echo ""
echo -e "${BOLD}=== Capturing logs ===${NC}"

run_as_testuser "journalctl --user -u lnxdrive --no-pager" > /logs/daemon-journal.log 2>&1 || true
run_as_testuser "journalctl --user --no-pager -n 200" > /logs/user-journal.log 2>&1 || true
step_info "Journal logs captured"

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "==================================="
echo -e "${BOLD}Results: ${PASS_COUNT} passed, ${FAIL_COUNT} failed (${TOTAL_STEPS} steps)${NC}"
echo "==================================="

# Write summary file
cat > /logs/summary.txt <<EOSUMMARY
LNXDrive Build & Test Summary
==============================
Date: $(date -Iseconds)
Passed: ${PASS_COUNT}
Failed: ${FAIL_COUNT}
Total:  ${TOTAL_STEPS}

Log files:
$(ls -1 /logs/*.log 2>/dev/null | sed 's/^/  /')
EOSUMMARY

if [ "$FAIL_COUNT" -eq 0 ]; then
    echo -e "${GREEN}All steps passed.${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}${FAIL_COUNT} step(s) failed. Check logs in /logs/ for details.${NC}"
    echo ""
    exit 1
fi
