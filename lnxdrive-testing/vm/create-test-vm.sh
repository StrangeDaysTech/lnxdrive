#!/usr/bin/env bash
# create-test-vm.sh - Create a QEMU/libvirt VM for LNXDrive testing
#
# Downloads a Fedora Cloud image, creates a VM with cloud-init provisioning,
# and provides a full GNOME desktop for end-to-end testing.
#
# Usage:
#   ./vm/create-test-vm.sh          # Create and start VM
#   ./vm/create-test-vm.sh --destroy # Remove VM and disk
#   ./vm/create-test-vm.sh --status  # Check VM status
#
# Connect: virt-viewer lnxdrive-test-gnome
#
# Requirements:
#   qemu-system-x86_64, virt-install, cloud-localds or genisoimage

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTING_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$(cd "$TESTING_DIR/.." && pwd)"

VM_NAME="lnxdrive-test-gnome"
VM_RAM=4096      # MB
VM_CPUS=2
VM_DISK_SIZE=20  # GB

# Fedora Cloud image
FEDORA_VERSION="43"
FEDORA_ARCH="x86_64"
FEDORA_IMAGE_URL="https://download.fedoraproject.org/pub/fedora/linux/releases/${FEDORA_VERSION}/Cloud/${FEDORA_ARCH}/images/Fedora-Cloud-Base-Generic-${FEDORA_VERSION}-1.6.${FEDORA_ARCH}.qcow2"
FEDORA_IMAGE_FILENAME="Fedora-Cloud-Base-Generic-${FEDORA_VERSION}-1.6.${FEDORA_ARCH}.qcow2"

CACHE_DIR="${TESTING_DIR}/vm"
BASE_IMAGE="${CACHE_DIR}/${FEDORA_IMAGE_FILENAME}"
VM_DISK="${CACHE_DIR}/${VM_NAME}.qcow2"
CLOUD_INIT_ISO="${CACHE_DIR}/${VM_NAME}-cloud-init.iso"

# --- Colors -------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# --- Parse arguments ----------------------------------------------------------

ACTION="create"
for arg in "$@"; do
    case "$arg" in
        --destroy) ACTION="destroy" ;;
        --status) ACTION="status" ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

# --- Destroy action -----------------------------------------------------------

if [ "$ACTION" = "destroy" ]; then
    echo -e "${BOLD}Destroying VM: ${VM_NAME}${NC}"
    virsh --connect qemu:///session destroy "${VM_NAME}" 2>/dev/null || true
    virsh --connect qemu:///session undefine "${VM_NAME}" --remove-all-storage 2>/dev/null || true
    rm -f "${VM_DISK}" "${CLOUD_INIT_ISO}"
    echo -e "${GREEN}VM destroyed.${NC}"
    exit 0
fi

# --- Status action ------------------------------------------------------------

