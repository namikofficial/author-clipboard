# author-clipboard

> A fast, native clipboard manager for COSMIC desktop, built entirely in Rust

**author-clipboard** delivers a powerful clipboard experience designed for COSMIC DE. Store clipboard history, search through past copies, pin favorites, detect sensitive content, and manage your clipboard with a native COSMIC UI and full Wayland support.

![Status: Active Development](https://img.shields.io/badge/status-active%20development-green)
![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)
![Language: Rust](https://img.shields.io/badge/language-rust-orange)

---

## Features

- **Persistent clipboard history** - Never lose a copy again
- **Instant search** - Type to filter through history
- **Pin important items** - Keep frequently used content accessible
- **Rich content** - Text, HTML, images, and file lists
- **Sensitive content detection** - Passwords, API keys, OTP codes auto-flagged
- **Encryption at rest** - AES-256-GCM for sensitive items
- **Audit logging** - Track security-relevant events
- **Screen lock protection** - Auto-clear sensitive items on lock
- **Incognito mode** - Temporarily stop recording
- **Data export/import** - JSON backup and restore
- **Quick paste** - Paste via wtype/ydotool backends
- **IPC control** - Unix socket protocol for daemon communication
- **CLI tool** - `author-clipboard-ctl` for scripting and automation
- **Config file** - JSON configuration at `~/.config/author-clipboard/config.json`
- **Native theming** - Follows COSMIC light/dark themes automatically
- **Global shortcut** - Configurable keyboard shortcut (default: Super+V)
- **Graceful shutdown** - Clean socket cleanup on exit

---

## Quick Start

### Prerequisites

- Linux with COSMIC desktop environment
- Rust toolchain (1.70+)
- Wayland compositor with `wlr-data-control` support

### Build and Run

```bash
git clone https://github.com/namikofficial/author-clipboard
cd author-clipboard
just setup         # Install dev tools
just build         # Build all components
just daemon        # Run clipboard daemon
just applet        # Run GUI applet
```

### CLI Tool

```bash
# Control the daemon
author-clipboard-ctl toggle          # Toggle visibility
author-clipboard-ctl show            # Show applet
author-clipboard-ctl hide            # Hide applet
author-clipboard-ctl ping            # Check daemon status

# Query clipboard
author-clipboard-ctl history         # List recent items
author-clipboard-ctl status          # Show database stats
author-clipboard-ctl clear           # Clear unpinned items

# Data management
author-clipboard-ctl export out.json # Export history
author-clipboard-ctl config          # Show current config
```

### Keyboard Shortcut (Super+V)

To open the clipboard picker with Super+V, add a custom shortcut in COSMIC Settings:

1. Open **COSMIC Settings** → **Keyboard** → **Custom Shortcuts**
2. Click **Add Shortcut**
3. Set the command to: `author-clipboard-ctl toggle`
4. Press **Super+V** as the key combination
5. Save

The shortcut sends a toggle command to the daemon, which signals the applet to come to the foreground.

### Configuration

Config file location: `~/.config/author-clipboard/config.json`

```json
{
  "max_items": 500,
  "max_age_days": 30,
  "keyboard_shortcut": "Super+V",
  "clear_on_lock": true,
  "encrypt_sensitive": false
}
```

---

## Architecture

```
author-clipboard/
├── crates/
│   ├── clipboard-daemon/    # Background Wayland clipboard watcher
│   ├── applet/              # COSMIC UI applet (popup interface)
│   ├── ctl/                 # CLI control tool
│   └── shared/              # Common library (DB, types, config, IPC)
├── data/                    # Desktop files, systemd services
├── resources/               # Icons, assets
└── docs/                    # Documentation
```

### Components

| Component | Binary | Purpose |
|-----------|--------|---------|
| **clipboard-daemon** | `author-clipboard-daemon` | Monitors Wayland clipboard, stores history in SQLite |
| **applet** | `author-clipboard` | COSMIC UI with history, search, pins, export/import |
| **ctl** | `author-clipboard-ctl` | CLI tool for scripting and daemon control via IPC |
| **shared** | *(library)* | Database, types, config, encryption, IPC, sensitive detection |

---

## Development

### Using the justfile

```bash
just                # Show available commands
just build         # Build all crates
just check         # Quick check without full build
just test          # Run tests
just fmt           # Format code
just lint          # Run clippy (strict: -D warnings)
just verify        # Full verification (fmt + lint + test + build)
just daemon        # Run clipboard daemon
just applet        # Run GUI applet
just dev           # Watch mode for development
```

### Development Phases

| Phase | Goal | Status |
|-------|------|--------|
| **Phase 0** | Clipboard watcher prototype | Done |
| **Phase 1** | Text history + basic UI | Done |
| **Phase 2** | Global shortcut + IPC | Done |
| **Phase 3** | Rich content (HTML, files) | Done |
| **Phase 5** | Quick paste + file handling | Done |
| **Phase 7** | Security + privacy features | Done |
| **Phase 8** | CLI tool + config + graceful shutdown | Done |
| **Phase 9** | Documentation + quality | In Progress |

See [PROJECT_PLAN.md](PROJECT_PLAN.md) for the detailed roadmap.

---

## Documentation

- **[PROJECT_PLAN.md](PROJECT_PLAN.md)** - Development phases and feature specifications
- **[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)** - How to contribute
- **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Development tooling and workflow
- **[docs/LOCAL_TESTING.md](docs/LOCAL_TESTING.md)** - Step-by-step local testing guide

---

## Contributing

Contributions are welcome! Please read [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) first.

```bash
just install-deps  # Install system dependencies
just setup         # Install Rust tools
just doctor        # Verify environment
just dev           # Start watch mode
just verify        # Run before committing
```

---

## License

GPL-3.0 - See [LICENSE](LICENSE) file for details.

---

## Related Projects

- [cosmic-utils/clipboard-manager](https://github.com/cosmic-utils/clipboard-manager) - Community COSMIC clipboard manager
- [pop-os/cosmic-applets](https://github.com/pop-os/cosmic-applets) - Official COSMIC applet examples

---

<div align="center">

**Built with love for the COSMIC desktop ecosystem**

</div>
