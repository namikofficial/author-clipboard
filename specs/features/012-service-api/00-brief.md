# Feature Brief: Service API Normalization

> Transform the clipboard daemon into the single authoritative service for all clipboard operations, replacing the current split between daemon IPC control and CLI direct database access.

---

## Problem Statement

Currently, the CLI bypasses the daemon for most useful operations. History, export, clear, config, and copy workflows open the SQLite database directly, while the daemon's IPC surface only exposes visibility-oriented messages (Toggle, Show, Hide, Ping, Pong, Status). This creates:

1. **Multiple code paths** for the same operations (CLI direct DB vs daemon IPC)
2. **No centralized policy enforcement** (sensitivity, masking, retention)
3. **No audit trail** for CLI-initiated mutations
4. **No live updates** when CLI modifies the database
5. **A weak foundation** for MCP, AI integration, and remote access

## Proposed Solution

Redefine the daemon as the **single authoritative service** for all clipboard state. All clients (CLI, applet, hypr-picker, MCP server) must route operations through the daemon's IPC interface. The daemon owns all mutations, queries, policy enforcement, and audit logging.

## Goals

- CLI operations (history, copy, clear, export, config) go through daemon IPC
- All state mutations are logged in the audit log
- Live update notifications when state changes
- One consistent policy engine for masking, sensitivity, retention
- A stable, versioned service API suitable for MCP exposure
- All future features (MCP, filtering, collections) build on this foundation

## Non-Goals

- Breaking existing CLI command syntax (flags remain the same, routing changes)
- Removing database access entirely (daemon uses it internally)
- Implementing full RPC framework (stay with JSON-over-unix-socket)
- Supporting remote IPC in Phase 12 (stdin/stdout local MCP only)

## Stakeholders

All users of author-clipboard, especially those using CLI automation, MCP-enabled AI tools (Codex, OpenCode), and future remote access scenarios.

---

**Created**: Phase 15 (Post-Research)
**Status**: Draft