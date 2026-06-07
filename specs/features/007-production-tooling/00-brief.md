# Feature Brief: Production Tooling

> CLI control tool, JSON config file support, graceful daemon shutdown, and IPC socket management.

---

## Problem Statement

Users need a reliable way to control the daemon, manage configuration, and perform operations like clearing history or exporting data.

## Proposed Solution

A CLI tool (`author-clipboard-ctl`) that communicates with the daemon via Unix socket IPC, JSON config file for persistent settings, and proper socket cleanup on shutdown.

## Goals

- CLI tool with subcommands: toggle, show, hide, ping, history, status, clear, export, config, picker, doctor
- JSON config file in `~/.config/author-clipboard/config.json`
- Graceful daemon shutdown with socket cleanup
- `--help` and `--version` support

---

**Created**: Phase 8 Complete
**Status**: Implemented v0.5.0