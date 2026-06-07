# Glossary

> Terminology and definitions for author-clipboard.

---

## Core Concepts

### Clipboard Item
A single entry in the clipboard history. Contains: content type, content data (possibly encrypted), timestamp, source application, pin status, and optional TTL.

### Content Type
The MIME type category of a clipboard item:
- **Text**: Plain text (`text/plain`)
- **HTML**: Rich text with markup (`text/html`)
- **Image**: Binary image data (`image/png`, `image/jpeg`, etc.)
- **Files**: File URI list (`text/uri-list`)

### Picker
The UI for selecting a clipboard item to restore. Three variants:
- **COSMIC applet**: libcosmic-based popup for COSMIC desktop
- **External menu**: CLI picker piped to `wofi`, `fuzzel`, or `rofi`
- **Native picker**: GTK4 layer-shell popup for Hyprland (`hypr-picker`)

### Quick Paste
Using `wtype` (preferred) or `ydotool` (optional) to type selected text directly into the active application, rather than just copying to clipboard.

### Incognito Mode
When `<data_dir>/.incognito` exists, the daemon pauses clipboard capture. Create or remove the file to toggle.

### Screen Lock Detection
Daemon monitors for screen lock events via `loginctl` or D-Bus `org.freedesktop.ScreenSaver` and clears sensitive items if `clear_on_lock` is enabled.

---

## Security Terms

### Sensitive Content
Clipboard content that may contain credentials or secrets. Detected patterns include:
- Password field values
- API keys and tokens (Bearer, `sk-`, `pk-`, `ghp_`, etc.)
- SSH private keys (`-----BEGIN * PRIVATE KEY-----`)
- AWS credentials (`AKIA` prefix)
- JWT tokens (`eyJ` base64 prefix)
- URI credentials (`://user:pass@host`)
- High-entropy strings that appear to be secrets

### Encryption at Rest
AES-256-GCM encryption for sensitive items when `encrypt_sensitive: true`. Key stored in `<data_dir>/.encryption_key` with mode 0600.

### Content Hash
SHA-256 hash of content used for deduplication. Prevents storing duplicate items within the `dedup_window_seconds` window.

---

## Wayland Terms

### wlr-data-control
Wayland protocol for clipboard selection access in wlroots compositors (Hyprland, Sway). Exposed by the compositor when the application requests it.

### COSMIC_DATA_CONTROL_ENABLED
Environment variable that enables the `zwlr_data_control_manager_v1` protocol on COSMIC desktop. Required for clipboard monitoring.

### Layer Shell
Wayland protocol for positioning windows above other windows. Used by both the COSMIC applet and the `hypr-picker`.

### wl-seat
Wayland protocol for input devices (keyboard, pointer). Used to read and write clipboard selections.

---

## Database Terms

### FTS5
Full-Text Search version 5. SQLite virtual table for fast text search with LIKE fallback.

### WAL Mode
Write-Ahead Logging. SQLite journal mode that allows concurrent reads during writes and provides crash safety.

### TTL
Time-To-Live. Per-item expiration timestamp. Unpinned items with expired TTL are cleaned up during the cleanup interval.

---

## Configuration Terms

### dedup_window_seconds
Time window for duplicate suppression. If identical content is copied within this window, the existing item is updated (bumped) rather than creating a new entry.

### cleanup_interval_seconds
How often the daemon runs cleanup tasks (deleting expired items, enforcing max_items limit).

### max_item_size
Maximum size in bytes for a single clipboard item. Items larger than this are not stored.

### mime_denylist
List of MIME types or MIME prefixes to skip during capture. Default: `["application/x-kde-cutselection"]`.

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `toggle` | Show picker if hidden, hide if shown |
| `show` | Show picker at current cursor position |
| `hide` | Hide picker |
| `ping` | Check daemon is running via IPC |
| `history` | List recent clipboard items |
| `status` | Show database statistics |
| `clear` | Clear all unpinned items |
| `export` | Export history to JSON |
| `config` | Show current configuration |
| `picker` | Open external menu picker |
| `doctor` | Probe display/protocol support |
| `copy <id>` | Restore item by ID to clipboard |
| `hyprland-config` | Print recommended Hyprland keybinds |

---

## File Paths

| Path | Description |
|------|-------------|
| `~/.config/author-clipboard/config.json` | Configuration file |
| `<data_dir>/clipboard.db` | SQLite database |
| `<data_dir>/images/` | Stored image files |
| `<data_dir>/thumbnails/` | Generated thumbnails |
| `<data_dir>/.incognito` | Incognito mode flag |
| `<data_dir>/.encryption_key` | AES-256 key (mode 0600) |
| `$XDG_RUNTIME_DIR/author-clipboard` | IPC socket |

---

**Last Updated**: Phase 14 Complete (v0.5.0)