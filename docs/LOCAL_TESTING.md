# Local Testing Guide

Step-by-step guide for testing **author-clipboard** on your local COSMIC desktop.

## Prerequisites

- COSMIC desktop (or any Wayland compositor with `wlr-data-control-unstable-v1`)
- Rust toolchain installed (`rustup`)
- System dependencies installed (`just install-deps`)
- `.env` file configured (copy from `.env.example`)

## Quick Start — End-to-End Test

The fastest way to test the full clipboard manager:

```bash
# 1. Make sure .env is set up
cp .env.example .env   # edit RUST_LOG=debug for verbose output

# 2. Build and run daemon + applet together
just run
```

This starts the **daemon** in the background (watches your clipboard) and opens the **applet** window. Now:

1. **Copy text** anywhere (Ctrl+C in any app, or `echo "test" | wl-copy`)
2. **See it appear** in the applet window within 2 seconds (auto-refreshes)
3. **Copy an image** (e.g., screenshot, or right-click → Copy Image in a browser)
4. **Image thumbnails** appear in the list
5. **Click any item** to copy it back to your clipboard
6. **Ctrl+V** in any app to paste

### Keyboard Shortcuts in the Applet

| Key | Action |
|-----|--------|
| ↑ / ↓ | Navigate items |
| Enter | Copy selected item |
| Esc | Clear search |
| Ctrl+F | Focus search bar |

### Tabs

The applet has 5 tabs:
- **📋 Clipboard** — Your clipboard history with search, pin, delete
- **😀 Emoji** — 500+ emoji organized by category with search
- **🔣 Symbols** — Math, currency, arrows, and technical symbols
- **顔 Kaomoji** — Japanese emoticons organized by mood
- **⚙️ Settings** — Privacy toggle, stats, and about info

### Incognito Mode

Click the 🕶️ button (or toggle in Settings tab) to pause clipboard recording. While active:
- New clipboard items are NOT stored
- The daemon skips all clipboard content
- Status bar shows "🕶️ Incognito"
- Toggle off to resume recording

### Sensitive Content Detection

The daemon automatically detects sensitive content:
- OTP/2FA codes (6-8 digit codes)
- API keys and tokens (OpenAI, GitHub, AWS, Stripe, etc.)
- JWT tokens
- Private keys
- Password-like strings

Sensitive items show a ⚠️ warning in the clipboard list.

> **Note:** Super+V global shortcut is not yet available — that requires COSMIC desktop runtime integration (Phase 2). For now, just launch the applet with `just run` or `just applet`.

## 1. Run the Test Suite

```bash
# All tests (unit tests across all crates)
just test

# Individual crate tests
cargo test -p author-clipboard-shared       # DB, types, config, image_store tests
cargo test -p author-clipboard-daemon       # Daemon tests (if any)
cargo test -p author-clipboard-applet       # Applet tests (if any)

# With output visible
cargo test --all -- --nocapture
```

## 2. Test the Clipboard Daemon (standalone)

### Start the daemon

```bash
# Normal mode (info-level logging)
just daemon

# Debug mode (edit .env: RUST_LOG=debug)
just daemon
```

### What to expect

1. Daemon connects to Wayland display
2. Binds `wlr-data-control-manager` and `wl_seat`
3. Opens (or creates) the SQLite database
4. Creates `images/` and `thumbnails/` directories for image support
5. Starts monitoring clipboard

### Test clipboard capture

With the daemon running in one terminal:

```bash
# Copy some text (another terminal)
echo "Hello from test" | wl-copy

# Copy something else
echo "Second copy" | wl-copy

# Copy an image
wl-copy --type image/png < some_image.png
```

You should see log output like:

```
INFO  📋 Stored: Hello from test
INFO  📋 Stored: Second copy
INFO  🖼️  Stored image: 00abcdef01234567.png (12345 bytes, image/png)
```

### Verify database contents

```bash
# Open the database directly
sqlite3 ~/.local/share/author-clipboard/clipboard.db

# List recent items
SELECT id, content, mime_type, content_type, timestamp FROM clipboard_items ORDER BY timestamp DESC LIMIT 10;

# Check item count
SELECT COUNT(*) FROM clipboard_items;

# Check images vs text
SELECT content_type, COUNT(*) FROM clipboard_items GROUP BY content_type;
```

### Test deduplication

```bash
# Copy the same text twice
echo "duplicate test" | wl-copy
sleep 1
echo "duplicate test" | wl-copy
```

The daemon should log the first copy as "Stored" and the second as a debug-level dedup bump (only visible with `RUST_LOG=debug`).

### Test size limit

The default max item size is 1 MB. Items larger than this are silently dropped:

```bash
# Generate a 2MB string
python3 -c "print('x' * 2_000_000)" | wl-copy
```

With `RUST_LOG=debug`, you should see "Ignoring oversized clipboard content".

## 3. Test the Applet (standalone)

```bash
just applet
```

The applet opens as a standalone window with:
- Search bar with real-time filtering
- Scrollable list of clipboard items (text + image thumbnails)
- Pin/delete/clear actions
- Keyboard navigation (↑↓ Enter Esc)
- Auto-refresh every 2 seconds (picks up new items from daemon)

## 4. COSMIC-Specific Setup

### Enable data control protocol

COSMIC requires an environment variable to enable the clipboard protocol:

```bash
export COSMIC_DATA_CONTROL_ENABLED=1
```

Add this to your `.env` file or `~/.profile` for persistence.

### Verify protocol support

```bash
# Check if the protocol is available
wayland-info | grep -i data.control
```

If the protocol isn't listed, ensure `COSMIC_DATA_CONTROL_ENABLED=1` is set and your COSMIC session was restarted.

## 5. Install for Daily Use

```bash
# Build release + install binaries, desktop file, icon, systemd service
just install

# Enable daemon autostart
just enable

# Check status
just status

# View live logs
just logs

# Uninstall
just uninstall
```

## 6. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| "Failed to connect to Wayland display" | Not running on Wayland | Run from a Wayland session, not X11 or SSH |
| "Compositor does not support wlr-data-control" | Protocol not enabled | Set `COSMIC_DATA_CONTROL_ENABLED=1` |
| "No seat found" | Unusual Wayland setup | Check `wayland-info` for `wl_seat` |
| No clipboard events | Using X11 apps under XWayland | XWayland clipboard should still be bridged, but some compositors don't bridge to `wlr-data-control` |
| Database errors | Permissions issue | Check `~/.local/share/author-clipboard/` permissions |
| Applet doesn't show new items | Daemon not running | Start daemon first: `just daemon` or `just run` |
| Images not captured | MIME type not supported | Check daemon logs with `RUST_LOG=debug` |

## 7. Full Verification

Run the complete quality check before committing:

```bash
just verify   # fmt → clippy → test → build
```
