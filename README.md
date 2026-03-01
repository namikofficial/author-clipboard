# author-clipboard

> A fast, native clipboard manager for COSMIC desktop, built entirely in Rust

**author-clipboard** delivers a powerful clipboard experience designed for COSMIC DE. Store clipboard history, search through past copies, and access emoji/GIF/symbol pickers - all with native COSMIC theming and Wayland support.

![Status: Early Development](https://img.shields.io/badge/status-early%20development-yellow)
![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)
![Language: Rust](https://img.shields.io/badge/language-rust-orange)

---

## 🎯 Vision

The missing productivity tool for COSMIC desktop users. A beautiful, fast, native clipboard manager that delivers a modern clipboard experience, built from the ground up for COSMIC's Wayland-native architecture.

### ✨ Key Features (Planned)

- 📋 **Persistent clipboard history** - Never lose a copy again
- 🔍 **Instant search** - Type to filter through history  
- 📌 **Pin important items** - Keep frequently used content accessible
- 😀 **Emoji picker** - Full Unicode emoji support with search
- 🎬 **GIF search** - Tenor API integration for animated content
- 🔣 **Symbol picker** - Math, currency, arrows, and special characters
- 🎨 **Native theming** - Follows COSMIC light/dark themes automatically
- ⚡ **Global shortcut** - Super+V (or configurable) opens from anywhere
- 🖼️ **Image support** - Copy and paste images with thumbnails
- 🔒 **Privacy-focused** - Local storage, no cloud sync
- 🛡️ **Security controls** - Sensitive item detection, encryption at rest

---

## 🚀 Quick Start

### Prerequisites

- Linux with COSMIC desktop environment
- Rust toolchain (1.70+)

### Development Setup

1. **Clone and setup:**
   ```bash
   git clone https://github.com/namik/author-clipboard
   cd author-clipboard
   just setup  # Install dev tools and dependencies
   ```

2. **Build and run:**
   ```bash
   just build      # Build all components
   just daemon     # Run clipboard daemon
   just applet     # Run GUI applet
   ```

3. **Development workflow:**
   ```bash
   just dev        # Watch mode - rebuilds on changes
   just check      # Quick syntax/type check
   just verify     # Full check (format, lint, test, build)
   ```

> **Note:** Until Phase 1 is complete, the components are minimal placeholders.

---

## 🏗️ Architecture

```
author-clipboard/
├── crates/
│   ├── clipboard-daemon/    # Background service (Wayland clipboard watcher)
│   ├── applet/             # COSMIC UI applet (popup interface) 
│   └── shared/             # Common types, database, configuration
├── data/                   # Desktop files, systemd services
├── resources/              # Icons, assets
└── docs/                   # Documentation
```

### Components

| Component | Purpose | Status |
|-----------|---------|---------|
| **clipboard-daemon** | Background service that monitors Wayland clipboard and stores history | 🏗️ In Progress |
| **applet** | COSMIC UI popup for browsing/selecting from history | 📋 Planned |
| **shared** | Database, types, and utilities shared between components | ⚡ Basic |

---

## 📅 Development Phases

| Phase | Goal | Status |
|-------|------|--------|
| **Phase 0** | Clipboard watcher prototype | 🏗️ **In Progress** |
| **Phase 1** | Text history + basic UI | 📋 Planned |
| **Phase 2** | Global shortcut + polish | 📋 Planned |
| **Phase 3** | Image support | 📋 Planned |
| **Phase 4** | Emoji/GIF/symbol pickers | 📋 Planned |
| **Phase 5** | Quick paste + file support | 📋 Planned |
| **Phase 6** | Settings + final polish | 📋 Planned |
| **Phase 7** | Security + privacy features | 📋 Planned |

👉 **See [PROJECT_PLAN.md](PROJECT_PLAN.md) for detailed roadmap**

---

## 🔧 Development

### Using the justfile

This project uses [just](https://github.com/casey/just) for common tasks:

```bash
just                # Show available commands
just build         # Build all crates
just check         # Quick check without full build  
just test          # Run tests
just fmt           # Format code
just lint          # Run clippy
just verify        # Full verification (fmt + lint + test + build)
just daemon        # Run clipboard daemon
just applet        # Run GUI applet  
just dev           # Watch mode for development
just clean         # Clean build artifacts
just setup         # One-time development setup
just install-deps  # Install system dependencies
just doctor        # Check development environment
```

### Project Structure

```bash
# Key files
├── justfile                 # Task runner commands
├── Cargo.toml              # Workspace definition  
├── PROJECT_PLAN.md         # Detailed development roadmap
└── docs/                   # Developer documentation
    ├── CONTRIBUTING.md      # Contributor guide
    ├── DEVELOPMENT.md       # Tooling & workflow reference
    └── LOCAL_TESTING.md     # Step-by-step testing guide

# Source code
├── crates/
│   ├── clipboard-daemon/   # Wayland clipboard monitoring
│   ├── applet/            # COSMIC UI application
│   └── shared/            # Common library (DB, types, config)
```

---

## 📖 Documentation

- **[PROJECT_PLAN.md](PROJECT_PLAN.md)** - Detailed development phases and feature specifications
- **[docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)** - How to contribute
- **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Development tooling and workflow
- **[docs/LOCAL_TESTING.md](docs/LOCAL_TESTING.md)** - Step-by-step local testing guide

---

## 🤝 Contributing

This is an early-stage project. Contributions are welcome once Phase 1 (basic functionality) is complete.

### Development Environment

```bash
# First-time setup
just install-deps  # Install system dependencies
just setup         # Install Rust tools
just doctor        # Verify environment

# Daily workflow
just dev           # Start watch mode
just verify        # Before committing
```

---

## 📄 License

GPL-3.0 - See [LICENSE](LICENSE) file for details.

---

## 🔗 Related Projects

- [cosmic-utils/clipboard-manager](https://github.com/cosmic-utils/clipboard-manager) - Community COSMIC clipboard manager
- [pop-os/cosmic-applets](https://github.com/pop-os/cosmic-applets) - Official COSMIC applet examples
- [gustavosett/Windows-11-Clipboard-History-For-Linux](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux) - Similar project (Tauri+React)

---

**Status:** 🏗️ **Phase 0 - Clipboard Watcher Prototype**  
**Next Milestone:** Basic Wayland clipboard monitoring working on COSMIC

---

<div align="center">

**Built with ❤️ for the COSMIC desktop ecosystem**

</div>