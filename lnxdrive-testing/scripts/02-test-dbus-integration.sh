#!/usr/bin/env bash
# 02-test-dbus-integration.sh - D-Bus integration tests
#
# Focused D-Bus testing: runs the mock daemon and real daemon tests
# inside a Podman container with systemd.
#
# Usage:
#   ./scripts/02-test-dbus-integration.sh [--rebuild]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTING_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$(cd "$TESTING_DIR/.." && pwd)"

IMAGE_NAME="localhost/lnxdrive-build-test"
CONTAINER_NAME="lnxdrive-dbus-test"

LNXDRIVE_DIR="${PROJECT_DIR}/lnxdrive-engine"
GNOME_DIR="${PROJECT_DIR}/lnxdrive-gnome"

TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
LOG_DIR="${TESTING_DIR}/logs/dbus-${TIMESTAMP}"

# --- Colors -------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

# --- Parse arguments ----------------------------------------------------------

FORCE_REBUILD=false
for arg in "$@"; do
    case "$arg" in
        --rebuild) FORCE_REBUILD=true ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

# --- Verify image exists (or build) ------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive D-Bus Integration Tests${NC}"
echo "=================================="
echo ""

IMAGE_EXISTS=$(podman images -q "${IMAGE_NAME}" 2>/dev/null)

if [ -z "$IMAGE_EXISTS" ] || $FORCE_REBUILD; then
    echo -e "${YELLOW}Building container image (required for D-Bus tests)...${NC}"
    podman build \
        -f "${TESTING_DIR}/containers/Containerfile.build-test" \
        -t "${IMAGE_NAME}" \
        "${TESTING_DIR}"
    echo ""
fi

# --- Prepare ------------------------------------------------------------------

mkdir -p "${LOG_DIR}"
ln -sfn "dbus-${TIMESTAMP}" "${TESTING_DIR}/logs/dbus-latest"

podman stop "${CONTAINER_NAME}" 2>/dev/null || true
podman rm "${CONTAINER_NAME}" 2>/dev/null || true

# --- Volume options -----------------------------------------------------------

# SELinux: disable labeling instead of :z (NTFS and other non-xattr FS compat)
SECURITY_OPT=()
if command -v getenforce &>/dev/null && [ "$(getenforce 2>/dev/null)" != "Disabled" ]; then
    SECURITY_OPT=(--security-opt label=disable)
fi

# --- Write inline D-Bus test script ------------------------------------------

cat > "${LOG_DIR}/_dbus-test-runner.sh" <<'INNERSCRIPT'
#!/usr/bin/env bash
# D-Bus integration test runner (runs inside container as root)
# Uses su - testuser for D-Bus operations (same pattern as container-test-runner.sh)
set -uo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

PASS=0
FAIL=0
TOTAL=0

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

run_test() {
    local name="$1"; local logfile="$2"; shift 2
    TOTAL=$((TOTAL + 1))
    echo -e "${BOLD}Test ${TOTAL}: ${name}${NC}"
    if "$@" > "$logfile" 2>&1; then
        echo -e "  ${GREEN}[PASS]${NC} $name"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}[FAIL]${NC} $name (see $logfile)"
        tail -15 "$logfile" 2>/dev/null || true
        FAIL=$((FAIL + 1))
    fi
}

echo ""
echo -e "${BOLD}D-Bus Integration Tests${NC}"
echo "========================"
echo ""

# Ensure /logs is writable
chmod 777 /logs 2>/dev/null || true

# Wait for testuser's D-Bus session bus
_uid="${TESTUSER_UID:-1000}"
_dbus_socket="/run/user/${_uid}/bus"
_waited=0
echo -e "${YELLOW}[INFO]${NC} Waiting for D-Bus session bus at ${_dbus_socket}..."
while [ ! -S "$_dbus_socket" ] && [ $_waited -lt 30 ]; do
    sleep 1
    _waited=$((_waited + 1))
