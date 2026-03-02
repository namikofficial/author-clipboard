# Contributing to author-clipboard

Thank you for your interest in contributing to author-clipboard! This guide will help you get started.

## Getting Started

### Prerequisites

- **Linux** with COSMIC desktop (or Pop!_OS 24.04+)
- **Rust** toolchain (1.70+) via [rustup](https://rustup.rs/)
- **just** command runner ([install](https://github.com/casey/just#installation))

### First-Time Setup

```bash
# 1. Clone the repository
git clone https://github.com/namikofficial/author-clipboard
cd author-clipboard

# 2. Install system dependencies (Ubuntu/Debian)
just install-deps

# 3. Set up development environment (Rust tools + git hooks)
just setup

# 4. Verify everything works
just doctor
```

### Verify Your Setup

```bash
just doctor    # Check tools, hooks, and workspace health
just build     # Should compile without errors
just verify    # Full check: format → lint → test → build
```

## Development Workflow

### Daily Development

```bash
just dev       # Watch mode - rebuilds on file changes
just check     # Quick syntax/type check (faster than build)
just quick     # Check + test (fast feedback loop)
```

### Before Committing

```bash
just verify    # Runs: fmt-check → lint → test → build
```

The git hooks will also catch issues:
- **Pre-commit hook**: Checks formatting and runs clippy
- **Commit-msg hook**: Enforces conventional commit format

**CI will run automatically** on push and PR via GitHub Actions (`.github/workflows/ci.yml`): format check → clippy → test → build.

### Auto-Fix Issues

```bash
just fix       # Auto-format + auto-fix clippy warnings
```

## Commit Convention

We use **[Conventional Commits](https://www.conventionalcommits.org/)** — enforced by a git hook.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | When to use |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting, no code change |
| `refactor` | Code change that's neither fix nor feature |
| `perf` | Performance improvement |
| `test` | Adding or fixing tests |
| `build` | Build system or dependencies |
| `ci` | CI/CD configuration |
| `chore` | Maintenance tasks |
| `revert` | Reverting a previous commit |

### Scopes (optional)

Use the crate name: `daemon`, `applet`, `shared`, or `deps` for dependency changes.

### Examples

```
feat(daemon): add wayland clipboard monitoring
fix(shared): handle empty clipboard content gracefully
docs: add conventional commits guide
refactor(applet): simplify message handling
chore(deps): bump rusqlite to 0.32
feat!: redesign clipboard item storage schema
```

### Breaking Changes

Add `!` after the type/scope, or add `BREAKING CHANGE:` in the footer:

```
feat(shared)!: change ClipboardItem id from i64 to uuid

BREAKING CHANGE: ClipboardItem.id is now a Uuid instead of i64.
Migration required for existing databases.
```

## Code Style

Code style is enforced automatically by the toolchain:

- **Formatting**: `rustfmt` with project config in `rustfmt.toml`
- **Linting**: `clippy` with workspace lints in `Cargo.toml`
- **Auto-format on save**: Configured in `.vscode/settings.json`

### Key Conventions

- Use `anyhow::Result` for fallible operations with `.context("description")`
- Use `thiserror::Error` for public error types in the shared library
- Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`) for logging
- Write doc comments (`///`) for all public items
- Keep functions focused and small

### Workspace Dependencies

Dependencies are managed centrally in the root `Cargo.toml`:

```toml
# Add new dependencies here, NOT in individual crate Cargo.toml files
[workspace.dependencies]
some-crate = "1.0"
```

Then reference in crate `Cargo.toml`:

```toml
[dependencies]
some-crate.workspace = true
```

## Project Structure

```
author-clipboard/
├── .github/
│   ├── workflows/ci.yml     # GitHub Actions CI (fmt, clippy, test, build)
│   └── CODEOWNERS           # Contribution area ownership
├── crates/
│   ├── clipboard-daemon/    # Background Wayland clipboard watcher
│   ├── applet/              # COSMIC UI applet (popup interface)
│   ├── ctl/                 # CLI control tool
│   └── shared/              # Common library (DB, types, config)
├── .githooks/               # Git hooks (pre-commit, commit-msg)
├── docs/                    # Developer documentation
├── data/                    # Desktop files, systemd services
├── resources/               # Icons, assets
├── rustfmt.toml             # Formatter configuration
├── justfile                 # Task runner commands
└── Cargo.toml               # Workspace + lint configuration
```

### Code Ownership

The `.github/CODEOWNERS` file defines ownership for contribution areas. PRs will automatically request reviews from the appropriate owners based on which files are changed.

## Testing

```bash
just test                                    # Run all tests
just test-verbose                            # Run with output
cargo test -p author-clipboard-daemon        # Test single crate
cargo test -p author-clipboard-shared        # Test shared library
```

### Writing Tests

- Place unit tests in the same file using `#[cfg(test)]` modules
- Use `#[tokio::test]` for async tests
- Keep tests isolated and deterministic

## Development Phases

Check `PROJECT_PLAN.md` for the current phase. Work should align with the active development phase.

## Getting Help

- **Architecture**: See `PROJECT_PLAN.md` and `docs/DEVELOPMENT.md`
- **Wayland**: Comments in `crates/clipboard-daemon/` and [protocol docs](https://wayland.app/protocols/)
- **COSMIC/libcosmic**: [libcosmic examples](https://github.com/pop-os/libcosmic)
- **Rust async**: [Tokio documentation](https://tokio.rs/)
