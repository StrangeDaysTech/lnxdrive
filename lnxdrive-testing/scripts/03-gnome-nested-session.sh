#!/usr/bin/env bash
# 03-gnome-nested-session.sh - GNOME nested session for visual testing
#
# Creates an isolated GNOME Shell nested session with:
#   - LNXDrive Shell indicator extension
#   - Mock D-Bus daemon with simulated file statuses
#   - Fake sync-root with test files
#
# The user sees a GNOME Shell window where they can:
#   - View the LNXDrive indicator in the top panel
#   - Open Nautilus to see overlay icons (if extension loads)
#   - Open the Preferences app
#
# Usage:
#   ./scripts/03-gnome-nested-session.sh [--build]
#
# Options:
#   --build   Build extensions before launching (requires meson, cargo)
#
# Requirements:
#   - gnome-shell (for nested session)
#   - dbus-run-session
#   - python3 + dbus-next (for mock daemon)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTING_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$(cd "$TESTING_DIR/.." && pwd)"

GNOME_DIR="${PROJECT_DIR}/lnxdrive-gnome"
LNXDRIVE_DIR="${PROJECT_DIR}/lnxdrive"

TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
LOG_DIR="${TESTING_DIR}/logs/gnome-nested-${TIMESTAMP}"

# Isolated XDG base
TEST_HOME="/tmp/lnxdrive-test-gnome-${TIMESTAMP}"
SYNC_ROOT="${TEST_HOME}/OneDrive"

# --- Colors -------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# --- Parse arguments ----------------------------------------------------------

DO_BUILD=false
for arg in "$@"; do
    case "$arg" in
        --build) DO_BUILD=true ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

# --- Verify prerequisites -----------------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive GNOME Nested Session${NC}"
echo "==============================="
echo ""

for cmd in gnome-shell dbus-run-session python3; do
    if ! command -v "$cmd" &>/dev/null; then
        echo -e "${RED}ERROR: '$cmd' not found. Install it first.${NC}"
        exit 1
    fi
done

if ! python3 -c "import dbus_next" 2>/dev/null; then
    echo -e "${RED}ERROR: python3 dbus-next not installed (pip install dbus-next)${NC}"
    exit 1
fi

# Detect GNOME Shell version for nested mode flag
GNOME_SHELL_VERSION=$(gnome-shell --version | awk '{print int($3)}')
if [ "$GNOME_SHELL_VERSION" -ge 49 ]; then
    # GNOME 49+ removed --nested; use --devkit (requires mutter-devel package)
    NESTED_FLAGS="--devkit --wayland"
    if [ ! -f /usr/libexec/mutter-devkit ]; then
        echo -e "${RED}ERROR: GNOME 49+ requires mutter-devkit for nested sessions.${NC}"
        echo -e "${RED}       Install it with: sudo dnf install mutter-devel${NC}"
        exit 1
    fi
else
    NESTED_FLAGS="--nested --wayland"
fi
echo -e "${YELLOW}GNOME Shell ${GNOME_SHELL_VERSION} detected, using: gnome-shell ${NESTED_FLAGS}${NC}"

# --- Setup isolated environment -----------------------------------------------

echo -e "${YELLOW}Setting up isolated test environment...${NC}"

mkdir -p "${TEST_HOME}"
mkdir -p "${LOG_DIR}"
ln -sfn "gnome-nested-${TIMESTAMP}" "${TESTING_DIR}/logs/gnome-nested-latest"

# XDG directories (isolated from host)
export XDG_DATA_HOME="${TEST_HOME}/.local/share"
export XDG_CONFIG_HOME="${TEST_HOME}/.config"
export XDG_CACHE_HOME="${TEST_HOME}/.cache"
export XDG_STATE_HOME="${TEST_HOME}/.local/state"

mkdir -p "${XDG_DATA_HOME}/gnome-shell/extensions"
mkdir -p "${XDG_CONFIG_HOME}/lnxdrive"
mkdir -p "${XDG_CACHE_HOME}"
mkdir -p "${XDG_STATE_HOME}"

# --- Create fake sync-root with test files ------------------------------------

echo -e "${YELLOW}Creating test files in sync root...${NC}"

