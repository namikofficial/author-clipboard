# Requirements: Native Power-User Revamp

> Functional and non-functional requirements for making author-clipboard a
> competitive Linux-native clipboard manager for developers and power users.

---

## User Stories

### US-001: Native Close And Window Behavior
**As a** Hyprland/Sway/COSMIC user  
**I want** the picker to open, resize, float, close, and focus like a native
utility window  
**So that** it does not feel like a hacked overlay or stuck process

**Acceptance Criteria**:
- Given the picker is open, when I press `Esc`, then the window closes and the
  picker process exits.
- Given the picker is open, when I click the close button, then the window
  closes and the picker process exits.
- Given the picker is launched through the desktop file or keybind on Hyprland,
  then it opens as a centered floating utility window.
- Given the picker is launched with `--layer-shell`, then it uses layer-shell
  overlay mode for users who prefer overlay behavior.
- Given the user resizes the picker, then the size is persisted through
  GSettings and restored on next launch.

### US-002: Rich Command-Center Picker
**As a** power user  
**I want** the picker to show history, filters, actions, and preview context in
one coherent shell  
**So that** I can find, inspect, and act on clipboard items quickly

**Acceptance Criteria**:
- Given an item is selected, then the UI shows a type-aware inspector/preview.
- Given an item row is focused, then actions for copy, quick-paste, pin, star,
  delete, reveal, and add-to-collection are visible or discoverable.
- Given the picker is narrow, then it uses a single-column layout with an
  expandable inspector.
- Given the picker is wide, then it uses a two-pane layout with list on the
  left and inspector on the right.
- Given a row is selected, then the selected state is visually obvious and
  keyboard focus remains clear.

### US-003: Developer-First Search And Filters
**As a** developer  
**I want** search and filters that understand developer content  
**So that** I can find commands, code, paths, URLs, prompts, JSON, SQL, and
tokens without scanning a raw list

**Acceptance Criteria**:
- Given I type `type:code`, `type:url`, `type:path`, `type:image`, `type:file`,
  `type:secret`, or `type:snippet`, then results filter accordingly.
- Given I type `project:<name>` or `app:<name>`, then results filter by stored
  context when available.
- Given I type quoted text, then exact phrase search is applied.
- Given I type a bare query, then FTS search is applied with LIKE fallback.
- Given the query is invalid, then the UI shows a non-blocking hint and falls
  back to plain search instead of failing.

### US-004: Item Inspector
**As a** user  
**I want** a preview pane that understands each content type  
**So that** I can inspect content safely before restoring it

**Acceptance Criteria**:
- Text previews show wrapped text, full length, word count, MIME type, age, and
  source context when available.
- Code-like previews use monospace text, line numbers, and language hints when
  feasible.
- HTML previews show safe plain-text fallback first, with an explicit rendered
  preview option if implemented.
- Images show thumbnail, dimensions, MIME type, and file size.
- File URI lists show file cards with name, path, existence status, and count.
- Sensitive items are redacted by default and require explicit reveal.

### US-005: Organization For Real Work
**As a** developer  
**I want** pins, stars, collections, saved filters, and project boards  
**So that** frequently reused material does not disappear in chronological noise

**Acceptance Criteria**:
- Pin prevents automatic cleanup and shows in pinned views.
- Star boosts ranking without changing retention.
- Collections group items into named boards.
- Saved filters preserve search/filter combinations such as `DB queries`,
  `Deploy commands`, `Prompt fragments`, or `Links`.
- Collection membership survives daemon restart.
- Deleting a collection does not delete the underlying clipboard items.

### US-006: Snippets And Templates As First-Class Content
**As a** power user  
**I want** snippets, variables, and rendered previews integrated into the picker  
**So that** reusable text feels like a native part of the clipboard workflow

**Acceptance Criteria**:
- Snippet entries appear beside history or in a dedicated source tab.
- Rendered snippet preview is visible before expansion.
- Expanding a snippet can copy or quick-paste the rendered output.
- Cursor offsets from templates are preserved in IPC even if not all paste
  backends consume them yet.
- Unknown variables remain literal and safe.

### US-007: Automation And Integrations
**As a** Linux power user  
**I want** stable CLI, IPC, bar, shell, and editor integration points  
**So that** the clipboard manager fits my workstation instead of being a silo

**Acceptance Criteria**:
- Every primary UI action has a CLI/IPC equivalent.
- `author-clipboard-ctl status --json` remains stable for bars and widgets.
- The picker and manager desktop entries are installed and validated.
- Hyprland keybind recommendations include the native picker, overlay mode, and
  external menu fallback.
