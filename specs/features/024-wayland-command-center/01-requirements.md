# Requirements: Wayland Clipboard Command Center

## Requirement Levels

| Level | Meaning |
|---|---|
| P0 | Must be completed before any command-center release is considered valid. |
| P1 | Must be completed for the first polished public release of this feature. |
| P2 | Strong follow-up for developer usefulness and adoption. |
| P3 | Later expansion; explicitly not release-blocking. |

## Personas

### Keyboard Developer

Uses Hyprland/COSMIC/Sway, copies commands, stack traces, JSON, URLs, code, and
secrets. Wants `Super+V` to be faster than opening old terminals or browser
tabs.

### Privacy-Sensitive User

Wants clipboard history, but does not want tokens, passwords, SSH keys, API
keys, URLs with credentials, or private notes exposed in UI, logs, export, or
AI tools.

### Local AI User

Uses Codex/OpenCode/Claude-like tools with MCP. Wants useful local context
without handing the model every raw clipboard secret.

### Desktop Customizer

Cares about good visuals, Waybar/Wayle integration, native Wayland behavior,
and a clean install/config story.

## Product Requirements

### PR-001: Product Identity

**Priority**: P0

Author Clipboard must present itself as a private Wayland clipboard command
center, not a generic clipboard manager.

**Acceptance criteria**:

- README starts with the command-center value proposition.
- The main UI title uses `Author Clipboard`, not generic `Clipboard Manager`.
- Documentation explains the five core user values:
  - search copied content,
  - act on selected content,
  - protect secrets,
  - manage snippets,
  - expose safe local context to MCP.
- Development-branch warnings are accurate but do not bury the product value
  proposition.

### PR-002: First-Run Clarity

**Priority**: P1

A new user must understand whether the app is working and how to invoke it.

**Acceptance criteria**:

- The manager shows daemon/socket/config/compositor status when history is
  empty or unavailable.
- The app explains the recommended shortcut and offers a CLI command to print
  compositor config.
- The UI distinguishes:
  - no history,
  - daemon unavailable,
  - search returned no results,
  - capture paused/incognito,
  - filters hiding results.
- Empty states include a next action.

### PR-003: Install Truthfulness

**Priority**: P0

Public docs must not claim packages or stores that are not actually available.

**Acceptance criteria**:

- README separates verified install paths from planned package formats.
- Release badges and package instructions match actual release artifacts.
- If a package path is experimental, the documentation says so.
- `just verify` or release validation checks documentation commands where
  practical.

## Functional Requirements

### US-001: Authoritative Clipboard Model

**Priority**: P0

**As** a keyboard user, **I want** popup and manager results to come from one
authoritative model, **so that** I can trust actions apply to what I see.

**Acceptance criteria**:

- GTK pages do not open the database directly for result rendering.
- Initial load goes through shared IPC/controller logic.
- Refresh goes through shared IPC/controller logic.
- `AppState.items` or its replacement is the source of truth for visible
  result actions.
- Popup and manager use the same item view model.
- Tests prove page-local database access is absent from GTK result pages.

### US-002: Correct Selection by ID

**Priority**: P0

**As** a user, **I want** actions to target the selected item, **so that** copy,
delete, pin, transform, and reveal never affect the wrong row.

**Acceptance criteria**:

- Selecting by database ID resolves the matching item.
- Unknown IDs do not silently map to index `0`.
- Selected ID survives refresh if the item remains visible.
- Selection falls back to a deterministic adjacent row if the selected item is
  removed.
- Tests cover:
  - select existing ID,
  - select unknown ID,
  - delete selected item,
  - refresh with selected item still present,
  - refresh with selected item gone.

### US-003: Explicit Refresh Signaling

**Priority**: P0

**As** a user, **I want** new captures and mutations to appear reliably,
**so that** I do not need arbitrary delay hacks.

**Acceptance criteria**:

