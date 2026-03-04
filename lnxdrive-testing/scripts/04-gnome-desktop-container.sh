#!/usr/bin/env bash
# 04-gnome-desktop-container.sh - GNOME desktop container with VNC
#
# Builds both repos, extracts binaries, then runs a container with a full
# GNOME desktop accessible via VNC viewer at localhost:5900.
#
# Usage:
#   ./scripts/04-gnome-desktop-container.sh          # Start
#   ./scripts/04-gnome-desktop-container.sh --stop    # Stop
#   ./scripts/04-gnome-desktop-container.sh --rebuild  # Force rebuild
#
# Connect: vncviewer localhost:5900 (password: testuser)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTING_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$(cd "$TESTING_DIR/.." && pwd)"

BUILD_IMAGE="localhost/lnxdrive-build-test"
DESKTOP_IMAGE="localhost/lnxdrive-gnome-desktop"
BUILD_CONTAINER="lnxdrive-build-extract"
DESKTOP_CONTAINER="lnxdrive-gnome-desktop"

LNXDRIVE_DIR="${PROJECT_DIR}/lnxdrive"
GNOME_DIR="${PROJECT_DIR}/lnxdrive-gnome"

TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
LOG_DIR="${TESTING_DIR}/logs/gnome-desktop-${TIMESTAMP}"
BUILD_OUTPUT="${TESTING_DIR}/build-output"

VNC_PORT=5900

# --- Colors -------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# --- Parse arguments ----------------------------------------------------------

ACTION="start"
FORCE_REBUILD=false

for arg in "$@"; do
    case "$arg" in
        --stop) ACTION="stop" ;;
        --rebuild) FORCE_REBUILD=true ;;
        --status) ACTION="status" ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

# --- Stop action --------------------------------------------------------------

if [ "$ACTION" = "stop" ]; then
    echo -e "${BOLD}Stopping GNOME desktop container...${NC}"
    podman stop "${DESKTOP_CONTAINER}" 2>/dev/null || true
    podman rm "${DESKTOP_CONTAINER}" 2>/dev/null || true
    echo -e "${GREEN}Container stopped.${NC}"
    exit 0
fi

# --- Status action ------------------------------------------------------------

if [ "$ACTION" = "status" ]; then
    if podman ps --format '{{.Names}}' 2>/dev/null | grep -q "^${DESKTOP_CONTAINER}$"; then
        echo -e "${GREEN}GNOME desktop container is running.${NC}"
        echo "Connect: vncviewer localhost:${VNC_PORT}"
    else
        echo -e "${YELLOW}GNOME desktop container is not running.${NC}"
    fi
    exit 0
fi

# --- Start action -------------------------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive GNOME Desktop Container (VNC)${NC}"
echo "========================================="
echo ""

# ---- Step 1: Build binaries using build container ----

echo -e "${BOLD}Step 1: Build binaries...${NC}"

BUILD_IMAGE_EXISTS=$(podman images -q "${BUILD_IMAGE}" 2>/dev/null)
if [ -z "$BUILD_IMAGE_EXISTS" ] || $FORCE_REBUILD; then
    echo -e "${YELLOW}Building build container image...${NC}"
    podman build \
        -f "${TESTING_DIR}/containers/Containerfile.build-test" \
        -t "${BUILD_IMAGE}" \
        "${TESTING_DIR}"
fi

mkdir -p "${BUILD_OUTPUT}"

# SELinux: disable labeling instead of :z (NTFS and other non-xattr FS compat)
SECURITY_OPT=()
if command -v getenforce &>/dev/null && [ "$(getenforce 2>/dev/null)" != "Disabled" ]; then
    SECURITY_OPT=(--security-opt label=disable)
fi

# Run build inside container
podman stop "${BUILD_CONTAINER}" 2>/dev/null || true
podman rm "${BUILD_CONTAINER}" 2>/dev/null || true

podman run -d \
    --name "${BUILD_CONTAINER}" \
    --systemd=always \
    "${SECURITY_OPT[@]}" \
    -v "${LNXDRIVE_DIR}:/src/lnxdrive:ro" \
    -v "${GNOME_DIR}:/src/lnxdrive-gnome:ro" \
    "${BUILD_IMAGE}"

sleep 5

TESTUSER_UID=$(podman exec "${BUILD_CONTAINER}" id -u testuser)

echo -e "${YELLOW}Compiling inside build container...${NC}"

podman exec \
    --env "PATH=/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin" \
    "${BUILD_CONTAINER}" \
    bash -c '
        EXCLUDE="--exclude lnxdrive-conflict --exclude lnxdrive-audit --exclude lnxdrive-telemetry"
        cp -a /src/lnxdrive /build/lnxdrive
        cp -a /src/lnxdrive-gnome /build/lnxdrive-gnome

        # Build daemon
        cargo build --manifest-path /build/lnxdrive/Cargo.toml --workspace $EXCLUDE 2>&1 | tail -5

        # Build preferences
        cargo build --manifest-path /build/lnxdrive-gnome/preferences/Cargo.toml 2>&1 | tail -5

        # Build GNOME extensions
        meson setup /build/gnome-builddir /build/lnxdrive-gnome --prefix=/usr 2>&1 | tail -5
        meson compile -C /build/gnome-builddir 2>&1 | tail -5
    ' || true

