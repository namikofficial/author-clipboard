# Code Conventions & Style

## Formatting (rustfmt.toml)
- Max line width: 100 chars
- 4-space indent
- Unix newlines (LF)
- Imports grouped: std → external → crate (group_imports = "StdExternalCrate")
- Module-level import granularity

## Linting (Cargo.toml workspace lints)
- clippy::correctness = deny
- clippy::suspicious, complexity, perf, style, pedantic = warn
- Allowed: module_name_repetitions, must_use_candidate, missing_errors_doc, missing_panics_doc
- unsafe_code = warn

## Error Handling
- `anyhow::Result` for application code with `.context("description")`
- `thiserror::Error` for public error types in shared library
- Wayland operations: gracefully handle disconnection

## Logging
- Use `tracing` macros: `info!`, `warn!`, `error!`, `debug!`
- Subscriber configured in each binary's main.rs
- Debug output: `RUST_LOG=debug`

## Dependencies
- All versions defined in root `Cargo.toml` under `[workspace.dependencies]`
- Crates reference with `.workspace = true`
- Never add dependency versions in individual crate Cargo.toml

## Git Conventions
- Conventional Commits enforced by git hook
- Format: `type(scope): description`
- Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
- Scopes: daemon, applet, shared, deps
- Max subject line: 72 chars

## Testing
- Unit tests in same file with `#[cfg(test)]` modules
- `#[tokio::test]` for async tests
- Isolated and deterministic tests

## Naming
- Package names: `author-clipboard-*` prefix
- Binary names may differ from package names
- Rust standard naming: snake_case for functions/variables, PascalCase for types
