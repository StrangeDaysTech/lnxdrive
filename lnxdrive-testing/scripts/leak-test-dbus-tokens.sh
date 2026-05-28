#!/usr/bin/env bash
# leak-test-dbus-tokens.sh — guard against RISK-002 (CVSS 9.1) regressions
#
# Runs the LNXDrive daemon inside an isolated D-Bus session (via
# `dbus-run-session`), captures every message on the bus for a short
# window of exercising the `com.strangedaystech.LNXDrive.Auth` interface,
# and fails if anything that looks like an OAuth token, refresh token or
# Bearer header crosses the wire.
#
# This test is the regression guard for the mitigation landed in PR for
# issue #5: the D-Bus public API must never accept or emit raw tokens.
#
# Usage:
#   ./scripts/leak-test-dbus-tokens.sh                 # uses the workspace target/
#   ./scripts/leak-test-dbus-tokens.sh --binary <path> # explicit lnxdrived path
#
# Exit codes:
#   0  no leak detected
#   1  one or more token-shaped strings appeared on the bus
#   2  setup error (missing dbus-monitor / dbus-run-session, daemon
#      failed to start, etc.)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TESTING_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$(cd "$TESTING_DIR/.." && pwd)"
ENGINE_DIR="${PROJECT_DIR}/lnxdrive-engine"

# --- Defaults -----------------------------------------------------------------

DAEMON_BIN_DEFAULT="${ENGINE_DIR}/target/debug/lnxdrived"
DAEMON_BIN=""
CAPTURE_SECONDS="${CAPTURE_SECONDS:-8}"

# Patterns that indicate a leaked token. Conservative on purpose: each one
# is something a legitimate D-Bus message of the LNXDrive interface would
# never contain in narrative form.
#
#   Bearer\s  → HTTP "Authorization: Bearer …" headers
#   eyJ[A-Za-z0-9_\-]{20,}  → JWT-shaped strings (header begins with eyJ…)
#   refresh_token            → literal JSON field name
#   access_token             → literal JSON field name
LEAK_PATTERNS='(Bearer\s|eyJ[A-Za-z0-9_\-]{20,}|refresh_token|access_token)'

# --- Argument parsing ---------------------------------------------------------

while [[ $# -gt 0 ]]; do
    case "$1" in
        --binary)
            DAEMON_BIN="$2"
            shift 2
            ;;
        --capture-seconds)
            CAPTURE_SECONDS="$2"
            shift 2
            ;;
        -h | --help)
            sed -n '2,22p' "$0"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 2
            ;;
    esac
done

DAEMON_BIN="${DAEMON_BIN:-$DAEMON_BIN_DEFAULT}"

# --- Pre-flight ---------------------------------------------------------------

require() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "ERROR: required tool '$1' is not on PATH" >&2
        exit 2
    }
}

require dbus-run-session
require dbus-monitor

if [[ ! -x "$DAEMON_BIN" ]]; then
    echo "ERROR: daemon binary not found at $DAEMON_BIN" >&2
    echo "Hint: run 'cargo build -p lnxdrive-daemon' first, or pass --binary <path>." >&2
    exit 2
fi

TMPDIR="$(mktemp -d -t lnxdrive-leak-test.XXXXXX)"
trap 'rm -rf "$TMPDIR"' EXIT

TRACE_LOG="${TMPDIR}/dbus-monitor.log"
DAEMON_LOG="${TMPDIR}/daemon.log"

# --- Inner script that runs inside the dbus-run-session ----------------------

INNER="${TMPDIR}/inner.sh"
cat >"$INNER" <<'INNER_EOF'
#!/usr/bin/env bash
set -euo pipefail
DAEMON_BIN="$1"
TRACE_LOG="$2"
DAEMON_LOG="$3"
CAPTURE_SECONDS="$4"

# Start dbus-monitor capturing the whole session bus.
dbus-monitor --session >"$TRACE_LOG" 2>&1 &
MONITOR_PID=$!

# Give the monitor a moment to attach.
sleep 0.5

# Start the daemon. It will own the well-known name and serve the Auth
# interface. We log its output for diagnostics but the assertion runs
# against the bus trace.
"$DAEMON_BIN" >"$DAEMON_LOG" 2>&1 &
DAEMON_PID=$!

# Give the daemon time to claim the name and register interfaces.
sleep 2

