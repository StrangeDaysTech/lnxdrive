#!/usr/bin/env bash
# collect-logs.sh - Collect and unify logs from all testing sources
#
# Scans the logs/ directory for all test runs and generates a unified summary.
#
# Usage:
#   ./scripts/collect-logs.sh          # Show summary
#   ./scripts/collect-logs.sh --latest  # Show only latest run per category
#   ./scripts/collect-logs.sh --detail  # Show detailed log contents

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTING_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_BASE="${TESTING_DIR}/logs"

# --- Colors -------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# --- Parse arguments ----------------------------------------------------------

SHOW_LATEST=false
SHOW_DETAIL=false

for arg in "$@"; do
    case "$arg" in
        --latest) SHOW_LATEST=true ;;
        --detail) SHOW_DETAIL=true ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

# --- Header -------------------------------------------------------------------

echo ""
echo -e "${BOLD}LNXDrive Testing - Log Summary${NC}"
echo "================================"
echo ""
echo "Log directory: ${LOG_BASE}/"
echo ""

# --- Scan log directories -----------------------------------------------------

if [ ! -d "$LOG_BASE" ]; then
    echo -e "${YELLOW}No logs directory found.${NC}"
    exit 0
fi

# Count categories
BUILD_RUNS=0
DBUS_RUNS=0
GNOME_NESTED_RUNS=0
GNOME_DESKTOP_RUNS=0

for dir in "${LOG_BASE}"/*/; do
    [ -d "$dir" ] || continue
    dirname="$(basename "$dir")"
    case "$dirname" in
        build-*) BUILD_RUNS=$((BUILD_RUNS + 1)) ;;
        dbus-*) DBUS_RUNS=$((DBUS_RUNS + 1)) ;;
        gnome-nested-*) GNOME_NESTED_RUNS=$((GNOME_NESTED_RUNS + 1)) ;;
        gnome-desktop-*) GNOME_DESKTOP_RUNS=$((GNOME_DESKTOP_RUNS + 1)) ;;
    esac
done

echo -e "${BOLD}Test Runs Found:${NC}"
echo "  Build & Test:      ${BUILD_RUNS}"
echo "  D-Bus Integration: ${DBUS_RUNS}"
echo "  GNOME Nested:      ${GNOME_NESTED_RUNS}"
echo "  GNOME Desktop:     ${GNOME_DESKTOP_RUNS}"
echo ""

# --- Show summaries -----------------------------------------------------------

show_run_summary() {
    local prefix="$1"
    local label="$2"

    echo -e "${BOLD}--- ${label} ---${NC}"

    local dirs=()
    for dir in "${LOG_BASE}"/${prefix}-*/; do
        [ -d "$dir" ] || continue
        dirs+=("$dir")
    done

    if [ ${#dirs[@]} -eq 0 ]; then
        echo -e "  ${YELLOW}No runs found.${NC}"
        echo ""
        return
    fi

    # If --latest, only show last one
    if $SHOW_LATEST; then
        dirs=("${dirs[-1]}")
    fi

    for dir in "${dirs[@]}"; do
        dirname="$(basename "$dir")"
        echo -e "  ${CYAN}${dirname}/${NC}"

        # Show summary.txt if present
        if [ -f "${dir}/summary.txt" ]; then
            while IFS= read -r line; do
                echo "    $line"
            done < "${dir}/summary.txt"
        else
            echo "    (no summary.txt)"
        fi

        # Show log files
        local log_count=0
        for logfile in "${dir}"/*.log; do
            [ -f "$logfile" ] || continue
            log_count=$((log_count + 1))
            local logname="$(basename "$logfile")"
            local logsize="$(stat -c%s "$logfile" 2>/dev/null || echo "?")"

            # Check for errors in log
            local has_error=false
            if grep -qi "error\|fail\|panic" "$logfile" 2>/dev/null; then
                has_error=true
            fi

            if $has_error; then
                echo -e "    ${RED}${logname}${NC} (${logsize} bytes) - contains errors"
            else
                echo -e "    ${GREEN}${logname}${NC} (${logsize} bytes)"
            fi

            # If --detail, show last lines of error logs
            if $SHOW_DETAIL && $has_error; then
                echo "      --- last 10 lines ---"
                tail -10 "$logfile" 2>/dev/null | while IFS= read -r line; do
                    echo "      $line"
                done
                echo "      ---"
            fi
        done

        if [ "$log_count" -eq 0 ]; then
            echo "    (no log files)"
        fi

        echo ""
    done
}

show_run_summary "build" "Build & Test Runs"
show_run_summary "dbus" "D-Bus Integration Runs"
show_run_summary "gnome-nested" "GNOME Nested Session Runs"
show_run_summary "gnome-desktop" "GNOME Desktop Container Runs"

# --- Generate unified summary file --------------------------------------------

SUMMARY_FILE="${LOG_BASE}/summary-$(date +%Y%m%d-%H%M%S).txt"

{
    echo "LNXDrive Testing - Unified Log Summary"
    echo "======================================="
    echo "Generated: $(date -Iseconds)"
    echo ""
    echo "Test Runs:"
    echo "  Build & Test:      ${BUILD_RUNS}"
    echo "  D-Bus Integration: ${DBUS_RUNS}"
    echo "  GNOME Nested:      ${GNOME_NESTED_RUNS}"
    echo "  GNOME Desktop:     ${GNOME_DESKTOP_RUNS}"
    echo ""

    # Latest summary from each category
    for prefix in build dbus gnome-nested gnome-desktop; do
        latest_dir=""
        for dir in "${LOG_BASE}"/${prefix}-*/; do
            [ -d "$dir" ] && latest_dir="$dir"
        done

        if [ -n "$latest_dir" ] && [ -f "${latest_dir}/summary.txt" ]; then
            echo "--- Latest ${prefix} ($(basename "$latest_dir")) ---"
            cat "${latest_dir}/summary.txt"
            echo ""
        fi
    done
} > "$SUMMARY_FILE"

# Update latest symlink
ln -sfn "$(basename "$SUMMARY_FILE")" "${LOG_BASE}/summary-latest.txt"

echo -e "${GREEN}Summary written to: ${SUMMARY_FILE}${NC}"
echo ""