done
if [ -S "$_dbus_socket" ]; then
    echo -e "${YELLOW}[INFO]${NC} D-Bus session bus ready (${_waited}s)"
else
    echo -e "${RED}[WARN]${NC} D-Bus session bus not found after 30s"
fi

# Copy source for building
cp -a /src/lnxdrive-engine /build/lnxdrive-engine 2>/dev/null || true
cp -a /src/lnxdrive-gnome /build/lnxdrive-gnome 2>/dev/null || true
chown -R testuser:testuser /build

EXCLUDE_STUBS="--exclude lnxdrive-conflict --exclude lnxdrive-audit --exclude lnxdrive-telemetry"
MOCK_DAEMON="/src/lnxdrive-gnome/tests/mock-dbus-daemon.py"
NAUTILUS_TEST="/src/lnxdrive-gnome/tests/test-nautilus-extension.py"
SHELL_TEST="/src/lnxdrive-gnome/tests/test-shell-extension.js"

# ---- Phase A: Build daemon ----

echo -e "${YELLOW}[INFO]${NC} Building daemon for D-Bus tests..."
cargo build --manifest-path /build/lnxdrive-engine/Cargo.toml --workspace $EXCLUDE_STUBS \
    > /logs/cargo-build.log 2>&1 || true

DAEMON_BIN="/build/lnxdrive-engine/target/debug/lnxdrived"
CLI_BIN="/build/lnxdrive-engine/target/debug/lnxdrive"

if [ -f "$DAEMON_BIN" ]; then
    cp "$DAEMON_BIN" /usr/local/bin/lnxdrived
    chmod +x /usr/local/bin/lnxdrived
fi
if [ -f "$CLI_BIN" ]; then
    cp "$CLI_BIN" /usr/local/bin/lnxdrive
    chmod +x /usr/local/bin/lnxdrive
fi

# Set up daemon config and service
cp /src/lnxdrive-engine/config/default-config.yaml /home/testuser/.config/lnxdrive/config.yaml 2>/dev/null || true
chown testuser:testuser /home/testuser/.config/lnxdrive/config.yaml

cat > /home/testuser/.config/systemd/user/lnxdrive.service <<'SVCEOF'
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
SVCEOF

mkdir -p /home/testuser/.config/systemd/user/default.target.wants
ln -sf /home/testuser/.config/systemd/user/lnxdrive.service \
       /home/testuser/.config/systemd/user/default.target.wants/lnxdrive.service
chown -R testuser:testuser /home/testuser/.config/systemd

# ---- Phase B: Test with real daemon ----

echo ""
echo -e "${BOLD}=== Phase B: Real daemon D-Bus tests ===${NC}"

if [ -f /usr/local/bin/lnxdrived ]; then
    run_as_testuser "systemctl --user daemon-reload" 2>/dev/null || true
    run_as_testuser "systemctl --user start lnxdrive" 2>/dev/null || true
    sleep 5

    # Capture D-Bus names
    run_as_testuser "busctl --user list" > /logs/dbus-names-real.log 2>&1 || true

    if [ -f "$NAUTILUS_TEST" ]; then
        run_test "Nautilus tests (real daemon)" /logs/test-nautilus-real.log \
            run_as_testuser "python3 '$NAUTILUS_TEST'"
    fi

    if [ -f "$SHELL_TEST" ]; then
        run_test "Shell tests (real daemon)" /logs/test-shell-real.log \
            run_as_testuser "gjs -m '$SHELL_TEST'"
    fi

    # Capture journal
    run_as_testuser "journalctl --user -u lnxdrive --no-pager" > /logs/daemon-journal.log 2>&1 || true

    run_as_testuser "systemctl --user stop lnxdrive" 2>/dev/null || true
    sleep 2
fi

# ---- Phase C: Mock daemon D-Bus tests ----

echo ""
echo -e "${BOLD}=== Phase C: Mock daemon D-Bus tests ===${NC}"

