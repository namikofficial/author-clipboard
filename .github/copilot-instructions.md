# Copilot Instructions for author-clipboard

This document helps GitHub Copilot and Cline work effectively in the **author-clipboard** repository—a native COSMIC desktop clipboard manager written in Rust.

**For Open Source Contributors**: This guide covers the essential setup, conventions, and workflows needed to contribute effectively. Please read through the sections relevant to your task before starting.

## Quick Reference

### Build & Test Commands
```bash
# Verify code quality before committing
just verify        # Runs: fmt → lint → test → build

# Individual commands
just build         # Full build all crates
just check         # Quick syntax/type check (fast, no full build)
just test          # Run all tests
just fmt           # Format code
just lint          # Run clippy with strict warnings (-D warnings)
just lint-fix      # Auto-fix clippy warnings

# Run binaries
just daemon        # Run clipboard daemon
just applet        # Run GUI applet
just dev           # Watch mode (auto-rebuild on changes)
```

**Test single crates:**
```bash
cargo test -p author-clipboard-daemon
cargo test -p author-clipboard-applet
cargo test -p author-clipboard-shared
```

### Development Workflow
1. **Code → Test → Verify**: Always run `just verify` before committing to catch formatting, linting, and test failures
2. **Watch mode**: Use `just dev` during development for continuous feedback
3. **One-off builds**: Use `just check` for quick validation without full rebuild

## Architecture Overview

The project is a **Rust monorepo** with three crates:

```
author-clipboard (workspace root)
├── crates/clipboard-daemon/     # Wayland clipboard watcher service
│   ├── Monitors system clipboard for changes
│   ├── Extracts text content
│   └── Sends data to shared database
├── crates/applet/               # COSMIC UI applet (popup interface)
│   ├── Displays clipboard history
│   ├── Implements search/filter
│   └── Uses libcosmic for native theming
└── crates/shared/               # Common library
    ├── Database schema & operations
    ├── Configuration management
    └── Type definitions & utilities
```

### Key Dependencies
- **Tokio** (async runtime) - Used across daemon and applet
- **Wayland** (`wayland-client`, `wayland-protocols`, `wayland-protocols-wlr`) - Clipboard monitoring
- **SQLite** (`rusqlite`) - Clipboard history storage
- **libcosmic** (git from `pop-os/libcosmic`) - Native COSMIC UI toolkit
- **Tracing** - Structured logging

## Key Conventions

### Crate Naming
- Package names use `author-clipboard-*` prefix (e.g., `author-clipboard-daemon`)
- Binary names may differ (e.g., applet binary is just `author-clipboard`)
- Always check `Cargo.toml` in each crate for the exact name

### Workspace Dependencies
Dependencies are centralized in root `Cargo.toml` under `[workspace.dependencies]` and referenced with `.workspace = true` in crate `Cargo.toml` files. **Add new dependencies in root, not individual crates**, then reference them in crates.

### Development Phases
The project uses a structured phase approach documented in `PROJECT_PLAN.md`:
- **Phase 0**: Clipboard watcher prototype (IN PROGRESS)
- **Phase 1**: Text history + basic UI
- **Phase 2**: Global shortcut + polish
- **Phases 3-7**: Features (images, pickers, security, etc.)

Check phase status and detailed specs in `PROJECT_PLAN.md` before implementing features.

### Code Organization
- `shared/src/types.rs` - Shared type definitions (models, data structures)
- `shared/src/config.rs` - Configuration handling
- Daemon uses async Tokio for non-blocking Wayland polling
- Applet uses libcosmic's message-based event architecture

### Testing Expectations
- Write unit tests in the same file using `#[cfg(test)]` modules
- Use `#[tokio::test]` for async tests in daemon/applet
- Keep tests isolated and deterministic
- Run `cargo test --all` to ensure no regressions

## Development Environment Setup

