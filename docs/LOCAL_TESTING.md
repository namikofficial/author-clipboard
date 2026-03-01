# Local Testing Guide

Step-by-step guide for testing **author-clipboard** on your local COSMIC desktop.

## Prerequisites

- COSMIC desktop (or any Wayland compositor with `wlr-data-control-unstable-v1`)
- Rust toolchain installed (`rustup`)
- System dependencies installed (`just install-deps`)

## 1. Run the Test Suite

```bash
# All tests (unit tests across all crates)
just test

# Individual crate tests
cargo test -p author-clipboard-shared       # DB, types, config tests
cargo test -p author-clipboard-daemon       # Daemon tests (if any)
cargo test -p author-clipboard-applet       # Applet tests (if any)

# With output visible
cargo test --all -- --nocapture
```

## 2. Test the Clipboard Daemon

### Start the daemon

```bash
# Normal mode (info-level logging)
just daemon

# Debug mode (verbose — see every Wayland event)
RUST_LOG=debug just daemon
```

### What to expect

1. Daemon connects to Wayland display
2. Binds `wlr-data-control-manager` and `wl_seat`
3. Opens (or creates) the SQLite database at `~/.local/share/author-clipboard/clipboard.db`
4. Starts monitoring clipboard

### Test clipboard capture

With the daemon running in one terminal:

```bash
# Copy some text (another terminal)
echo "Hello from test" | wl-copy

# Copy something else
echo "Second copy" | wl-copy

# Copy a longer string
cat /etc/hostname | wl-copy
```

You should see log output like:

```
INFO  📋 Stored: Hello from test
INFO  📋 Stored: Second copy
INFO  📋 Stored: <hostname>
```

### Verify database contents

```bash
# Open the database directly
sqlite3 ~/.local/share/author-clipboard/clipboard.db

# List recent items
SELECT id, content, timestamp FROM clipboard_items ORDER BY timestamp DESC LIMIT 10;

# Check item count
SELECT COUNT(*) FROM clipboard_items;

# Check for duplicates (should have none — dedup is built in)
SELECT content_hash, COUNT(*) as cnt
FROM clipboard_items
GROUP BY content_hash
HAVING cnt > 1;
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

## 3. Test the Applet (Phase 1+)

```bash
just applet
```

> **Note:** The applet is a placeholder in Phase 0/1. Full UI testing comes in later phases.

## 4. COSMIC-Specific Setup

### Enable data control protocol

COSMIC requires an environment variable to enable the clipboard protocol:

```bash
export COSMIC_DATA_CONTROL_ENABLED=1
```

Add this to your `~/.profile` or `~/.bash_profile` for persistence.

### Verify protocol support

```bash
# Check if the protocol is available
wayland-info | grep -i data.control
```

If the protocol isn't listed, ensure `COSMIC_DATA_CONTROL_ENABLED=1` is set and your COSMIC session was restarted.

## 5. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| "Failed to connect to Wayland display" | Not running on Wayland | Run from a Wayland session, not X11 or SSH |
| "Compositor does not support wlr-data-control" | Protocol not enabled | Set `COSMIC_DATA_CONTROL_ENABLED=1` |
| "No seat found" | Unusual Wayland setup | Check `wayland-info` for `wl_seat` |
| No clipboard events | Using X11 apps under XWayland | XWayland clipboard should still be bridged, but some compositors don't bridge to `wlr-data-control` |
| Database errors | Permissions issue | Check `~/.local/share/author-clipboard/` permissions |

## 6. Full Verification

Run the complete quality check before committing:

```bash
just verify   # fmt → clippy → test → build
```