- Fixed one-shot timeout refresh is not the primary synchronization mechanism.
- Opening a popup triggers one explicit load.
- Daemon capture, delete, pin, star, snippet update, settings change, and clear
  operations trigger a model refresh or change event.
- Daemon unavailable state is rendered with retry.
- The refresh protocol is versioned or additive so existing IPC clients do not
  break.

### US-004: Scalable Result Rendering

**Priority**: P0

**As** a power user, **I want** history with many items to remain responsive,
**so that** the popup does not feel heavy.

**Acceptance criteria**:

- Refreshing 1,000 entries does not remove/recreate every visible GTK child.
- The implementation uses a GTK model/list factory where practical, or a tested
  row-reuse fallback where factory binding is not stable.
- A benchmark or synthetic test records observed load/render behavior.
- First usable result target: under 150 ms for 1,000 local entries on supported
  hardware, measured before claiming success.
- Focus and selected ID survive refresh.

### US-005: Content-Aware Classification

**Priority**: P1

**As** a user, **I want** each item to identify its content type, **so that** I
can choose the right item quickly.

**Acceptance criteria**:

- Shared code produces a `ContentPresentation` or equivalent pure view model.
- Supported kinds:
  - text,
  - URL,
  - color,
  - JSON,
  - code-like text,
  - HTML,
  - image,
  - file URI,
  - secret,
  - unknown fallback.
- Sensitive classification runs before every other classifier.
- Classification has size limits and safe failure behavior.
- No network requests are used for classification.

### US-006: Recognizable Result Cards

**Priority**: P1

**As** a user, **I want** compact cards that make copied content recognizable,
**so that** I do not have to open preview for every row.

**Acceptance criteria**:

- Every row shows:
  - content type badge,
  - safe preview,
  - relative time,
  - pinned/starred/sensitive state where applicable,
  - source app when available,
  - selected-row action hints.
- URLs show normalized domain.
- Colors show a swatch and canonical HEX.
- JSON/code show a compact formatted preview.
- Images show thumbnail or metadata fallback.
- Files show filename/path hint and MIME/icon where available.
- Secrets show redacted preview and secret kind, never raw value.

### US-007: Rich Manager Preview

**Priority**: P1

**As** a user, **I want** a richer preview in the manager, **so that** I can
inspect an item before acting.

**Acceptance criteria**:

- Preview pane renders the selected item from the same authoritative model.
- Preview supports:
  - text,
  - URL,
  - color,
  - JSON/code,
  - image,
  - file,
  - secret,
  - unknown fallback.
- Sensitive preview is redacted by default.
- Reveal action starts a visible countdown and auto-redacts.
- Preview pane does not stretch images beyond reasonable bounds.
- Preview never crashes on malformed content.

### US-008: Action-Oriented Popup

**Priority**: P1

**As** a frequent user, **I want** visible actions for the selected item,
**so that** common work takes one keystroke.

**Acceptance criteria**:

- Popup shows compact status pills for daemon, incognito, and privacy state.
- Selected item exposes context-valid actions:
  - copy,
  - quick paste,
  - copy plain text,
  - pin/unpin,
  - star/unstar,
  - delete,
  - create snippet,
  - transform,
  - open URL/file when safe,
  - reveal/copy redacted for secrets.
- Unavailable actions are hidden or disabled with accessible explanation.
- Existing keyboard semantics remain:
  - `Enter` copy/restore,
  - `Ctrl+Enter` quick paste,
  - arrows move,
  - `/` search,
  - `Esc` clear/close,
  - `?` shortcuts.
- Mouse actions have keyboard equivalents.

### US-009: Grouped Browsing and Search Behavior

**Priority**: P1

**As** a user, **I want** browsing to be organized but searching to stay
relevant, **so that** the UI feels predictable.

**Acceptance criteria**:

- Empty query view groups results by useful sections:
  - pinned,
  - recent,
  - today,
  - images,
  - files,
  - links,
  - code,
  - secrets.
