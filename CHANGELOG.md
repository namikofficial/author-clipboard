# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

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
