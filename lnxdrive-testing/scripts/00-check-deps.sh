#!/usr/bin/env bash
# 00-check-deps.sh - Verify host dependencies for LNXDrive testing
#
# Reports which testing levels are available based on installed tools.
#
# Exit codes:
#   0 - At least one testing level is available
#   1 - No testing levels available (critical deps missing)

set -euo pipefail

# --- Colors and formatting ---------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

ok() { echo -e "  ${GREEN}[OK]${NC}   $1"; }
miss() { echo -e "  ${RED}[MISS]${NC} $1"; }
opt() { echo -e "  ${YELLOW}[OPT]${NC}  $1"; }
info() { echo -e "  ${CYAN}[INFO]${NC} $1"; }

# --- Header -------------------------------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive Testing - Dependency Check${NC}"
echo "====================================="
echo ""

# --- Check categories ---------------------------------------------------------

CONTAINER_READY=true
HOST_BUILD_READY=true
GNOME_NESTED_READY=true
VM_READY=true

# ---- 1. Container testing (Podman) ----

echo -e "${BOLD}Container Testing (Podman)${NC}"

if command -v podman &>/dev/null; then
    ok "podman $(podman --version 2>/dev/null | head -1)"
else
    miss "podman - required for container-based testing"
    CONTAINER_READY=false
fi

if podman info --format '{{.Host.Security.SELinuxEnabled}}' &>/dev/null; then
    SELINUX=$(podman info --format '{{.Host.Security.SELinuxEnabled}}' 2>/dev/null || echo "unknown")
    info "SELinux: ${SELINUX} (use :z volume flag if enabled)"
fi

echo ""

# ---- 2. Host build tools ----

echo -e "${BOLD}Build Tools (Host)${NC}"

if command -v rustup &>/dev/null; then
    RUST_VER=$(rustc --version 2>/dev/null || echo "unknown")
    ok "rustup + ${RUST_VER}"
else
    miss "rustup/cargo - required for Rust compilation"
    HOST_BUILD_READY=false
fi

if command -v cargo &>/dev/null; then
    ok "cargo $(cargo --version 2>/dev/null | head -1)"
else
    miss "cargo - required for Rust compilation"
    HOST_BUILD_READY=false
fi

if command -v meson &>/dev/null; then
    ok "meson $(meson --version 2>/dev/null)"
else
    miss "meson - required for GNOME extension compilation"
    HOST_BUILD_READY=false
fi

if command -v ninja &>/dev/null; then
    ok "ninja $(ninja --version 2>/dev/null)"
else
    miss "ninja - required for GNOME extension compilation"
    HOST_BUILD_READY=false
fi

if command -v pkg-config &>/dev/null; then
    ok "pkg-config"
else
    miss "pkg-config - required for native library discovery"
    HOST_BUILD_READY=false
fi

echo ""

# ---- 3. Python + D-Bus testing ----

echo -e "${BOLD}Python / D-Bus Testing${NC}"

if command -v python3 &>/dev/null; then
    ok "python3 $(python3 --version 2>/dev/null)"
else
    miss "python3 - required for mock daemon and tests"
    CONTAINER_READY=false
fi

if python3 -c "import dbus_next" 2>/dev/null; then
    ok "python3-dbus-next"
else
    opt "python3-dbus-next - needed on host for direct D-Bus tests (pip install dbus-next)"
fi

if python3 -c "import gi; gi.require_version('Gio', '2.0')" 2>/dev/null; then
    ok "PyGObject (gi.repository.Gio)"
else
    opt "PyGObject - needed on host for Nautilus extension tests"
fi

echo ""

# ---- 4. GJS (Shell extension tests) ----

echo -e "${BOLD}GJS (Shell Extension)${NC}"

if command -v gjs &>/dev/null; then
    ok "gjs $(gjs --version 2>/dev/null)"
else
    opt "gjs - needed for Shell extension tests on host"
fi

echo ""

# ---- 5. GNOME nested session ----

echo -e "${BOLD}GNOME Nested Session${NC}"

if command -v gnome-shell &>/dev/null; then
    ok "gnome-shell $(gnome-shell --version 2>/dev/null)"
else
    miss "gnome-shell - required for nested session testing"
    GNOME_NESTED_READY=false
fi

if command -v dbus-run-session &>/dev/null; then
    ok "dbus-run-session"
else
    miss "dbus-run-session - required for isolated D-Bus session"
    GNOME_NESTED_READY=false
fi

if command -v nautilus &>/dev/null; then
    ok "nautilus $(nautilus --version 2>/dev/null)"
else
    opt "nautilus - needed for visual file manager testing"
fi

echo ""

# ---- 6. VM testing (QEMU/libvirt) ----

echo -e "${BOLD}VM Testing (QEMU/libvirt)${NC}"

if command -v qemu-system-x86_64 &>/dev/null; then
    ok "qemu-system-x86_64"
else
    miss "qemu-system-x86_64 - required for VM testing"
    VM_READY=false
fi

if command -v virt-install &>/dev/null; then
    ok "virt-install"
else
    miss "virt-install - required for VM creation"
    VM_READY=false
fi

if command -v virt-viewer &>/dev/null; then
    ok "virt-viewer"
else
    opt "virt-viewer - needed for VM graphical connection"
fi

if command -v cloud-localds &>/dev/null || command -v genisoimage &>/dev/null; then
    ok "cloud-init ISO tools (cloud-localds or genisoimage)"
else
    miss "cloud-localds/genisoimage - required for cloud-init ISO creation"
    VM_READY=false
fi

echo ""

# --- Summary ------------------------------------------------------------------

echo "====================================="
echo -e "${BOLD}Available Testing Levels${NC}"
echo "====================================="
echo ""

ANY_AVAILABLE=false

if $CONTAINER_READY; then
    echo -e "  ${GREEN}[AVAILABLE]${NC} Container testing (make build-test, make test-dbus)"
    echo -e "  ${GREEN}[AVAILABLE]${NC} GNOME Desktop container (make gnome-container)"
    ANY_AVAILABLE=true
else
    echo -e "  ${RED}[UNAVAIL]${NC}  Container testing — install podman"
fi

if $GNOME_NESTED_READY; then
    echo -e "  ${GREEN}[AVAILABLE]${NC} GNOME nested session (make gnome-nested)"
    ANY_AVAILABLE=true
else
    echo -e "  ${RED}[UNAVAIL]${NC}  GNOME nested session — install gnome-shell, dbus-run-session"
fi

if $VM_READY; then
    echo -e "  ${GREEN}[AVAILABLE]${NC} VM testing (make vm-create)"
    ANY_AVAILABLE=true
else
    echo -e "  ${RED}[UNAVAIL]${NC}  VM testing — install qemu, virt-install, cloud-localds"
fi

echo ""

if $ANY_AVAILABLE; then
    echo -e "${GREEN}At least one testing level is available.${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}No testing levels available. Install required dependencies.${NC}"
    echo ""
    echo "Quick install (Fedora):"
    echo "  sudo dnf install podman qemu-kvm virt-install virt-viewer cloud-utils-growpart"
    echo "  sudo dnf install gnome-shell nautilus dbus-tools gjs"
    echo "  sudo dnf install rust cargo meson ninja-build"
    echo "  pip install dbus-next"
    echo ""
    exit 1
fi