- Groups render only when non-empty.
- Search query view prioritizes search relevance/chronology and does not
  over-group results in a way that hides matches.
- Group headers are keyboard-skippable and screen-reader labelled.
- Filters and grouping compose predictably.

### US-010: Private-by-Default Sensitive Handling

**Priority**: P0

**As** a developer, **I want** copied secrets protected by default, **so that**
clipboard history does not leak credentials.

**Acceptance criteria**:

- New installations default `encrypt_sensitive` to enabled.
- Existing users with explicit settings retain their settings.
- Migration behavior is documented and tested.
- Sensitive content is excluded from raw-content search/index output.
- UI, CLI, logs, export, and MCP default to redacted previews.
- Reveal requires explicit local action.
- Reveal auto-redacts after five seconds.
- Copying full sensitive content through MCP requires explicit confirmation.

### US-011: Application Capture Rules

**Priority**: P2

**As** a user, **I want** per-app capture rules, **so that** I can avoid storing
content from password managers, private browsers, or specific apps.

**Acceptance criteria**:

- Rules support at least:
  - ignore capture,
  - force redact,
  - tag,
  - max TTL override.
- Match fields support source app/window metadata where available.
- Rule precedence is documented and tested.
- Rules can be evaluated as pure data without a compositor.
- Settings UI can list/add/disable/delete rules.
- A reset path exists for broken rules.

### US-012: Ignore Next Copy

**Priority**: P2

**As** a user, **I want** to ignore exactly one upcoming copy, **so that** I can
copy temporary sensitive content without storing it.

**Acceptance criteria**:

- CLI and UI can arm ignore-next-copy.
- The next eligible capture is skipped exactly once.
- User feedback distinguishes armed, consumed, and expired states.
- Ignore-next-copy state does not survive longer than intended.
- Tests cover concurrent/rapid copy behavior as far as practical.

### US-013: Transformations

**Priority**: P2

**As** a developer, **I want** common transformations, **so that** repeated
formatting work is instant.

**Acceptance criteria**:

- Supported transforms:
  - plain text,
  - Markdown link,
  - fenced code,
  - quote,
  - JSON pretty,
  - JSON minified,
  - copy redacted.
- Invalid transforms return non-sensitive errors and leave original content
  unchanged.
- Transformations are pure functions in shared code.
- UI, CLI, and MCP use the same transform implementation.
- Transformations never bypass sensitive confirmation policy.

### US-014: Snippet Variables

**Priority**: P2

**As** a user, **I want** snippets with variables, **so that** common messages
and templates can adapt to context.

**Acceptance criteria**:

- Supported variables:
  - `{date}`,
  - `{time}`,
  - `{clipboard}`,
  - `{selection}`.
- Escaping is documented and tested.
- Sensitive clipboard/selection values require confirmation before expansion.
- Snippet preview shows what will be inserted where safe.
- Snippet errors are actionable and non-sensitive.

### US-015: Safe MCP Search and Resources

**Priority**: P1

**As** a local AI user, **I want** MCP search/resources to be useful without
silent leaks, **so that** automation is safe.

**Acceptance criteria**:

- MCP search returns redacted previews for sensitive items by default.
- MCP resources for recent/pins/snippets/stats enforce shared privacy policy.
- Full sensitive get/copy requires explicit per-request confirmation.
- Destructive operations require explicit confirmation.
- No broad persistent "always allow secrets" flag is introduced.
- Error responses are machine-readable and actionable.
- Tests prove raw sensitive content is absent from default JSON responses.

### US-016: MCP Product Documentation

**Priority**: P1

**As** a user, **I want** MCP setup examples, **so that** I can connect local
tools correctly.

**Acceptance criteria**:

- Docs include stdio setup for Codex and one other verified MCP client.
- Docs include safe example prompts.
- Docs state exactly what clipboard data an MCP client can access.
- Docs explain confirmation parameters and refusal behavior.
- Docs state that the architecture is local-only by default.

