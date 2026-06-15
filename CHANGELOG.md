# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).


## [Unreleased]

### 🎨 Style

- **mcp-server:** apply rustfmt and fix unused variable warnings
- allow clippy struct_excessive_bools lint on ClipboardItem

### 🏗️ Build

- **arch:** add gtk4 deps and install hypr-picker binary
- **flake:** add gtk4 and gtk4-layer-shell to buildInputs
- add Nix flake for reproducible builds and dev environment

### 🐛 Bug Fixes

- **applet:** update debian asset paths and add hypr-picker binary
- **applet:** resolve main.rs conflicts and stabilize picker navigation
- **arch:** fix PKGBUILD source format and .SRCINFO
- **arch:** fix PKGBUILD deps and re-enable CI
- **audit:** lint cleanup and branch audit
- **ci:** fix SRCDEST permissions for builder user
- **ci:** fix BUILDDIR permissions for builder user
- **ci:** add gtk4-layer-shell build deps to Arch CI
- **ci:** run makepkg as non-root user in Arch container
- **ci:** add git to Arch container deps
- **ci:** fix Arch makepkg root issue and Debian apt failures
- **ci:** add permissions block and fix Arch pkg name
- **ci:** fix changelog blank lines and skipped commits in cliff.toml
- **clippy:** use is_ok_and instead of map + unwrap_or in tool_exists
- **mcp-server:** adjust params borrowing and clean up imports
- **metainfo:** drop unreleased 0.4.0/0.5.0 entries and correct 0.3.1 date
- **security:** re-derive sensitive flag on import
- **security:** run sensitive detection in new_html and new_files
- **shared:** prefix unused error variables with underscore in db.rs
- resolve 6 picker navigation/scroll desync bugs

### 👷 CI

- add arch and debian packaging jobs to workflows
- optimize CI to single job, add tag-triggered release workflow

### 📚 Documentation

- **changelog:** update changelog
- **changelog:** add Unreleased dev section and curated highlights
- **changelog:** update changelog
- **hyprland:** add demo section with reproducible shell transcript and layout
- **mcp:** update task plan with completed MCP server tasks
- **packaging:** expand install guides for Flatpak, AppImage, and NixOS
- **plan:** clarify 'Project Status' refers to dev branch, not release
- **plan:** mark Phase 19 (Hyprland-native UX & wlroots polish) complete
- **plan:** mark phase 18 distribution packaging as complete
- **readme:** add branch status banner and clarify dev vs main
- **security:** clarify branch status and supported versions
- **specs:** add hardening & polish pass feature brief (022)
- **specs:** add feature 021 brief and requirements for hyprland polish
- **specs:** add distribution and release artifacts feature spec 020
- **specs:** mark dedup-fix and config-cleanup tasks as completed in Phase 16
- **specs:** mark 012-service-api tasks T001-T006 as completed
- **specs:** add specifications
- **waybar:** add Waybar module integration with status script and README
- add PR 0 handoff doc for feature 023
- human intervention in code started
- add Flatpak build notes and COSMIC Store submission guide
- add AUR submission guide and release runbook
- add project roadmap
- add spec-driven development workflow and feature specs
- add AGENTS.md to vide code it now

### 🔧 Refactoring

- **flake:** clean up build and library path configuration
- **mcp-server:** update handler to match shared crate API changes
- **mcp-server:** use CopyMode enum for clipboard.copy handler
- **ui-gtk:** replace Rc<Cell<String>> with Rc<RefCell<String>> in SearchEntry2
- rename SearchFilters to FilterOptions, add CopyMode import

### 🔨 Miscellaneous

- **ci:** verify PKGBUILD structure instead of building
- **ci:** simplify Arch PKGBUILD verification
- **ci:** disable arch-pkg and deb jobs temporarily
- **ctl:** collapse match arms and tidy JSON formatting
- **mcp-server:** apply rustfmt formatting and silence unused variables
- **merge:** merge origin/main into dev (changelog CI workflow)
- **packaging:** add menu picker optdepends for wofi, fuzzel, and rofi
- **shared:** wrap EncryptionManager in backticks for consistency
- Merge pull request #4 from namikofficial/dev
- Merge pull request #2 from namikofficial/dev
- Merge branch 'feat/023-popup-bugs' into dev

### 🚀 Features

