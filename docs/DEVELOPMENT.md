# Development Guide

> Complete guide to the development tooling, workflow, and conventions for author-clipboard.

## Tooling Overview

This project uses Rust-native tools that serve the same purpose as popular Node.js tools:

| Node.js Tool | Rust Equivalent | Config File | Command |
|-------------|----------------|-------------|---------|
| Prettier | `rustfmt` | `rustfmt.toml` | `just fmt` |
| ESLint | `clippy` | `Cargo.toml [workspace.lints]` | `just lint` |
| Husky | `.githooks/` | `git config core.hooksPath` | `just setup-hooks` |
| commitlint | `.githooks/commit-msg` | (built-in script) | (automatic on commit) |
| Jest/Vitest | `cargo test` | (built-in) | `just test` |
| nodemon | `cargo-watch` | (built-in) | `just dev` |

## Quick Reference

```bash
# ── Code Quality ──
just fmt           # Auto-format all code
just fmt-check     # Check formatting
just lint          # Run linter
just lint-fix      # Auto-fix lint issues
just fix           # Format + fix everything

# ── Build & Test ──
just check         # Quick type check (fast, no codegen)
just build         # Full build
just test          # Run all tests
just test-verbose  # Tests with stdout output

# ── Workflow ──
just dev           # Watch mode (auto-rebuild on changes)
just quick         # Quick check + test
just verify        # Full verification before commit

# ── Setup ──
just setup         # First-time dev environment setup
just setup-hooks   # Install git hooks only
just install-deps  # System dependencies (apt)
just doctor        # Health check
```

## Formatting (rustfmt)

Configured in `rustfmt.toml`. Key settings:

- **Max line width**: 100 characters
- **Indent**: 4 spaces
- **Import grouping**: std → external → crate (like `import/order` in ESLint)
- **Unix newlines**: LF only

Format is checked automatically:
- On save in VS Code (via rust-analyzer)
- In pre-commit hook
- In `just verify`

Run manually: `just fmt` (writes) or `just fmt-check` (check only).

## Linting (clippy)

Configured in root `Cargo.toml` under `[workspace.lints.clippy]`. Lint levels:

| Category | Level | Purpose |
|----------|-------|---------|
| `correctness` | deny | Bugs and logic errors |
| `suspicious` | warn | Code that looks wrong |
| `complexity` | warn | Unnecessarily complex code |
| `perf` | warn | Performance issues |
| `style` | warn | Idiomatic Rust style |
| `pedantic` | warn | Opinionated but useful checks |

Some pedantic lints are allowed to reduce noise (e.g., `module_name_repetitions`).

Run manually: `just lint` (check) or `just lint-fix` (auto-fix).

## Git Hooks

Installed via `just setup-hooks` (or `just setup`). Lives in `.githooks/`:

### Pre-commit Hook
Runs before every commit:
1. Checks formatting (`cargo fmt -- --check`)
2. Runs clippy (`cargo clippy -- -D warnings`)

If either fails, the commit is blocked with a helpful error message.

### Commit-msg Hook
Validates commit messages follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

**Valid types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

**Scopes** (optional): `daemon`, `applet`, `shared`, `deps`

**Examples**:
```bash
git commit -m "feat(daemon): add clipboard change detection"     # ✅
git commit -m "fix: handle empty content"                        # ✅
git commit -m "updated stuff"                                    # ❌ Rejected
git commit -m "Fix the thing"                                    # ❌ Rejected (capital, no type)
```

### Bypassing Hooks (Emergency Only)

```bash
git commit --no-verify -m "wip: temporary work"
```

## Changelog Generation

The project uses [git-cliff](https://git-cliff.org/) to auto-generate `CHANGELOG.md` from conventional commit messages. Configured in `cliff.toml`.

```bash
just changelog          # Generate/update CHANGELOG.md
just changelog-preview  # Preview unreleased changes (dry-run)
just release 0.2.0      # Tag release, generate changelog, commit
```

Commit types map to changelog sections:
- `feat` → 🚀 Features
- `fix` → 🐛 Bug Fixes
- `docs` → 📚 Documentation
- `perf` → ⚡ Performance
- `refactor` → 🔧 Refactoring
- `chore(deps)` → 📦 Dependencies

## VS Code Integration

Settings in `.vscode/settings.json`:

- **Format on save**: Enabled for Rust files via rust-analyzer
- **Clippy on save**: rust-analyzer runs clippy instead of `cargo check`
- **Trailing whitespace**: Auto-trimmed
- **Final newline**: Auto-inserted
- **Rulers**: Visible at 100 chars (matching rustfmt max_width)

### Recommended Extensions

- **rust-analyzer** - Rust language support
- **Even Better TOML** - Cargo.toml editing
- **Error Lens** - Inline error display
- **crates** - Dependency version management

## Workspace Lints

All crates inherit lints from the workspace. Each crate's `Cargo.toml` includes:

```toml
[lints]
workspace = true
```

This ensures consistent lint rules across all crates without duplication.

## Adding Dependencies

Always add dependencies to the **workspace root** first:

```toml
# In root Cargo.toml
[workspace.dependencies]
new-crate = "1.0"
```

Then reference in the crate that needs it:

```toml
# In crates/*/Cargo.toml
[dependencies]
new-crate.workspace = true
```

## Troubleshooting

### Pre-commit hook fails

```bash
# Fix formatting
just fmt

# Fix clippy issues
just lint-fix

# Then try committing again
```

### "hooks not running"

```bash
# Re-install hooks
just setup-hooks

# Verify
git config core.hooksPath  # Should show: .githooks
```

### clippy warnings you disagree with

Add an allow attribute locally:
```rust
#[allow(clippy::too_many_arguments)]
fn complex_function(...) { }
```

Or add to workspace config in `Cargo.toml` if it's project-wide.