mkdir -p "${SYNC_ROOT}/photos/vacation"
mkdir -p "${SYNC_ROOT}/projects/src"
mkdir -p "${SYNC_ROOT}/shared"

# Files matching mock daemon's hardcoded statuses
echo "Synced PDF content" > "${SYNC_ROOT}/document.pdf"
echo "Syncing document" > "${SYNC_ROOT}/report.docx"
echo "Conflict spreadsheet" > "${SYNC_ROOT}/budget.xlsx"
echo "Synced notes" > "${SYNC_ROOT}/notes.txt"
echo "Pending presentation" > "${SYNC_ROOT}/presentation.pptx"
echo "Excluded archive" > "${SYNC_ROOT}/archive.zip"
echo "Cloud-only beach photo" > "${SYNC_ROOT}/photos/vacation/beach.jpg"
echo "Synced readme" > "${SYNC_ROOT}/projects/readme.md"
echo "Synced source" > "${SYNC_ROOT}/projects/src/main.rs"
echo "Error document" > "${SYNC_ROOT}/shared/team-notes.docx"

# --- Install Shell extension into isolated XDG --------------------------------

SHELL_EXT_SRC="${GNOME_DIR}/shell-extension/lnxdrive-indicator@enigmora.com"
SHELL_EXT_DEST="${XDG_DATA_HOME}/gnome-shell/extensions/lnxdrive-indicator@enigmora.com"

if [ -d "$SHELL_EXT_SRC" ]; then
    echo -e "${YELLOW}Installing Shell extension...${NC}"
    cp -r "$SHELL_EXT_SRC" "$SHELL_EXT_DEST"
    echo -e "${GREEN}Shell extension installed to ${SHELL_EXT_DEST}${NC}"
else
    echo -e "${YELLOW}Shell extension source not found at ${SHELL_EXT_SRC}, skipping${NC}"
fi

# --- Optionally build extensions ----------------------------------------------

if $DO_BUILD; then
    echo ""
    echo -e "${BOLD}Building extensions...${NC}"

    # Build GNOME extensions with meson
    DESTDIR="${TEST_HOME}/destdir"
    if [ -f "${GNOME_DIR}/meson.build" ]; then
        BUILD_DIR="${TEST_HOME}/gnome-builddir"
        meson setup "$BUILD_DIR" "$GNOME_DIR" --prefix=/usr \
            > "${LOG_DIR}/meson-setup.log" 2>&1 && \
        meson compile -C "$BUILD_DIR" \
            > "${LOG_DIR}/meson-compile.log" 2>&1 && \
        DESTDIR="$DESTDIR" meson install -C "$BUILD_DIR" \
            > "${LOG_DIR}/meson-install.log" 2>&1 && \
        echo -e "${GREEN}GNOME extensions built and installed.${NC}" || \
        echo -e "${YELLOW}GNOME extension build had issues (check logs).${NC}"
    fi

    # Build preferences app and install to local bin
    LOCAL_BIN="${TEST_HOME}/.local/bin"
    mkdir -p "$LOCAL_BIN"
    if [ -f "${GNOME_DIR}/preferences/Cargo.toml" ]; then
        cargo build --release --manifest-path "${GNOME_DIR}/preferences/Cargo.toml" \
            > "${LOG_DIR}/cargo-preferences.log" 2>&1 && \
        cp "${GNOME_DIR}/preferences/target/release/lnxdrive-preferences" "$LOCAL_BIN/" && \
        echo -e "${GREEN}Preferences app built and installed to ${LOCAL_BIN}/${NC}" || \
        echo -e "${YELLOW}Preferences build had issues (check logs).${NC}"
    fi
fi

# --- Copy default LNXDrive config ---------------------------------------------

if [ -f "${LNXDRIVE_DIR}/config/default-config.yaml" ]; then
    cp "${LNXDRIVE_DIR}/config/default-config.yaml" \
       "${XDG_CONFIG_HOME}/lnxdrive/config.yaml"
fi

# --- Create GSettings override to enable the extension -------------------------

SCHEMA_DIR="${XDG_DATA_HOME}/glib-2.0/schemas"
mkdir -p "$SCHEMA_DIR"

