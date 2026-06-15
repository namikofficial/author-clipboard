#!/usr/bin/env bash
# Smoke test for the unified GTK4 UI.
#
# Launches under Xvfb, sends keypresses via xdotool, saves screenshots
# to docs/UI/snapshots/.
#
# Requires: xvfb-run, xdotool, import (ImageMagick).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# Ensure the binary is built.
cargo build -p author-clipboard-applet 2>/dev/null

mkdir -p docs/UI/snapshots

manager_screenshot() {
    local name="$1"
    import -window root "docs/UI/snapshots/${name}.png" || true
    echo "Saved docs/UI/snapshots/${name}.png"
}

launch_manager() {
    xvfb-run -a -s "-screen 0 1280x800x24" \
        target/debug/author-clipboard --manager &
    MANAGER_PID=$!
    sleep 2
}

launch_popup() {
    target/debug/author-clipboard --popup &
    POPUP_PID=$!
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
manager_screenshot "manager"

# navigate to Settings via sidebar
xdotool mousemove 100 200 click 1  # click sidebar row ~3 (Settings)
sleep 1
manager_screenshot "settings"

# Navigate back to Clipboard
xdotool mousemove 100 80 click 1
sleep 1
manager_screenshot "clipboard-page"

kill ${MANAGER_PID:-} || true
wait 2>/dev/null

# ── Scenario 2: Popup with search ───────────────────────────────
echo "=== Scenario 2: Popup with search ==="
launch_popup

# Type / to focus search, then type "git"
xdotool key slash
sleep 0.3
xdotool type "git"
sleep 1
manager_screenshot "popup-search"

# Esc to clear search
xdotool key Escape
sleep 0.3
manager_screenshot "popup-esc"

# Close the popup
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
manager_screenshot "sensitive-reveal"

# Wait for countdown to expire (5s)
sleep 5
manager_screenshot "sensitive-expired"

kill ${MANAGER_PID:-} || true
wait 2>/dev/null

echo "=== All smoke tests complete ==="