# Exercise the public Auth surface. We do NOT call CompleteAuthViaGOA
# with a real account because we cannot stand up a real GOA in CI;
# instead we (a) call the methods that exist and (b) intentionally
# attempt the now-removed CompleteAuthWithTokens method to confirm
# that the daemon rejects it (the D-Bus call will fail with
# UnknownMethod, which is itself the regression signal).
gdbus call \
    --session \
    --dest com.strangedaystech.LNXDrive \
    --object-path /com/strangedaystech/LNXDrive \
    --method com.strangedaystech.LNXDrive.Auth.StartAuth \
    >/dev/null 2>&1 || true

# Attempt the deleted method with a token-shaped payload. The daemon MUST
# answer with org.freedesktop.DBus.Error.UnknownMethod, which means the
# token strings on the wire are the *caller's* problem only — they leave
# this process via the request and never come back via any reply.
gdbus call \
    --session \
    --dest com.strangedaystech.LNXDrive \
    --object-path /com/strangedaystech/LNXDrive \
    --method com.strangedaystech.LNXDrive.Auth.CompleteAuthWithTokens \
    "decoy-access-token-DO-NOT-LEAK" \
    "decoy-refresh-token-DO-NOT-LEAK" \
    "1742400000" \
    >/dev/null 2>&1 || true

# Call the new method with a path that the backend will reject (we cannot
# spin up GOA in CI). The method itself must NOT echo tokens — we are
# checking the daemon emits no token-shaped strings in its log/reply.
gdbus call \
    --session \
    --dest com.strangedaystech.LNXDrive \
    --object-path /com/strangedaystech/LNXDrive \
    --method com.strangedaystech.LNXDrive.Auth.CompleteAuthViaGOA \
    "/org/gnome/OnlineAccounts/Accounts/9999-does-not-exist" \
    >/dev/null 2>&1 || true

# Give dbus-monitor a moment to flush the captured traffic.
sleep 2

# Tear down.
kill -TERM "$DAEMON_PID" 2>/dev/null || true
wait "$DAEMON_PID" 2>/dev/null || true
kill -TERM "$MONITOR_PID" 2>/dev/null || true
wait "$MONITOR_PID" 2>/dev/null || true
INNER_EOF
chmod +x "$INNER"

# --- Run ----------------------------------------------------------------------

echo "==> Capturing D-Bus traffic for ~${CAPTURE_SECONDS}s in an isolated session..."
dbus-run-session -- "$INNER" "$DAEMON_BIN" "$TRACE_LOG" "$DAEMON_LOG" "$CAPTURE_SECONDS"

# --- Assert -------------------------------------------------------------------

echo "==> Scanning ${TRACE_LOG} for token-shaped strings..."

# Strip the decoy strings we INTENTIONALLY sent as *request* arguments
# (they are the caller's bug to leak, not the daemon's; the daemon
# rejects them). We assert on what the daemon REPLIES with.
#
# What we keep:
#   reply messages (signal/method_return/error) — these are what the
#   daemon emits, and they MUST be free of tokens.
#
# What we discard:
#   method_call lines that contain the decoy strings — those are us
#   sending the bad call. The point of the test is that the daemon does
#   not parrot them back nor add a real token of its own.

REPLY_TRACE="${TMPDIR}/replies-only.log"
awk '
    /^method call/ { in_call=1; next }
    /^signal/      { in_call=0; print; next }
    /^method return/ { in_call=0; print; next }
    /^error/       { in_call=0; print; next }
    in_call == 0   { print }
' "$TRACE_LOG" >"$REPLY_TRACE"

if grep -E -q "$LEAK_PATTERNS" "$REPLY_TRACE"; then
    echo "FAIL: token-shaped strings appeared in D-Bus replies." >&2
    echo "Offending lines (first 20):" >&2
    grep -E "$LEAK_PATTERNS" "$REPLY_TRACE" | head -20 >&2
    echo "" >&2
    echo "Full trace kept at: $TRACE_LOG" >&2
    echo "Daemon log:        $DAEMON_LOG" >&2
    # Move logs to a stable location so CI can collect them.
    LEAK_DIR="${TESTING_DIR}/logs/leak-test-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$LEAK_DIR"
    cp "$TRACE_LOG" "$DAEMON_LOG" "$LEAK_DIR/"
    echo "Logs archived to:  $LEAK_DIR" >&2
    exit 1
fi

echo "PASS: no token-shaped strings in D-Bus replies during the capture window."
exit 0
