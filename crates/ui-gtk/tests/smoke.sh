#!/usr/bin/env bash
# Smoke test for the unified GTK4 UI.
#
# Runs inside the Xvfb session provided by `just ui-smoke`, sends input via
# xdotool, and saves screenshots to docs/UI/snapshots/.
#
# Requires: xvfb-run, xdotool, ffmpeg.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

for command in xdotool ffmpeg; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "Missing required command: $command" >&2
        exit 1
    fi
done

if [[ -z "${DISPLAY:-}" ]]; then
    echo "DISPLAY is unset. Run this script through: just ui-smoke" >&2
    exit 1
fi

SMOKE_ROOT="$(mktemp -d)"
export XDG_CONFIG_HOME="$SMOKE_ROOT/config"
export XDG_DATA_HOME="$SMOKE_ROOT/data"
export XDG_CACHE_HOME="$SMOKE_ROOT/cache"
export XDG_RUNTIME_DIR="$SMOKE_ROOT/runtime"
export GSETTINGS_BACKEND=memory
export GSETTINGS_SCHEMA_DIR="$ROOT/crates/ui-gtk/data"
export NO_AT_BRIDGE=1
export GDK_BACKEND=x11
export GSK_RENDERER=cairo
export XDG_SESSION_TYPE=x11
unset WAYLAND_DISPLAY
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" "$XDG_RUNTIME_DIR"
mkdir -p "$XDG_DATA_HOME/author-clipboard"
chmod 700 "$XDG_RUNTIME_DIR"

# Ensure the binary is built.
cargo build -p author-clipboard-applet 2>/dev/null

mkdir -p docs/UI/snapshots

capture_screenshot() {
    local name="$1"
    xdotool mousemove 1279 799
    sleep 0.1
    ffmpeg \
        -hide_banner \
        -loglevel error \
        -f x11grab \
        -video_size 1280x800 \
        -i "$DISPLAY" \
        -frames:v 1 \
        -y "docs/UI/snapshots/${name}.png"
    echo "Saved docs/UI/snapshots/${name}.png"
}

wait_for_window() {
    local title="$1"
    local attempts=0
    while ! xdotool search --onlyvisible --name "$title" >/dev/null 2>&1; do
        attempts=$((attempts + 1))
        if (( attempts >= 40 )); then
            echo "Timed out waiting for window: $title" >&2
            exit 1
        fi
        sleep 0.1
    done
    xdotool search --onlyvisible --name "$title" | head -1
}

launch_manager() {
    target/debug/author-clipboard --mode manager &
    MANAGER_PID=$!
    MANAGER_WINDOW="$(wait_for_window "Clipboard Manager")"
    xdotool windowmove "$MANAGER_WINDOW" 90 40
    sleep 1
}

launch_popup() {
    target/debug/author-clipboard --mode popup &
    POPUP_PID=$!
    POPUP_WINDOW="$(wait_for_window "Clipboard")"
    xdotool windowmove "$POPUP_WINDOW" 280 120
    sleep 1
}

cleanup() {
    kill ${MANAGER_PID:-} 2>/dev/null || true
    kill ${POPUP_PID:-} 2>/dev/null || true
}
trap cleanup EXIT

# ── Scenario 1: Manager basic ───────────────────────────────────
echo "=== Scenario 1: Manager basic ==="
launch_manager
capture_screenshot "manager"

# navigate to Settings via sidebar
xdotool mousemove --window "$MANAGER_WINDOW" 90 255 click 1
sleep 1
capture_screenshot "settings"

# Navigate back to Clipboard
xdotool mousemove --window "$MANAGER_WINDOW" 90 65 click 1
sleep 1
capture_screenshot "clipboard-page"

kill ${MANAGER_PID:-} || true
wait 2>/dev/null

# ── Scenario 2: Popup with search ───────────────────────────────
echo "=== Scenario 2: Popup with search ==="
launch_popup
capture_screenshot "popup"

# Type / to focus search, then type "git"
xdotool key slash
sleep 0.3
xdotool type "git"
sleep 1
capture_screenshot "popup-search"

# Esc to clear search
xdotool key Escape
sleep 0.3

kill ${POPUP_PID:-} || true
wait 2>/dev/null

# ── Scenario 3: Sensitive reveal countdown ──────────────────────
echo "=== Scenario 3: Manager sensitive reveal ==="
launch_manager

# Ctrl+Shift+R to trigger sensitive reveal
xdotool key ctrl+shift+r
sleep 1
capture_screenshot "sensitive-reveal"

kill ${MANAGER_PID:-} || true
wait 2>/dev/null

echo "=== All smoke tests complete ==="
