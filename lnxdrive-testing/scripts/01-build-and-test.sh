#!/usr/bin/env bash
# 01-build-and-test.sh - Build both repos + run all tests in a container
#
# Builds the Podman image if needed, mounts both repos and a logs volume,
# then runs the container-test-runner.sh inside a systemd container.
#
# Usage:
#   ./scripts/01-build-and-test.sh [--rebuild]
#
# Options:
#   --rebuild   Force rebuild of the container image

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTING_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$(cd "$TESTING_DIR/.." && pwd)"

IMAGE_NAME="localhost/lnxdrive-build-test"
CONTAINER_NAME="lnxdrive-build-test"

LNXDRIVE_DIR="${PROJECT_DIR}/lnxdrive-engine"
GNOME_DIR="${PROJECT_DIR}/lnxdrive-gnome"

TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
LOG_DIR="${TESTING_DIR}/logs/build-${TIMESTAMP}"

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

# --- Verify source repos exist ------------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive Build & Test (Container)${NC}"
echo "==================================="
echo ""

if [ ! -f "${LNXDRIVE_DIR}/Cargo.toml" ]; then
    echo -e "${RED}ERROR: lnxdrive repo not found at ${LNXDRIVE_DIR}${NC}"
    exit 1
fi

if [ ! -f "${GNOME_DIR}/meson.build" ]; then
    echo -e "${RED}ERROR: lnxdrive-gnome repo not found at ${GNOME_DIR}${NC}"
    exit 1
fi

# --- Build image if needed ----------------------------------------------------

IMAGE_EXISTS=$(podman images -q "${IMAGE_NAME}" 2>/dev/null)

if [ -z "$IMAGE_EXISTS" ] || $FORCE_REBUILD; then
    echo -e "${YELLOW}Building container image...${NC}"
    echo "  This may take several minutes on first run."
    echo ""
    podman build \
        -f "${TESTING_DIR}/containers/Containerfile.build-test" \
        -t "${IMAGE_NAME}" \
        "${TESTING_DIR}"
    echo ""
    echo -e "${GREEN}Image built successfully.${NC}"
else
    echo -e "${GREEN}Using existing image: ${IMAGE_NAME}${NC}"
fi

# --- Prepare log directory ----------------------------------------------------

mkdir -p "${LOG_DIR}"
echo -e "Logs: ${LOG_DIR}"

# Create symlink for latest logs
ln -sfn "build-${TIMESTAMP}" "${TESTING_DIR}/logs/build-latest"

# --- Clean up any existing container ------------------------------------------

podman stop "${CONTAINER_NAME}" 2>/dev/null || true
podman rm "${CONTAINER_NAME}" 2>/dev/null || true

# --- Run the container --------------------------------------------------------

echo ""
echo -e "${BOLD}Starting systemd container...${NC}"

# SELinux: use --security-opt label=disable instead of :z volume suffix.
# The source repos may reside on filesystems without xattr support (e.g. NTFS),
# where :z relabeling fails silently, causing empty mounts.
SECURITY_OPT=()
if command -v getenforce &>/dev/null && [ "$(getenforce 2>/dev/null)" != "Disabled" ]; then
    SECURITY_OPT=(--security-opt label=disable)
fi

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

# Get testuser UID for D-Bus env inside the runner
TESTUSER_UID=$(podman exec "${CONTAINER_NAME}" id -u testuser)

echo -e "${BOLD}Running tests...${NC}"
echo ""

# Execute the test runner as root. The script internally uses su - testuser
# for operations that require the systemd user session (D-Bus, systemctl).
EXIT_CODE=0
podman exec \
    --env "TESTUSER_UID=${TESTUSER_UID}" \
    --env "PATH=/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
    "${CONTAINER_NAME}" \
    /usr/local/bin/container-test-runner.sh || EXIT_CODE=$?

# --- Cleanup ------------------------------------------------------------------

echo ""
echo "Stopping container..."
podman stop "${CONTAINER_NAME}" 2>/dev/null || true
podman rm "${CONTAINER_NAME}" 2>/dev/null || true

# --- Summary ------------------------------------------------------------------

echo ""
if [ -f "${LOG_DIR}/summary.txt" ]; then
    echo -e "${BOLD}Test Summary:${NC}"
    cat "${LOG_DIR}/summary.txt"
fi

echo ""
echo "Full logs: ${LOG_DIR}/"
echo ""

exit $EXIT_CODE