- **applet:** enhance header with daemon indicator, item count, and tooltips
- **applet:** route clipboard operations through IPC daemon with fallback
- **ci:** add automatic changelog update workflow on push to main
- **ci:** add automatic changelog update workflow on push to main
- **ctl:** add --json and --pretty flags to status command
- **db:** add encryption-at-rest insert and decrypt methods
- **db:** add get_most_recent query for status bar
- **hypr-picker:** add --filter, just ui-check/ui-smoke, update docs (PR 7)
- **ipc:** add mime Option to IpcCommand::Copy with serde(default)
- **ipc:** include daemon_pid in Ping and Status responses
- **justfile:** add Waybar module install and check target
- **justfile:** add release, packaging, and inspection recipes
- **mcp-server:** add MCP server with clipboard service handler
- **packaging:** add AppImage build script and metadata
- **security:** add HTML-aware sensitive content check
- **shared:** thread PickerFilter through filter_and_query + build_external_rows
- **shared:** add encryption metadata to ClipboardItem
- **storage:** add encryption metadata columns and redacted_preview
- **ui:** complete unified GTK4 UI with all widgets, pages, and bug fixes
- **ui:** add unified GTK4 + libadwaita UI library (skeleton)
- **ui-gtk:** manager rewrite with AdwOverlaySplitView + sidebar + GSettings persistence (PR 6)
- **ui-gtk:** feature-gate webkit6 behind webview feature (PR 5.5)
- **ui-gtk:** implement key controller, GSettings bindings, fix preview.rs API
- **ui-gtk:** PreviewPane for text / image / files / sensitive
- **ui-gtk:** complete reducer (pin/star/delete/reveal/window/settings/snippets/daemon)
- **ui-gtk:** introduce AppState + Action + reduce() foundation slice
- **ui-gtk:** thread ClipboardPageProps through clipboard page
- merge dev into main (Phase 18-19 release)
- add non-flake default.nix package build
- add mcp-server crate and normalize ipc commands
- external picker mode and sensitive item confirmation to picker functionality
- add emoji, kaomoji, symbols modules and picker logic
- enhance clipboard functionality with Wayland support and external picker integration
- documentation and improve clipboard manager functionality
- dead code allowance for icon field and update installation paths
- enhance scroll & manage scroll offset in clipboard manager
- update version to 0.5.0 and format libcosmic dependencies
- update version to 0.4.0 & enhance clipboard manager functionality
- add security policy documentation

### 🧪 Testing

- **db:** add encryption at rest invariant tests
- **shared:** mark fixture sensitive to cover encryption path

## [0.3.1] - 2026-03-02

### 🐛 Bug Fixes

- scroll position, layout, URI detection, IPC security

### 📚 Documentation

- add Phase 11-14 to project plan, update status
- add LICENSE, fix config, add install guide

## [0.3.0] - 2026-03-01

### 🐛 Bug Fixes

- **applet:** scroll follows selection, Enter works

### 📚 Documentation

- update project plan and README for phases 9-10

### 🚀 Features

- **applet:** add advanced keyboard navigation
- **applet:** use COSMIC icons and daemon status

## [0.2.0] - 2026-03-01

### 🐛 Bug Fixes

- **applet:** escape closes, click pastes, keyboard nav works
- **daemon:** fix Wayland clipboard capture crash and hang
- **pre-commit:** improve comments and streamline staged file checks
- update repository URL to use the official GitHub account

### 📚 Documentation

- **development:** dev guide with tooling and workflow details
- **shared:** add rustdoc comments to public API
- update README with applet usage and install instructions
- add Super+V keyboard shortcut setup instructions
- mark Phase 8 as complete in project plan
- regenerate changelog from git history
- update README with current features and CLI reference
- 📋 mark Phase 2 and Phase 5 as complete
- update documentation references and remove outdated README
- enhance local testing guide with steps & troubleshooting tips
- add contributing guide for author-clipboard

### 📦 Dependencies

- **deps:** update libcosmic and add chrono to applet workspace

### 🔧 Refactoring

- **applet:** remove signal-file visibility toggle

### 🔨 Miscellaneous

- update Cargo.lock for clap dependency
- add clap dependency and ctl crate to workspace
- Add README.md with project overview, features, and development setup
- Add .gitignore and project.yml for Serena configuration
- Initialize author-clipboard workspace with multiple crates
- Add .gitignore file for Rust workspace to exclude build artifacts and temporary files

### 🚀 Features

- **applet:** toggle functionality for applet launch and termination
- **applet:** add window visibility toggle functionality
- **applet:** add visibility toggle via daemon signal file
- **applet:** ⌨️ add quick paste UI and enhanced file display
- **applet:** implement initial application structure and UI components
- **clipboard-daemon:** integrate database and config for history state
- **ctl:** add CLI control tool with IPC commands
- **daemon:** add graceful shutdown and CLI help
- **daemon:** 🎯 add IPC server for shortcut activation
- **daemon:** clear sensitive clipboard items on screen lock
- **database:** clipboard item management with deduplication & stats
- **deps:** add image processing library and update deps in Cargo.toml
- **emoji:** emoji categories and search functionality for emoji picker
- **env:** .env.example for dev config & load settings in justfile
- **pre-commit:** check only staged Rust files
- **project-plan:** update development phases to reflect progress
- **readme:** add comprehensive overview & phased development plan
- **serena:** overview, coding conventions, suggested commands etc
- **shared:** add config file load and save support
- **shared:** 📦 register new modules in shared crate
- **shared:** 📁 add file handler with metadata extraction
- **shared:** ⚡ add quick paste module with wtype integration
- **shared:** 🔌 add IPC module for daemon-applet communication
- **shared:** 🔑 add shortcut parsing and conflict detection
- **shared:** add data export/import and update project plan
- support for HTML & file list clipboard items & schema updates
- add image handling and incognito mode support
- add changelog generation and release tasks to justfile
- add clipboard daemon and applet with Wayland support