### US-017: Doctor Command

**Priority**: P2

**As** a user, **I want** a diagnostic command, **so that** setup issues are
obvious.

**Acceptance criteria**:

- `author-clipboard-ctl doctor` checks:
  - daemon process/reachability,
  - IPC socket path,
  - config path,
  - database path,
  - storage permissions,
  - compositor/session type,
  - wl-copy/wtype/ydotool availability,
  - picker dependencies,
  - UI dependencies where practical.
- Output has human and JSON modes.
- `--fix` only mutates safe user-owned Author Clipboard paths.
- Compositor dotfile changes are never performed without explicit path and
  confirmation.

### US-018: Safe Hyprland Config Generator

**Priority**: P2

**As** a Hyprland user, **I want** generated config, **so that** setup is fast
and idempotent.

**Acceptance criteria**:

- CLI prints recommended bind snippets.
- `--write <path>` uses a managed block with begin/end comments.
- Existing managed block is updated idempotently.
- Existing unrelated config is preserved.
- A backup or dry-run is available before writes.
- Command clearly states what it changed.

### US-019: Manager Workspace

**Priority**: P1

**As** a user, **I want** the full manager to feel like a workspace, **so that**
I can manage history, snippets, secrets, rules, and setup.

**Acceptance criteria**:

- Sidebar contains clear product sections:
  - Home,
  - History,
  - Pinned,
  - Secrets,
  - Images,
  - Links,
  - Code,
  - Files,
  - Snippets,
  - Rules,
  - MCP,
  - Settings.
- Sections may be implemented progressively, but hidden/incomplete sections are
  not shown as broken pages.
- Home page summarizes status and quick actions.
- Manager window persists size and last page.
- Responsive layout works at narrow widths.

### US-020: Import, Export, and Data Safety

**Priority**: P2

**As** a user, **I want** import/export to be safe, **so that** backup does not
leak secrets accidentally.

**Acceptance criteria**:

- Export has modes:
  - redacted default,
  - full export with explicit confirmation,
  - snippets-only,
  - settings-only.
- Sensitive fields are redacted by default.
- Import re-runs sensitive detection.
- Import preview shows counts and warnings.
- Existing data is not overwritten without confirmation.

### US-021: Accessibility and Keyboard Parity

**Priority**: P1

**As** a keyboard or assistive-tech user, **I want** every action to be labelled
and reachable, **so that** the app is usable without a mouse.

**Acceptance criteria**:

- Every icon-only action has accessible label and tooltip.
- Every keyboard action has a visible or discoverable UI equivalent.
- Focus ring is consistent across rows, chips, buttons, search, and actions.
- Group headers and status pills are screen-reader labelled.
- `?` opens a complete shortcuts overlay.
- Esc behavior remains deterministic.

### US-022: Visual Quality

**Priority**: P1

**As** a user, **I want** the UI to feel modern and native, **so that** it is
pleasant enough to keep installed.

**Acceptance criteria**:

- Popup uses a command-center shell, not a plain stacked utility layout.
- Rows have consistent spacing, radius, type badges, hover, selected, and
  focus states.
- Dark and light themes are both usable.
- Sensitive cards have distinct visual treatment.
- Status indicators are real widgets/classes, not ASCII glyph hacks.
- Visual snapshots are updated after major UI changes.

### US-023: Release Demo

**Priority**: P1

**As** a prospective user, **I want** to see the product quickly, **so that** I
can decide whether to install it.

**Acceptance criteria**:

- README includes a short demo asset or clear screenshot sequence.
- Screenshots cover:
  - popup,
  - manager,
  - secret card,
  - rich URL/color/code/JSON/image/file previews,
  - snippets,
  - MCP safety.
- Demo content uses fake data only.
- Screenshots are generated from real app surfaces, not design mockups.

### US-024: Compatibility

**Priority**: P0