### First-Time Setup
```bash
just install-deps  # Install system Wayland/COSMIC development packages
just setup         # Install Rust tools (rustfmt, clippy, cargo-watch)
just doctor        # Verify environment readiness
```

### Common Issues
- **Wayland library missing**: Run `just install-deps`
- **Clippy not found**: Run `just setup` to install clippy
- **Git libcosmic errors**: Ensure git is installed and you have internet access during `cargo build`

## Testing & Verification Strategy

### Before Committing
```bash
just verify  # Full check: format → lint → test → build
```

### Testing Daemon Specifically
```bash
# Run daemon with output (useful for debugging Wayland issues)
just daemon

# Run tests with output
cargo test -p author-clipboard-daemon -- --nocapture
```

### Testing Applet Specifically
```bash
# Run applet UI
just applet

# Run tests
cargo test -p author-clipboard-applet
```

### Testing Shared Library
```bash
cargo test -p author-clipboard-shared
```

## Important Patterns & Guidelines

### Error Handling
- Use `anyhow::Result` for fallible operations with context (`.context("description")`)
- Use `thiserror::Error` for public error types in libraries
- Wayland operations should gracefully handle disconnection

### Async Code (Daemon/Applet)
- All I/O is async via Tokio (clipboard monitoring, database, Wayland)
- Use `tokio::spawn` for background tasks
- Prefer `tokio::select!` for multiplexing operations

### Database (SQLite via rusqlite)
- Queries go in `shared/` library functions
- Use parameterized queries (`?` placeholders) to prevent SQL injection
- Keep schema definition in a single, clear location

