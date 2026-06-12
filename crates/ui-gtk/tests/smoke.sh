#!/usr/bin/env bash
# Smoke test for the unified GTK4 UI.
#
# Launches `author-clipboard --manager` under Xvfb, sends a few
# keypresses, and saves a screenshot to docs/UI/snapshots/.
#
# Requires: xvfb-run, xdotool, import (ImageMagick).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# Ensure the binary is built.
cargo build -p author-clipboard-applet 2>/dev/null

mkdir -p docs/UI/snapshots

# Launch the manager under Xvfb.
LOG="$(mktemp)"
echo "Launching author-clipboard --manager under Xvfb…"
xvfb-run -a -s "-screen 0 1280x800x24" \
    target/debug/author-clipboard --manager &
PID=$!
trap 'kill $PID 2>/dev/null || true; rm -f "$LOG"' EXIT

# Wait for the window to appear.
sleep 2

# Take a screenshot.
import -window root docs/UI/snapshots/manager.png || true
echo "Saved docs/UI/snapshots/manager.png"

# Try the popup, too.
target/debug/author-clipboard --popup &
POPUP_PID=$!
sleep 1
import -window root docs/UI/snapshots/popup.png || true
kill $POPUP_PID 2>/dev/null || true
echo "Saved docs/UI/snapshots/popup.png"

echo "Smoke test complete."
