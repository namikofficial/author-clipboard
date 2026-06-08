# Feature Brief: MCP Protocol Server

> A Model Context Protocol (MCP) server that exposes clipboard history as tools, resources, and prompts for AI coding agents like Codex and OpenCode.

---

## Problem Statement

AI coding agents (Codex, OpenCode, GitHub Copilot) cannot access clipboard history, snippets, or quick-paste functionality. This limits their ability to:
- Retrieve previously copied code snippets
- Search clipboard history for API patterns or configurations
- Incorporate clipboard context into code generation
- Access user-defined snippets without manual copy/paste

## Proposed Solution

Implement an MCP server (`author-clipboard-mcp`) that:
1. Exposes clipboard tools for search, copy, pin, delete operations
2. Provides URI-addressable resources for browsing clipboard state
3. Offers prompt templates for common clipboard workflows
4. Sits above the authoritative daemon service (Feature 012) for consistent policy

## Goals

- Codex compatibility: stdio and Streamable HTTP transport
- OpenCode compatibility: local and remote MCP servers
- GitHub Copilot CLI compatibility
- Minimal, high-value tool surface (not every DB query becomes a tool)
- Sensitive content masking by default
- Human-in-the-loop confirmation for destructive operations

## Non-Goals

- Implementing full MCP SDK from scratch (use reference implementation)
- Supporting every possible clipboard query as a tool
- Remote HTTP server with authentication (Phase 13)
- Multi-daemon clustering

## Stakeholders

- Developers using Codex/OpenCode for code generation
- Users who want AI agents to access clipboard context
- Power users with extensive snippet collections

---

**Created**: Phase 15 (Post-Research)
**Status**: Draft