# Nautilus test manages its own mock daemon (must run without a pre-started daemon)
if [ -f "$NAUTILUS_TEST" ] && [ -f "$MOCK_DAEMON" ]; then
    run_test "Nautilus tests (mock daemon)" /logs/test-nautilus-mock.log \
        run_as_testuser "python3 '$NAUTILUS_TEST'"
fi

# Start mock daemon for Shell extension tests
if [ -f "$MOCK_DAEMON" ]; then
    run_as_testuser "python3 '$MOCK_DAEMON' --authenticated --signal-interval 999 \
        --sync-root /home/testuser/OneDrive \
        > /tmp/mock-daemon.log 2>&1 &"
    sleep 3

    # Capture D-Bus names
    run_as_testuser "busctl --user list" > /logs/dbus-names-mock.log 2>&1 || true
    cp /tmp/mock-daemon.log /logs/mock-daemon.log 2>/dev/null || true

    if [ -f "$SHELL_TEST" ]; then
        run_test "Shell tests (mock daemon)" /logs/test-shell-mock.log \
            run_as_testuser "gjs -m '$SHELL_TEST'"
    fi

    run_as_testuser "pkill -f mock-dbus-daemon" 2>/dev/null || true
    sleep 1
    cp /tmp/mock-daemon.log /logs/mock-daemon.log 2>/dev/null || true
fi

# ---- Phase D: Test without daemon ----

echo ""
echo -e "${BOLD}=== Phase D: No-daemon graceful handling ===${NC}"

if [ -f "$SHELL_TEST" ]; then
    run_test "Shell tests (no daemon)" /logs/test-shell-nodaemon.log \
        run_as_testuser "gjs -m '$SHELL_TEST' --no-daemon"
fi

# ---- Summary ----

echo ""
echo "========================"
echo -e "${BOLD}D-Bus Tests: ${PASS} passed, ${FAIL} failed (${TOTAL} total)${NC}"
echo "========================"

cat > /logs/summary.txt <<EOSUMMARY
D-Bus Integration Test Summary
===============================
Date: $(date -Iseconds)
Passed: ${PASS}
Failed: ${FAIL}
Total:  ${TOTAL}

Log files:
$(ls -1 /logs/*.log 2>/dev/null | sed 's/^/  /')
EOSUMMARY

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}All D-Bus tests passed.${NC}"
    exit 0
else
    echo -e "${RED}${FAIL} test(s) failed.${NC}"
    exit 1
fi
INNERSCRIPT
chmod +x "${LOG_DIR}/_dbus-test-runner.sh"

# --- Launch container ---------------------------------------------------------

echo -e "${BOLD}Starting systemd container...${NC}"

podman run -d \
    --name "${CONTAINER_NAME}" \
    --systemd=always \
    "${SECURITY_OPT[@]}" \
    -v "${LNXDRIVE_DIR}:/src/lnxdrive-engine:ro" \
    -v "${GNOME_DIR}:/src/lnxdrive-gnome:ro" \
    -v "${LOG_DIR}:/logs" \
    "${IMAGE_NAME}"

echo "Waiting for systemd boot..."
sleep 5

TESTUSER_UID=$(podman exec "${CONTAINER_NAME}" id -u testuser)

EXIT_CODE=0
podman exec \
    --env "TESTUSER_UID=${TESTUSER_UID}" \
    --env "PATH=/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
    "${CONTAINER_NAME}" \
    bash /logs/_dbus-test-runner.sh || EXIT_CODE=$?

# --- Cleanup ------------------------------------------------------------------

podman stop "${CONTAINER_NAME}" 2>/dev/null || true
podman rm "${CONTAINER_NAME}" 2>/dev/null || true

echo ""
if [ -f "${LOG_DIR}/summary.txt" ]; then
    cat "${LOG_DIR}/summary.txt"
fi
echo ""
echo "Logs: ${LOG_DIR}/"
echo ""

exit $EXIT_CODE
