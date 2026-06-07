# AGENTS.md — author-clipboard

## Spec-Driven Development

**Prompt creates spec. Spec creates code. Code must pass spec.**

This project uses GitHub Spec Kit style development. All non-trivial changes follow this workflow:

```
idea → 00-brief.md → 01-requirements.md → 05-technical-design.md → 06-task-plan.md → implementation → 08-review-checklist.md
```

### Feature Spec Structure

All features have specs under `/specs/features/FFF-name/`:

```
00-brief.md           # What problem does this solve?
01-requirements.md    # User stories, acceptance criteria
02-domain-model.md    # Data structures, state
03-api-contract.md    # IPC commands, CLI changes
04-ui-flow.md         # User interaction flows
05-technical-design.md # Implementation approach
06-task-plan.md       # Atomic, verifiable tasks
07-test-plan.md       # Test strategy
08-review-checklist.md # Pre-merge checklist
09-decisions.md       # Architectural decisions
```

### Agent Roles

| Role | Responsibility |
|------|----------------|
| **Spec Agent** | Turn rough intent into requirements, define acceptance criteria |
| **Architect Agent** | Propose technical design, preserve conventions |
| **Task Planner** | Convert specs into atomic, independently verifiable tasks |
| **Implementation Agent** | Execute one task at a time, verify against spec |
| **Review Agent** | Check requirements coverage, security, test quality |

### Rule Summary

1. **Do not write code first.** Create or update a feature spec.
2. **Requirements need acceptance criteria.** Each criterion must be independently verifiable.
3. **Design must list affected files.** No unspecified side-effects.
4. **Tasks must be atomic.** One task, one goal, one verification command.
5. **Implement one task at a time.** No combining unrelated changes.
6. **Verify against spec.** Run tests after implementation.
7. **Document deviations.** Update `09-decisions.md` if implementation differs from design.
8. **No unrelated refactors.** Stay focused on the current task.

---

## Build & Verify Commands

```bash
just verify        # fmt → lint → test → build (full check before commit)
just build         # build all crates
just check         # quick type check (no full build)
just test          # run all tests
just fmt           # format code
just lint          # clippy with -D warnings

cargo test -p author-clipboard-daemon   # test single crate
cargo test -p author-clipboard-shared
cargo test -p author-clipboard-applet
```

## Crate Structure

```
crates/
├── clipboard-daemon/   # Wayland clipboard watcher + IPC daemon
├── applet/             # libcosmic popup UI (binary: author-clipboard)
├── shared/             # DB schema, config, types, image_store
├── ctl/                # CLI tool (binary: author-clipboard-ctl)
└── hypr-picker/        # GTK4 layer-shell native picker for Hyprland
```

## Dependency Management

Add dependencies to **root `Cargo.toml`** under `[workspace.dependencies]`, then reference in crate Cargo.toml:

```toml
# root Cargo.toml
[workspace.dependencies]
new-crate = "1.0"

# crates/some-crate/Cargo.toml
[dependencies]
new-crate.workspace = true
```

## Environment Setup

- The justfile auto-loads `.env` (dotenv-load is set). Copy `.env.example` to `.env` for development.
- **COSMIC desktop**: `COSMIC_DATA_CONTROL_ENABLED=1` is required for clipboard monitoring. Set in `.env` or `~/.config/cosmic-comp/env`.
- **Hyprland/Sway**: Do NOT set `COSMIC_DATA_CONTROL_ENABLED`; compositor must expose `wlr-data-control`.

## Incognito Mode

Daemon skips capture when `<data_dir>/.incognito` file exists. Create/remove to toggle:

```bash
touch ~/.local/share/author-clipboard/.incognito  # enable
rm ~/.local/share/author-clipboard/.incognito    # disable
```

## Git Hooks

- **pre-commit**: runs `cargo fmt -- --check` and `cargo clippy -- -D warnings`. Hooks blocked if either fails.
- **commit-msg**: enforces [Conventional Commits](https://www.conventionalcommits.org/). Format: `<type>(<scope>): <description>`. Valid types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.

## Special Dependencies

- **libcosmic**: fetched from `git = "https://github.com/pop-os/libcosmic.git"`. Requires git + internet at build time.
- **rusqlite**: uses `features = ["bundled"]` (SQLite compiled from source).

## Running Binaries

```bash
just daemon         # clipboard daemon (Wayland monitor)
just applet         # libcosmic popup UI
just run            # daemon in background + applet foreground (end-to-end test)
just dev            # watch mode (auto-rebuild on changes)
```

## Key Paths

- Config: `~/.config/author-clipboard/config.json`
- Database: `<data_dir>/clipboard.db`
- Images: `<data_dir>/images`, `<data_dir>/thumbnails`
- IPC socket: `$XDG_RUNTIME_DIR/author-clipboard` (fallback: `<cache_dir>/author-clipboard`)
- Encryption key: `<data_dir>/.encryption_key` (mode 0600)

## Reference

- `justfile` — exact build/test/run commands
- `specs/000-product/constitution.md` — non-negotiable project rules
- `specs/000-product/architecture.md` — system design
- `specs/000-product/glossary.md` — terminology
- `specs/features/` — feature specifications
- `docs/DEVELOPMENT.md` — tooling, lint config, git hooks detail
- `docs/LOCAL_TESTING.md` — daemon/applet testing, Wayland troubleshooting
- `.github/copilot-instructions.md` — architecture overview, MCP setup