# ---- Step 2: Extract binaries from build container ----

echo -e "${BOLD}Step 2: Extract binaries...${NC}"

# Extract daemon binaries
podman cp "${BUILD_CONTAINER}:/build/lnxdrive/target/debug/lnxdrived" \
    "${BUILD_OUTPUT}/lnxdrived" 2>/dev/null || true
podman cp "${BUILD_CONTAINER}:/build/lnxdrive/target/debug/lnxdrive" \
    "${BUILD_OUTPUT}/lnxdrive" 2>/dev/null || true

# Extract preferences binary
podman cp "${BUILD_CONTAINER}:/build/lnxdrive-gnome/preferences/target/debug/lnxdrive-preferences" \
    "${BUILD_OUTPUT}/lnxdrive-preferences" 2>/dev/null || true

# Copy Shell extension source directly (GJS doesn't compile)
if [ -d "${GNOME_DIR}/shell-extension" ]; then
    mkdir -p "${BUILD_OUTPUT}/shell-extension"
    cp -r "${GNOME_DIR}/shell-extension/lnxdrive-indicator@enigmora.com" \
        "${BUILD_OUTPUT}/shell-extension/" 2>/dev/null || true
fi

# Copy mock daemon
cp "${GNOME_DIR}/tests/mock-dbus-daemon.py" "${BUILD_OUTPUT}/mock-dbus-daemon.py" 2>/dev/null || true

# Copy Nautilus extension if it was compiled
mkdir -p "${BUILD_OUTPUT}/nautilus-extension"
podman cp "${BUILD_CONTAINER}:/build/gnome-builddir/nautilus-extension/" \
    "${BUILD_OUTPUT}/nautilus-extension/" 2>/dev/null || true

# Stop build container
podman stop "${BUILD_CONTAINER}" 2>/dev/null || true
podman rm "${BUILD_CONTAINER}" 2>/dev/null || true

echo -e "${GREEN}Binaries extracted to ${BUILD_OUTPUT}/${NC}"

# ---- Step 3: Build desktop image ----

echo -e "${BOLD}Step 3: Build desktop container image...${NC}"

DESKTOP_IMAGE_EXISTS=$(podman images -q "${DESKTOP_IMAGE}" 2>/dev/null)
if [ -z "$DESKTOP_IMAGE_EXISTS" ] || $FORCE_REBUILD; then
    podman build \
        -f "${TESTING_DIR}/containers/Containerfile.gnome-desktop" \
        -t "${DESKTOP_IMAGE}" \
        "${TESTING_DIR}"
fi

# ---- Step 4: Launch desktop container ----

echo -e "${BOLD}Step 4: Launch GNOME desktop container...${NC}"

mkdir -p "${LOG_DIR}"
ln -sfn "gnome-desktop-${TIMESTAMP}" "${TESTING_DIR}/logs/gnome-desktop-latest"

podman stop "${DESKTOP_CONTAINER}" 2>/dev/null || true
podman rm "${DESKTOP_CONTAINER}" 2>/dev/null || true

podman run -d \
    --name "${DESKTOP_CONTAINER}" \
    --systemd=always \
    "${SECURITY_OPT[@]}" \
    -p "${VNC_PORT}:5900" \
    -v "${BUILD_OUTPUT}:/opt/lnxdrive:ro" \
    -v "${LOG_DIR}:/logs" \
    "${DESKTOP_IMAGE}"

sleep 5

# Start the desktop session inside the container
podman exec -d "${DESKTOP_CONTAINER}" /usr/local/bin/start-desktop.sh

echo ""
echo -e "${GREEN}====================================${NC}"
echo -e "${GREEN}GNOME Desktop container is running!${NC}"
echo -e "${GREEN}====================================${NC}"
echo ""
echo -e "${CYAN}Connect with: ${BOLD}vncviewer localhost:${VNC_PORT}${NC}"
echo -e "${CYAN}VNC password: ${BOLD}testuser${NC}"
echo ""
echo -e "What you'll see:"
echo -e "  - GNOME Shell desktop"
echo -e "  - LNXDrive indicator in top panel (if extension loaded)"
echo -e "  - Test files in ~/OneDrive with simulated sync statuses"
echo -e "  - Mock daemon running (no real OneDrive connection)"
echo ""
echo -e "To stop:  ${BOLD}$0 --stop${NC}"
echo -e "Logs:     ${LOG_DIR}/"
echo ""
