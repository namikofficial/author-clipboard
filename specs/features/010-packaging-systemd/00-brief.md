# Feature Brief: Packaging & Systemd Integration

> Systemd user service, `just install` command, and documentation for Arch PKGBUILD and manual install.

---

## Problem Statement

Users need an easy path from source to running daemon, with proper service management and packaging docs.

## Proposed Solution

Systemd user service for auto-start, `just install` to build and install binaries, and documentation for various packaging methods.

## Goals

- Systemd user service with auto-restart
- `just install` builds release and installs all files
- `just enable/disable/status/logs` for service management
- `just uninstall` for clean removal
- Packaging docs for Arch, Debian, manual install

---

**Created**: Phase 13 Complete
**Status**: Implemented v0.5.0