# author-clipboard - Project Overview

## Purpose
A native COSMIC desktop clipboard manager written in Rust. Provides clipboard history, search, and expression pickers (emoji, GIF, symbols) with Wayland-native integration.

## Tech Stack
- **Language**: Rust (edition 2021)
- **Async runtime**: Tokio
- **UI**: libcosmic (COSMIC desktop toolkit, based on Iced)
- **Database**: SQLite via rusqlite (bundled)
- **Clipboard**: Wayland wlr-data-control protocol
- **Logging**: tracing + tracing-subscriber
- **Error handling**: anyhow (applications), thiserror (libraries)
- **Build**: Cargo workspace, just (task runner)

## Architecture
Rust monorepo with 3 crates:
- `crates/clipboard-daemon/` → `author-clipboard-daemon` binary - Background Wayland clipboard watcher
- `crates/applet/` → `author-clipboard` binary - COSMIC UI popup applet
- `crates/shared/` → `author-clipboard-shared` library - Common types, DB, config

## Development Phases
- Phase 0: Clipboard watcher prototype (IN PROGRESS)
- Phase 1: Text history + basic UI
- Phase 2: Global shortcut + polish
- Phases 3-7: Rich content, expression pickers, security

## Key Files
- `Cargo.toml` - Workspace definition + lint config
- `justfile` - Task runner commands
- `rustfmt.toml` - Formatter config
- `.githooks/` - Pre-commit (fmt+clippy) and commit-msg (conventional commits) hooks
- `PROJECT_PLAN.md` - Detailed development roadmap
