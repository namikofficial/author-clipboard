# Suggested Commands

## Build & Check
- `just build` - Build all crates
- `just check` - Quick syntax/type check (fast, no codegen)
- `just build-release` - Build optimized release binary

## Code Quality
- `just fmt` - Format all code (rustfmt)
- `just fmt-check` - Check formatting without changes
- `just lint` - Run clippy linter with -D warnings
- `just lint-fix` - Auto-fix clippy warnings
- `just fix` - Format + fix everything

## Testing
- `just test` - Run all tests
- `just test-verbose` - Tests with stdout output
- `cargo test -p author-clipboard-daemon` - Test single crate
- `cargo test -p author-clipboard-shared` - Test shared lib

## Workflow
- `just dev` - Watch mode (auto-rebuild on changes)
- `just quick` - Quick check + test
- `just verify` - Full verification: fmt-check → lint → test → build

## Run
- `just daemon` - Run clipboard daemon
- `just applet` - Run GUI applet

## Setup
- `just setup` - First-time dev environment setup (installs tools + hooks)
- `just setup-hooks` - Install git hooks only
- `just install-deps` - System dependencies (apt)
- `just doctor` - Health check

## Changelog & Releases
- `just changelog` - Generate/update CHANGELOG.md from conventional commits
- `just changelog-preview` - Preview unreleased changes (dry-run)
- `just release 0.2.0` - Tag release, generate changelog, commit

## Maintenance
- `just clean` - Clean build artifacts
- `just update` - Update dependencies
- `just reset` - Clean + remove Cargo.lock
