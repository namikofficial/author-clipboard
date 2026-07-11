# Author-Clipboard Roadmap

> High-level project tracking for author-clipboard development.

> Note: the unified GTK4 UI rewrite tracked in `specs/features/023-unified-gtk4-ui/` is complete on `dev`. This roadmap now tracks the remaining product phases and broader maintenance work.
> Historical spec numbering note: `024-ui-cohesion-polish` is designated
> **024A**, while `024-wayland-command-center` is designated **024B** and maps
> to product roadmap phase 25. Both paths are retained to avoid breaking links.

---

## Vision

A world-class clipboard manager for Linux that captures everything, organizes intelligently, and integrates seamlessly with AI coding workflows.

---

## Phase Status

### Completed Phases

| Phase | Feature | Status | Notes |
|-------|---------|--------|-------|
| 1-11 | Core Features | ✅ Done | Basic clipboard capture, UI, security, Hyprland integration |
| 12 | Service API Normalization | ✅ Done | Daemon as single authority, expanded IPC commands |
| 13 | MCP Protocol | ✅ Done | MCP server with tools, resources, prompts |
| 14 | Advanced Filtering | ✅ Done | Composable filter chips (content_type, pinned, sensitive, source_app, age) |
| 15 | Collections | ✅ Done | Starred field, collections tables, IPC handlers |
| 16 | World-Class UX | ✅ Done | Premium UI/UX specification |
| 17 | Dotfiles Integration | ✅ Done | Hyprland scripts updated for author-clipboard |
| 18 | Dedup Fix | ✅ Done | SHA-256 hashing, dedup window enforcement |
| 19 | Config Cleanup | ✅ Done | content_denylist, ContentPatternMode enum |
| 21 | Collections UI | ✅ Done | Native manager page: create, rename, delete, inspect membership, add/remove |
| 22 | Expression Pickers | ✅ Done | Shared searchable emoji/symbol/kaomoji UI with categories and persistent recents |
| 23 | Rich Content | ✅ Done | Safe HTML fallback, image previews, file metadata, MIME-preserving restore |

### In Progress

| Phase | Feature | Status | Notes |
|-------|---------|--------|-------|
| 20 | MCP Integration Testing | 🔄 In Progress | End-to-end testing with Codex/OpenCode |
| 25 | Wayland Clipboard Command Center | 🔄 In Progress | Core shared/UI/CLI/MCP increments implemented; release and compositor evidence pending |

### Upcoming

| Phase | Feature | Priority | Notes |
|-------|---------|----------|-------|
| 24 | Packaging | High | Systemd unit, AppImage, distribution |

---

## Feature Matrix

| Feature | Status | Confidence | Notes |
|---------|--------|-------------|-------|
| Clipboard capture | ✅ Done | High | Works on COSMIC and Hyprland |
| Content dedup (SHA-256) | ✅ Done | High | Using sha2 crate |
| Search/filter | ✅ Done | High | Full-text search with filters |
| Collections | ✅ Done | High | Storage/IPC plus native manager workflows |
| Expression pickers | ✅ Done | High | Search, categories, clipboard activation, persistent recents |
| Rich content | ✅ Done | High | Safe HTML, images, files, metadata and restore paths |
| Command center | 🔄 In Progress | Medium | Core increments integrated; final verification/manual compositor evidence pending |
| MCP server | ✅ Done | High | stdio transport, tools/resources/prompts |
| Dotfiles integration | ✅ Done | High | Scripts updated in dotfiles repo |
| Pinned items | ✅ Done | High | Pin/unpin via IPC |
| Snippets | ✅ Done | High | CRUD via IPC |
| Quick paste | ✅ Done | High | CopyMode::QuickPaste |
| Security (sensitive) | ✅ Done | High | Masked previews, confirmation required |
| Wayland clipboard | ✅ Done | High | wl-copy integration |
| Hyprland support | ✅ Done | High | hypr-picker binary |

---

## Technical Debt

| Item | Priority | Notes |
|------|----------|-------|
| HTTP transport for MCP | Low | stdio is sufficient for local use |
| TLS for remote MCP | Low | Security consideration for remote |
| OAuth for remote MCP | Low | Phase 13 item not yet implemented |
| Performance optimization | Medium | Measure before optimizing |
| E2E tests | Medium | Integration testing with real clipboard |

---

## Dependencies

- **COSMIC desktop**: Requires `COSMIC_DATA_CONTROL_ENABLED=1` for clipboard monitoring
- **Hyprland/Sway**: Uses `wlr-data-control` protocol (do NOT set COSMIC_DATA_CONTROL_ENABLED)
- **libcosmic**: Fetched from pop-os/libcosmic.git
- **rusqlite**: Uses bundled SQLite

---

## Verification Commands

```bash
just verify        # fmt → lint → test → build (full check)
just build         # build all crates
just check         # quick type check
just test          # run all tests
just fmt           # format code
just lint          # clippy with -D warnings

# Single crate testing
cargo test -p author-clipboard-daemon
cargo test -p author-clipboard-shared
cargo test -p author-clipboard-mcp
```

---

**Last Updated**: Phase 19