### Logging & Debugging
- Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`)
- Subscriber is configured in daemon's main.rs
- Run with environment variable for debug output: `RUST_LOG=debug`

## Recommended Reading

- **`README.md`** - High-level feature overview and quick start
- **`PROJECT_PLAN.md`** - Detailed development roadmap, phases, and success criteria
- **`plans/SETUP_AND_DEV_GUIDE.md`** - Step-by-step setup for beginners
- **Cargo.toml** (root) - Workspace configuration and shared dependency versions

## When Adding Features

1. **Check the phase**: Ensure you're implementing a feature for the current phase (Phase 0 = watcher only)
2. **Update PROJECT_PLAN.md**: Mark deliverables as complete, update success criteria
3. **Test across crates**: Features may span daemon (data collection), shared (schema), and applet (UI)
4. **Run `just verify`**: Always verify before pushing changes
5. **Document in code**: Especially non-obvious Wayland protocol details

## Git Conventions

- Commits should reference the feature/phase they belong to (e.g., "Phase 0: Add wlr-data-control binding")
- Keep changes focused; one feature per PR when possible
- Run `just verify` before committing to catch issues early

---

## MCP Servers & AI-Assisted Development Setup

This project is optimized for GitHub Copilot and Cline. Set up the following MCP (Model Context Protocol) servers to enhance your development experience.

### **Serena MCP (Recommended for Code Intelligence)**

Serena provides advanced code search, symbol navigation, and file understanding—essential for working efficiently in a Rust monorepo.

**Installation (Cline with Claude models):**

1. Add to `.cline_rules.json`:
```json
{
  "mcpServers": {
    "serena": {
      "command": "npx",
      "args": ["@oraios-ai/serena"],
      "env": {
        "SERENA_MODE": "codebase"
      }
    }
  }
}
```

2. For VS Code + Copilot CLI, configure in `.vscode/settings.json`:
```json
{
  "github.copilot.enable": {
    "*": true,
    "plaintext": true,
    "markdown": true
  }
}
```

**Key Serena capabilities for this project:**
- Symbol search across monorepo (find `ClipboardItem`, `WaylandManager`, etc.)
- Fast file pattern matching (`**/*.rs`, `crates/*/src/`)
- Code intelligence for Rust (method navigation, type hierarchy)
- Dependency analysis (find where `wayland-client` is used)

**Usage examples:**
```
"Find all references to the ClipboardItem struct"
"Show me the WaylandDisplay implementation"
"Search for async functions in the daemon crate"
"Where is the Wayland clipboard connection established?"
```

### **GitHub MCP (Integration with Issues & PRs)**

Connects to GitHub for seamless issue/PR workflows.

**Configuration in `.cline_rules.json`:**
```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "your_github_token"
      }
    }
  }
}
```

**Useful for:**
- Linking code changes to issues
- Checking PR review comments
- Browsing the project's issue backlog

---

## Windows 11 Win+V Clipboard Design Inspiration

While building for COSMIC desktop, this project draws design inspiration from **Windows 11's Win+V clipboard picker**:

- **Instant history access** - Global shortcut opens clipboard picker
- **Search & filter** - Find past copies by typing
- **Pin favorites** - Keep frequently used items accessible
- **Visual preview** - See content at a glance
- **Quick paste** - Single-click restore to clipboard

The COSMIC implementation uses native Wayland protocols and libcosmic styling to deliver similar UX on Linux.

---

## Contributing to author-clipboard

### For Open Source Contributors

**Before you start:**

1. **Check the current phase** in `PROJECT_PLAN.md` - work should align with active development phase
2. **Read the architecture section** above - understand the three-crate structure
3. **Run the setup** to ensure your environment is ready:
   ```bash
   just install-deps  # System packages
   just setup         # Rust tools
   just doctor        # Verify everything
   ```

4. **Set up MCP servers** (optional but recommended) - especially Serena for code navigation

### Typical Contribution Workflow

1. **Pick a task** from `PROJECT_PLAN.md` or open issues
2. **Create a branch** for your work
3. **Develop locally** using `just dev` (watch mode)
4. **Run tests** frequently: `cargo test -p <crate-name>`
5. **Verify before push** with `just verify` (full check)
6. **Update documentation** if adding features (especially `PROJECT_PLAN.md`)
7. **Submit PR** with clear description of changes

### Code Review Checklist

Before submitting a PR, ensure:
- [ ] `just verify` passes with no errors
- [ ] Tests added for new functionality
- [ ] Code follows Rust conventions (rustfmt enforced)
- [ ] No clippy warnings (strict mode: `-D warnings`)
- [ ] Commit messages reference the feature/phase
- [ ] `PROJECT_PLAN.md` updated if completing deliverables

### Getting Help

- **Architecture questions**: Check `PROJECT_PLAN.md` and `plans/SETUP_AND_DEV_GUIDE.md`
- **Wayland-specific questions**: See comments in `crates/clipboard-daemon/` and referenced `wayland-protocols` documentation
- **COSMIC/libcosmic questions**: Refer to [libcosmic examples](https://github.com/pop-os/libcosmic)
- **Rust async patterns**: See Tokio documentation for `select!`, `spawn`, etc.

### Development Environment Troubleshooting

| Issue | Solution |
|-------|----------|
| `libwayland-dev` not found | Run `just install-deps` to install system packages |
| Clippy/rustfmt missing | Run `just setup` to install Rust tools |
| libcosmic build fails | Ensure git is available; try `cargo update` |
| Test failures | Run `cargo test --all` to see full output; check RUST_LOG=debug |

---

## IDE Configuration for Contributors

### VS Code + Copilot CLI
- Install **rust-analyzer** extension
- Install **GitHub Copilot** extension
- Configure in `.vscode/settings.json` (provided)
- Rust-analyzer will auto-check with clippy
- Auto-format on save (rustfmt)

### VS Code + Cline
- Install **Cline** extension
- Configure `.cline_rules.json` with Serena and GitHub MCPs
- Cline will use these for autonomous coding tasks
- Perfect for batch refactoring or large features

### Neovim
- Use **Cline** or **Copilot CLI** from terminal
- `.cline_rules.json` applies when using Cline
- Configure Copilot CLI as per GitHub's documentation