**As** an existing user, **I want** upgrades to preserve data and workflows,
**so that** new UI work does not break my setup.

**Acceptance criteria**:

- Existing DB migrations have tests.
- Existing config files load correctly.
- Existing CLI commands continue to work or provide clear migration errors.
- Additive IPC fields use serde defaults.
- Existing systemd user service remains valid unless explicitly migrated.
- Release notes document breaking changes.

## Non-Functional Requirements

| ID | Requirement | Target |
|---|---|---|
| NFR-001 | Popup responsiveness | First usable result under 150 ms for 1,000 local entries, measured before claim |
| NFR-002 | Search responsiveness | Query update feels interactive with debounced search and no UI freeze |
| NFR-003 | Local-only classification | No network traffic for classification, transforms, snippets, previews, or privacy policy |
| NFR-004 | Privacy | Raw sensitive values do not appear in default UI/CLI/MCP/log/export paths |
| NFR-005 | Accessibility | Keyboard and screen-reader accessible equivalents for every action |
| NFR-006 | Compatibility | Existing DB/config/IPC clients continue or have documented migration |
| NFR-007 | Testability | Core classification, transforms, rules, privacy, and selection are pure/testable |
| NFR-008 | Reliability | Daemon unavailable state does not crash UI; user receives retry path |
| NFR-009 | Theming | Light/dark themes remain coherent through libadwaita tokens |
| NFR-010 | Verification | Every delivered task has focused tests plus `just verify` unless documented |
| NFR-011 | Security logging | Logs never include raw sensitive content |
| NFR-012 | Data ownership | User data stays in local config/data paths unless explicitly exported |

## Priority Matrix

| Priority | Scope |
|---|---|
| P0 | Authoritative UI model, correct selection, refresh sync, scalable rendering, private sensitive defaults, compatibility, install truthfulness |
| P1 | Content classification, rich cards, action popup, manager workspace, MCP safety, accessibility, visual quality, demo |
| P2 | App rules, ignore-next-copy, transforms, snippet variables, doctor, Hyprland generator, safe import/export |
| P3 | OCR, favicons/title fetching, usage ranking, web metadata, full plugin runtime, cross-device sync, additional package stores |

## Security and Privacy Requirements

- Default behavior must prefer redaction over convenience.
- Sensitive detection must run on imported data as well as captured data.
- Redacted previews must be derived without storing raw previews in unsafe
  fields.
- Any API returning raw sensitive content must require explicit confirmation.
- Confirmation is request-scoped and not stored globally.
- Destructive operations require explicit confirmation in MCP and CLI where
  appropriate.
- Error messages must not echo sensitive content.
- Debug logs must use IDs, types, hashes, and safe previews only.

## UX Guardrails

- Do not show broken placeholder pages.
- Do not expose settings that do nothing.
- Do not show a feature in README until it is implemented or clearly marked as
  planned.
- Do not make users edit dotfiles blindly.
- Do not make search feel slower because of decorative grouping.
- Do not make secrets harder to protect than normal text is to copy.
- Do not add network-based enrichments in this feature.

## Out of Scope for This Feature

- Remote/cloud sync.
- AI-generated summaries inside the app.
- Remote MCP transport.
- Automatic browser metadata fetching.
- OCR.
- Marketplace/plugin runtime.
- X11 parity.
- Browser extension.
- Mobile companion.
- Telemetry or analytics.
- User accounts.

## Done Definition

A task is done only when:

- Implementation matches the acceptance criteria for its scoped stories.
- Focused tests pass.
- `just verify` passes or failure is documented as unrelated with exact reason.
- Manual Wayland smoke check is recorded where UI/compositor behavior changes.
- Docs are updated for user-visible behavior.
- Screenshots are updated when UI changes materially.
- No raw sensitive data appears in new logs, tests, fixtures, screenshots, or
  docs.

**Created**: 2026-07-11  
**Updated**: 2026-07-12  
**Status**: Proposed