# Copy the org.gnome.shell schema if available
SHELL_SCHEMA="/usr/share/glib-2.0/schemas/org.gnome.shell.gschema.xml"
if [ -f "$SHELL_SCHEMA" ]; then
    cp "$SHELL_SCHEMA" "$SCHEMA_DIR/"
fi

# Copy the LNXDrive Preferences schema
PREFS_SCHEMA="${GNOME_DIR}/preferences/data/com.enigmora.LNXDrive.Preferences.gschema.xml"
if [ -f "$PREFS_SCHEMA" ]; then
    cp "$PREFS_SCHEMA" "$SCHEMA_DIR/"
fi

cat > "${SCHEMA_DIR}/99-lnxdrive-test.gschema.override" <<'EOF'
[org.gnome.shell]
enabled-extensions=['lnxdrive-indicator@enigmora.com']
disable-user-extensions=false
EOF

glib-compile-schemas "$SCHEMA_DIR" 2>/dev/null || true

# --- Cleanup function ---------------------------------------------------------

MOCK_PID=""

cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"

    # Stop mock daemon
    if [ -n "$MOCK_PID" ] && kill -0 "$MOCK_PID" 2>/dev/null; then
        kill "$MOCK_PID" 2>/dev/null || true
        wait "$MOCK_PID" 2>/dev/null || true
    fi

    # Remove test home
    rm -rf "${TEST_HOME}"

    echo -e "${GREEN}Cleanup complete.${NC}"
    echo "Logs: ${LOG_DIR}/"
}

trap cleanup EXIT

# --- Launch mock daemon -------------------------------------------------------

echo ""
echo -e "${BOLD}Starting mock D-Bus daemon...${NC}"

MOCK_DAEMON="${GNOME_DIR}/tests/mock-dbus-daemon.py"

if [ ! -f "$MOCK_DAEMON" ]; then
    echo -e "${RED}ERROR: Mock daemon not found at ${MOCK_DAEMON}${NC}"
    exit 1
fi

# The mock daemon will run inside the dbus-run-session below.
# We prepare the command but launch it inside the session.

# --- Launch GNOME nested session ----------------------------------------------

echo ""
echo -e "${BOLD}Launching GNOME nested session...${NC}"
echo ""
echo -e "${CYAN}  A GNOME Shell window will open.${NC}"
echo -e "${CYAN}  Look for the LNXDrive indicator in the top panel.${NC}"
echo -e "${CYAN}  Open Nautilus (Files) to browse: ${SYNC_ROOT}${NC}"
echo -e "${CYAN}  Close the window to end the session.${NC}"
echo ""

# Detect Nautilus extension dir installed by meson (via DESTDIR)
NAUTILUS_EXT_DIR=""
if [ -d "${TEST_HOME}/destdir" ]; then
    NAUTILUS_EXT_DIR="$(find "${TEST_HOME}/destdir" -name 'extensions-4' -type d 2>/dev/null | head -1)"
fi

# Run inside an isolated D-Bus session
dbus-run-session -- bash -c "
    # Start mock daemon inside this session
    python3 '${MOCK_DAEMON}' \
        --authenticated \
        --signal-interval 5 \
        --sync-root '${SYNC_ROOT}' \
        > '${LOG_DIR}/mock-daemon.log' 2>&1 &
    MOCK_PID=\$!

    # Give daemon time to register on bus
    sleep 2

    # Set GSETTINGS_SCHEMA_DIR for our override
    export GSETTINGS_SCHEMA_DIR='${SCHEMA_DIR}'

    # Add preferences binary to PATH
    export PATH='${TEST_HOME}/.local/bin':\$PATH

    # Point Nautilus to the locally installed extension
    if [ -n '${NAUTILUS_EXT_DIR}' ] && [ -d '${NAUTILUS_EXT_DIR}' ]; then
        export NAUTILUS_EXTENSION_DIR='${NAUTILUS_EXT_DIR}'
    fi

    # Launch nested GNOME Shell
    gnome-shell ${NESTED_FLAGS} \
        > '${LOG_DIR}/gnome-shell.log' 2>&1

    # When gnome-shell exits, clean up
    kill \$MOCK_PID 2>/dev/null || true
"

echo ""
echo -e "${GREEN}GNOME nested session ended.${NC}"
echo "Logs: ${LOG_DIR}/"