- JSON export/import can round-trip history, snippets, collections, and saved
  filters without exposing encrypted secrets by default.

### US-008: Privacy And Trust
**As a** privacy-conscious user  
**I want** secret detection, redaction, incognito mode, and auditability to be
visible and predictable  
**So that** I can trust the tool with clipboard data

**Acceptance Criteria**:
- Sensitive rows never show raw content unless explicitly revealed.
- Reveal actions are time-limited and visually marked.
- Incognito mode is visible in the status area.
- Screen-lock clearing behavior is surfaced in settings/status.
- Logs never print raw sensitive clipboard content.

### US-009: First-Run And Health Experience
**As a** new user  
**I want** the app to explain missing runtime dependencies and setup status  
**So that** I can fix issues without reading source code

**Acceptance Criteria**:
- If the daemon is down, the picker shows a clear daemon-down state with a
  start/retry action.
- If `wl-copy`, `wtype`, `ydotool`, `gtk4-layer-shell`, or compositor support is
  missing, `doctor` and the UI explain what is affected.
- If the GSettings schema is missing, install/check commands surface the exact
  fix and `just install` installs it.
- If the app is running in a tiling compositor, docs and install helpers provide
  float/center rules.

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | Native XDG utility picker with optional layer-shell mode | Must | Hypr picker defaults to XDG floating window. |
| FR-002 | Close button + reliable Esc close lifecycle | Must | Window and process both exit. |
| FR-003 | Responsive two-pane picker/manager shell | Must | List + inspector on wide widths. |
| FR-004 | Type-aware inspector previews | Must | Text, code, HTML fallback, image, files, sensitive. |
| FR-005 | Row action rail/menu | Must | Copy, quick-paste, pin, star, delete, collection, reveal. |
| FR-006 | Keyboard shortcut overlay | Must | `?` or F1. |
| FR-007 | Query parser for developer filters | Must | `type:`, `app:`, `project:`, exact phrase. |
| FR-008 | Collections and saved filters | Must | See `015-collections`. |
| FR-009 | Snippet/template integration | Must | See `026-snippet-templates`. |
| FR-010 | Import/export v2 includes org metadata | Should | Redact encrypted/sensitive by default. |
| FR-011 | Bar/widget status contract | Must | Stable JSON. |
| FR-012 | First-run/doctor UI | Should | Show missing dependencies and service status. |
| FR-013 | Install path includes schemas, desktop files, service, icons | Must | `just install`, packages, AUR/Nix/deb. |
| FR-014 | UI screenshots and docs refreshed | Must | Real screenshots after implementation. |

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | Picker cold launch | < 250 ms target | Measured after release build install. |
| NFR-002 | Search update latency | < 100 ms for 5k items | Use indexed DB queries and debouncing. |
| NFR-003 | List interaction | No visible jank at 5k rows | Prefer GTK list model/factory or pagination. |
| NFR-004 | Memory usage | < 150 MB with 5k items loaded | Avoid loading full image/text payloads until preview. |
| NFR-005 | Secret safety | 0 raw sensitive strings in logs/UI by default | Unit and smoke checks. |
| NFR-006 | Accessibility | Keyboard-only complete workflow | Visible focus, shortcuts, labels. |
| NFR-007 | Install reliability | Fresh `just install` produces runnable picker | Includes schema and desktop launcher. |

## Edge Cases

| Case | Handling |
|------|----------|
| Daemon down | UI shows daemon-down state; direct DB read only when safe and explicit. |
| Huge text item | Row truncates; inspector lazily loads full content. |
| Multi-byte Unicode | All truncation is char-safe. |
| Binary/unknown MIME | Show generic binary/file preview and metadata. |
| Large image | Thumbnail in list; scaled preview in inspector. |
| Sensitive encrypted item | Redacted row; preview requires reveal and decryption boundary. |
| Missing source-app metadata | Display "unknown app"; do not fake app names. |
| Invalid query filter | Show hint; fall back to plain text search. |
| Multiple monitors | Open near focused monitor/window when compositor supports it. |
| Collection deleted while item selected | Remove membership; item remains in history. |

## Out of Scope

- SaaS/cloud sync.
- Collaborative collections.
- Shell command execution inside snippets.
- X11 fallback implementation.
- Full plugin marketplace.
- OCR/image text extraction.

## Dependencies

- `015-collections` for collections/star/pin semantics.
- `016-world-class-ux` for preview/performance ideas, superseded by this spec.
- `021-hyprland-wlroots-polish` for compositor packaging/support.
- `024-ui-cohesion-polish` for visual token/style system.
- `026-snippet-templates` for template rendering.

---

**Last Updated**: 2026-06-19
