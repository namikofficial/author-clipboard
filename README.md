# author-clipboard

> A fast, native clipboard manager for the COSMIC desktop — built entirely in Rust.

[![CI](https://github.com/namikofficial/author-clipboard/actions/workflows/ci.yml/badge.svg)](https://github.com/namikofficial/author-clipboard/actions)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/namikofficial/author-clipboard)](https://github.com/namikofficial/author-clipboard/releases)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**author-clipboard** is a privacy-focused clipboard manager for [COSMIC DE](https://system76.com/cosmic). It stores clipboard history in a local SQLite database with FTS5 full-text search, detects and encrypts sensitive content, and provides a native COSMIC popup UI with emoji/symbol/kaomoji pickers — all over Wayland.

---

## Why author-clipboard?

| | author-clipboard | Electron-based managers | GTK clipboard tools |
|---|---|---|---|
| **Desktop integration** | Native COSMIC (libcosmic) | Chromium runtime | GTK theming only |
| **Language** | Rust — memory-safe, no GC | JavaScript | C / Vala |
| **Search** | SQLite FTS5 — instant across thousands | In-memory filter | Simple substring |
| **Security** | Sensitive detection, AES-256-GCM, IPC hardening | Varies | Minimal |
| **Privacy** | Screen lock detection, per-item TTL, no cloud | Varies | Varies |
| **Footprint** | ~5 MB binary, minimal RAM | 100+ MB | 10–30 MB |

---

## Features

### Clipboard & Storage
- **Persistent history** with SQLite (WAL mode for crash safety)
- **FTS5 full-text search** — instant results across your entire history
- **Pin / unpin items** — keep important content from expiring
- **Per-item TTL** — auto-expire unpinned items (default: 7 days)
- **Dedup controls** — configurable window to skip duplicate copies
- **Export / import** — JSON backup and restore

### Security & Privacy
- **Sensitive content detection** — passwords, API keys, tokens, SSH keys, URI credentials
- **AES-256-GCM encryption** at rest for sensitive items (opt-in)
- **Screen lock detection** — optionally clear sensitive items on lock
- **Incognito mode** — temporarily pause recording
- **IPC hardening** — Unix socket in XDG runtime dir (never `/tmp`)

### UI & Navigation
- **COSMIC native popup** with light/dark theme support
- **COSMIC native icons** — symbolic icons for all actions and content types
- **Emoji picker, symbol picker, kaomoji picker**
- **Full keyboard navigation** — Home/End, PgUp/PgDn, Ctrl+1-9 quick select, Delete to remove
- **Quick paste** via `wtype` / `ydotool` backends
- **Daemon status indicator** — real-time capture status in the UI

### System Integration
- **CLI tool** (`author-clipboard-ctl`) for scripting and automation
- **IPC via Unix socket** — toggle, list, clear, export from scripts
- **Systemd user service** — start on login, restart on failure
- **JSON config file** at `~/.config/author-clipboard/config.json`
- **Global shortcut** — configurable (default: Super+V)

### Planned
- 🗓 Image / file clipboard support
- 🗓 Packaging (.deb, Arch AUR, Nix flake, Flatpak)

---

## Quick Start

### Prerequisites

- Linux with a Wayland compositor supporting `wlr-data-control` ([details](#wayland-requirements))
- Rust toolchain (1.75+)
- For COSMIC desktop: `COSMIC_DATA_CONTROL_ENABLED=1` ([how to enable](#enabling-on-cosmic-desktop))

### Build & Install

```bash
git clone https://github.com/namikofficial/author-clipboard
cd author-clipboard
just setup         # Install dev tools
just build         # Build all components
just install       # Install binaries, .desktop file, icon, systemd service
just enable        # Enable and start the daemon service
```

### CLI Tool

```bash
author-clipboard-ctl toggle          # Open or close applet
author-clipboard-ctl show            # Open applet
author-clipboard-ctl hide            # Close applet
author-clipboard-ctl ping            # Check daemon status
author-clipboard-ctl history         # List recent items
author-clipboard-ctl status          # Show database stats
author-clipboard-ctl clear           # Clear unpinned items
author-clipboard-ctl export out.json # Export history
author-clipboard-ctl config          # Show current config
```

### Keyboard Shortcut (Super+V)

Add a custom shortcut in **COSMIC Settings → Keyboard → Custom Shortcuts**:

1. Set command to `author-clipboard-ctl toggle`
2. Bind to **Super+V**

### Keyboard Navigation

| Key | Action |
|-----|--------|
| **↑ / ↓** | Navigate items |
| **Enter** | Copy selected item and close |
| **Escape** | Clear search (or close if empty) |
| **Home / End** | Jump to first / last item |
| **Page Up / Down** | Jump 10 items |
| **Delete** or **Ctrl+D** | Delete selected item |
| **Ctrl+1–9** | Quick copy by position |
| **Ctrl+Tab / Ctrl+Shift+Tab** | Next / previous tab |
| Type anything | Search is auto-focused |

---

## Configuration

Config file: `~/.config/author-clipboard/config.json`

| Key | Default | Description |
|-----|---------|-------------|
| `max_items` | `100` | Maximum clipboard items to retain |
| `max_item_size` | `1048576` | Max size per item in bytes (1 MB) |
| `ttl_seconds` | `604800` | Auto-expire unpinned items (7 days). `0` = never |
| `cleanup_interval_seconds` | `300` | How often cleanup runs (5 min) |
| `keyboard_shortcut` | `"Super+V"` | Display reference for configured shortcut |
| `encrypt_sensitive` | `false` | Encrypt sensitive items at rest (AES-256-GCM) |
| `clear_on_lock` | `true` | Clear sensitive items on screen lock |
| `dedup_window_seconds` | `2` | Skip duplicate copies within this window |

```json
{
  "max_items": 100,
  "max_item_size": 1048576,
  "ttl_seconds": 604800,
  "cleanup_interval_seconds": 300,
  "keyboard_shortcut": "Super+V",
  "encrypt_sensitive": false,
  "clear_on_lock": true,
  "dedup_window_seconds": 2
}
```

---

## Enabling on COSMIC Desktop

COSMIC requires `COSMIC_DATA_CONTROL_ENABLED=1` to allow clipboard managers to access the Wayland data control protocol. Choose one method:

**Session (temporary):**
```bash
export COSMIC_DATA_CONTROL_ENABLED=1
```

**Persist across logins:**
```bash
# Add to ~/.config/cosmic-comp/env (create if needed)
COSMIC_DATA_CONTROL_ENABLED=1
```

**System-wide (NixOS example):**
```nix
environment.sessionVariables.COSMIC_DATA_CONTROL_ENABLED = "1";
```

> **Security note:** This allows any Wayland application to read clipboard contents. Only enable if you trust all running applications.

Log out and back in after setting the variable.

---

## Wayland Requirements

Requires `wlr-data-control-unstable-v1` protocol support.

| Compositor | Status |
|-----------|--------|
| **COSMIC** | ✅ Supported (with `COSMIC_DATA_CONTROL_ENABLED=1`) |
| **Sway** | ✅ Supported |
| **Hyprland** | ✅ Supported |
| **wlroots-based** | ✅ Supported |
| GNOME / Mutter | ❌ Not supported |
| KDE / KWin | ❌ Not supported |

**Wayland only** — X11/XWayland clipboard events are not captured. The UI uses libcosmic and looks native only on COSMIC.

---

## Architecture

```
author-clipboard/
├── crates/clipboard-daemon/   # Wayland clipboard monitor (wlr-data-control)
├── crates/applet/             # COSMIC UI applet (popup history, search, emoji)
├── crates/ctl/                # CLI control tool (toggle, list, clear)
└── crates/shared/             # Database, config, encryption, IPC, types
```

| Component | Binary | Purpose |
|-----------|--------|---------|
| **clipboard-daemon** | `author-clipboard-daemon` | Monitors Wayland clipboard, stores history in SQLite |
| **applet** | `author-clipboard` | COSMIC UI with history, search, pins, pickers, export/import |
| **ctl** | `author-clipboard-ctl` | CLI tool for scripting and daemon control via IPC |
| **shared** | *(library)* | Database, types, config, encryption, IPC, sensitive detection |

---

## Development

```bash
just                # Show available commands
just verify        # Full check: format → lint → test → build
just build         # Build all crates
just check         # Quick type check (no full build)
just test          # Run all tests
just fmt           # Format code
just lint          # Clippy with -D warnings
just dev           # Watch mode (auto-rebuild on changes)
just daemon        # Run clipboard daemon
just applet        # Run GUI applet
```

See [PROJECT_PLAN.md](PROJECT_PLAN.md) for the development roadmap.

---

## Contributing

Contributions are welcome! Please read **[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)** before submitting a PR.

For security issues, see **[SECURITY.md](SECURITY.md)**.

```bash
just install-deps  # Install system dependencies
just setup         # Install Rust tools
just doctor        # Verify environment
just verify        # Run before committing
```

---

## Documentation

- **[PROJECT_PLAN.md](PROJECT_PLAN.md)** — Development phases and feature specifications
- **[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)** — How to contribute
- **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** — Development tooling and workflow
- **[docs/LOCAL_TESTING.md](docs/LOCAL_TESTING.md)** — Step-by-step local testing guide
- **[SECURITY.md](SECURITY.md)** — Security policy and reporting

---

## Related Projects

- [cosmic-utils/clipboard-manager](https://github.com/cosmic-utils/clipboard-manager) — Community COSMIC clipboard manager
- [pop-os/cosmic-applets](https://github.com/pop-os/cosmic-applets) — Official COSMIC applet examples

## License

[GPL-3.0](LICENSE)

---

<div align="center">
<strong>Built for the COSMIC desktop ecosystem</strong>
</div>