if [ "$ACTION" = "status" ]; then
    if virsh --connect qemu:///session dominfo "${VM_NAME}" &>/dev/null; then
        STATE=$(virsh --connect qemu:///session domstate "${VM_NAME}" 2>/dev/null || echo "unknown")
        echo -e "VM ${VM_NAME}: ${GREEN}${STATE}${NC}"
        if [ "$STATE" = "running" ]; then
            echo "Connect: virt-viewer ${VM_NAME}"
        fi
    else
        echo -e "VM ${VM_NAME}: ${YELLOW}not defined${NC}"
    fi
    exit 0
fi

# --- Create action ------------------------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive Test VM (QEMU/libvirt)${NC}"
echo "================================="
echo ""

# Check prerequisites
for cmd in qemu-system-x86_64 virt-install virsh; do
    if ! command -v "$cmd" &>/dev/null; then
        echo -e "${RED}ERROR: '$cmd' not found. Install libvirt and QEMU.${NC}"
        echo "  sudo dnf install qemu-kvm libvirt virt-install virt-viewer"
        exit 1
    fi
done

# Check for cloud-localds or genisoimage
CLOUD_INIT_TOOL=""
if command -v cloud-localds &>/dev/null; then
    CLOUD_INIT_TOOL="cloud-localds"
elif command -v genisoimage &>/dev/null; then
    CLOUD_INIT_TOOL="genisoimage"
else
    echo -e "${RED}ERROR: Neither cloud-localds nor genisoimage found.${NC}"
    echo "  sudo dnf install cloud-utils-growpart   # for cloud-localds"
    echo "  sudo dnf install genisoimage             # alternative"
    exit 1
fi

# ---- Step 1: Download Fedora Cloud image ----

if [ -f "$BASE_IMAGE" ]; then
    echo -e "${GREEN}Using cached base image: ${BASE_IMAGE}${NC}"
else
    echo -e "${YELLOW}Downloading Fedora Cloud ${FEDORA_VERSION} image...${NC}"
    echo "  URL: ${FEDORA_IMAGE_URL}"
    echo "  This is ~400MB, may take a few minutes."
    curl -L -f --progress-bar -o "${BASE_IMAGE}" "${FEDORA_IMAGE_URL}"

    # Validate the download is a real qcow2 image
    if ! file "$BASE_IMAGE" | grep -q "QEMU QCOW"; then
        echo -e "${RED}ERROR: Downloaded file is not a valid qcow2 image.${NC}"
        echo "  The URL may have changed. Check: https://www.fedoraproject.org/cloud/download/"
        rm -f "$BASE_IMAGE"
        exit 1
    fi
    echo -e "${GREEN}Download complete.${NC}"
fi

# ---- Step 2: Create VM disk from base image ----

if [ -f "$VM_DISK" ]; then
    echo -e "${YELLOW}VM disk already exists, removing...${NC}"
    rm -f "$VM_DISK"
fi

echo -e "${YELLOW}Creating VM disk (${VM_DISK_SIZE}GB)...${NC}"
qemu-img create -f qcow2 -b "$BASE_IMAGE" -F qcow2 "$VM_DISK" "${VM_DISK_SIZE}G"

# ---- Step 3: Create cloud-init ISO ----

echo -e "${YELLOW}Creating cloud-init ISO...${NC}"

CLOUD_INIT_DIR="${SCRIPT_DIR}/cloud-init"
if [ ! -f "${CLOUD_INIT_DIR}/user-data" ]; then
    echo -e "${RED}ERROR: cloud-init/user-data not found${NC}"
    exit 1
fi

if [ "$CLOUD_INIT_TOOL" = "cloud-localds" ]; then
    cloud-localds "$CLOUD_INIT_ISO" \
        "${CLOUD_INIT_DIR}/user-data" \
        "${CLOUD_INIT_DIR}/meta-data"
else
    genisoimage -output "$CLOUD_INIT_ISO" \
        -volid cidata -joliet -rock \
        "${CLOUD_INIT_DIR}/user-data" \
        "${CLOUD_INIT_DIR}/meta-data"
fi

# ---- Step 4: Remove existing VM if defined ----

if virsh --connect qemu:///session dominfo "${VM_NAME}" &>/dev/null; then
    echo -e "${YELLOW}Removing existing VM definition...${NC}"
    virsh --connect qemu:///session destroy "${VM_NAME}" 2>/dev/null || true
    virsh --connect qemu:///session undefine "${VM_NAME}" 2>/dev/null || true
fi

# ---- Step 5: Create and start VM ----

echo -e "${BOLD}Creating VM: ${VM_NAME}${NC}"
echo "  RAM: ${VM_RAM}MB, CPUs: ${VM_CPUS}, Disk: ${VM_DISK_SIZE}GB"

VM_XML="${CACHE_DIR}/${VM_NAME}.xml"

# Generate domain XML (without creating the VM)
virt-install \
    --connect qemu:///session \
    --name "${VM_NAME}" \
    --ram "${VM_RAM}" \
    --vcpus "${VM_CPUS}" \
    --disk path="${VM_DISK}",format=qcow2 \
    --disk path="${CLOUD_INIT_ISO}",device=cdrom \
    --os-variant "fedora-unknown" \
    --graphics spice \
    --channel spicevmc \
    --filesystem source="${PROJECT_DIR}",target=lnxdrive,accessmode=mapped \
    --network user \
    --security type=none \
    --import \
    --print-xml > "$VM_XML"

# Add passt backend (required for portForward) and SSH port forwarding (host:2222 -> guest:22)
sed -i "/<interface type/a\\      <backend type='passt'/>" "$VM_XML"
sed -i "s|</interface>|<portForward proto='tcp'><range start='2222' to='22'/></portForward></interface>|" "$VM_XML"

# Define and start VM
virsh --connect qemu:///session define "$VM_XML"
virsh --connect qemu:///session start "$VM_NAME"
rm -f "$VM_XML"

echo ""
echo -e "${GREEN}====================================${NC}"
echo -e "${GREEN}VM '${VM_NAME}' is starting!${NC}"
echo -e "${GREEN}====================================${NC}"
echo ""
echo -e "${CYAN}Cloud-init will provision the VM (install GNOME, compile LNXDrive).${NC}"
echo -e "${CYAN}This takes ~10-15 minutes on first boot.${NC}"
echo ""
echo -e "Connect:  ${BOLD}virt-viewer ${VM_NAME}${NC}"
echo -e "SSH:      ${BOLD}ssh -p 2222 testuser@localhost${NC}  (user-mode networking)"
echo -e "Status:   ${BOLD}$0 --status${NC}"
echo -e "Destroy:  ${BOLD}$0 --destroy${NC}"
echo ""
echo -e "Default credentials:"
echo -e "  User: ${BOLD}testuser${NC}"
echo -e "  Password: ${BOLD}testuser${NC}"
echo ""
