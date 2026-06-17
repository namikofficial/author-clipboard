# Requirements: Snippet Token Replacement & Preview

> Variable substitution syntax, built-in variables, preview behaviour,
> and IPC contract.

---

## User Stories

### US-001: Use a built-in date variable
**As a** user
**I want** to write a snippet `Today's date is ${date}.`
**So that** I always paste today's date when I use the snippet

**Acceptance Criteria**:
- Given a snippet `Today's date is ${date}.`, when the user expands it, then the rendered output is `Today's date is 2026-06-17.` (or current date).
- Given a snippet with `${date}`, when the user opens the picker, then the preview shows the same expanded text.
- Given a snippet with `${date}` at midnight UTC, when the user expands at 23:59:59 and again at 00:00:01, then the two expansions may show different dates (the renderer uses the current wall clock).

### US-002: UUID for one-shot tokens
**As a** user
**I want** to write a snippet `request-${uuid}.txt`
**So that** each paste gets a fresh UUID

**Acceptance Criteria**:
- Two consecutive expansions of `request-${uuid}.txt` produce two distinct UUIDs.
- The UUID matches the canonical 8-4-4-4-12 hex format.

### US-003: Cursor marker
**As a** user
**I want** to write a snippet `Hello, ${cursor}world!`
**So that** after paste the caret lands between the comma and "world"

**Acceptance Criteria**:
- The rendered output is `Hello, world!` (the `${cursor}` contributes zero bytes).
- The IPC response includes a `cursor_offset` of `7` (the byte position after `Hello, `).
- A snippet without `${cursor}` returns `cursor_offset: null`.

### US-004: Unknown variable is preserved literally
**As a** user
**I want** to write a snippet `Hi ${name}, welcome!` and have `${name}` survive
**So that** the receiver can manually fill in `name` if needed

**Acceptance Criteria**:
- Expanding `Hi ${name}, welcome!` returns the string `Hi ${name}, welcome!` unchanged.
- The picker preview also shows `Hi ${name}, welcome!` unchanged.
- No warning is logged for unknown variables in v1 (kept quiet to avoid log spam).

### US-005: Escape sequences
**As a** user
**I want** to write `$${not_a_var}` and get `${not_a_var}` literally
**So that** I can include dollar signs in my snippets without surprises

**Acceptance Criteria**:
- `$${name}` expands to `${name}` (the `$$` becomes `$`, and `{name}` is preserved as literal because it's not `${name}`).
- A single `$` not followed by `{` is preserved literally.

### US-006: Rendered preview in picker
**As a** user
**I want** the snippet picker to show me the rendered text in the preview column
**So that** I can tell at a glance what each snippet will paste

**Acceptance Criteria**:
- For `Today's date is ${date}.`, the preview line shows `Today's date is 2026-06-17.` (or current date).
- For a static snippet `git status`, the preview line shows `git status`.
- The preview is read-only and updates when the snippet content changes.

### US-007: CLI expand
**As a** user
**I want** to run `author-clipboard-ctl expand-snippet my-snippet`
**So that** I can render a snippet without opening the GUI

**Acceptance Criteria**:
- Running `expand-snippet my-snippet` writes the rendered text to the clipboard and prints it to stdout.
- Running `expand-snippet --stdout my-snippet` writes only to stdout.
- Running `expand-snippet --cursor-offset my-snippet` prints `text<TAB>offset` (machine-friendly).
- Running `expand-snippet nonexistent` exits non-zero with a clear error.

---

## Functional Requirements

| ID | Requirement | Priority | Notes |
|----|-------------|----------|-------|
| FR-001 | `${name}` substitution with closed set of built-in vars | Must | See variable table below. |
| FR-002 | Escape `$$` → `$` and `\$` → `$` | Must | Backslash form is the shell-friendly escape. |
| FR-003 | Unknown `${name}` preserved verbatim | Must | Don't crash, don't log. |
| FR-004 | Cursor marker `${cursor}` returns zero-length insertion + `cursor_offset` | Must | |
| FR-005 | `RenderSnippet { id }` IPC returns `{ content, cursor_offset }` | Must | |
| FR-006 | Picker preview row shows rendered text | Must | Live; refreshes on snippet content change. |
| FR-007 | `author-clipboard-ctl expand-snippet` CLI | Must | Three modes: copy + print, stdout only, cursor offset. |
| FR-008 | Built-in variables deterministic for fixed `now` | Should | `render_with_now(input, now)` for tests. |
| FR-009 | Renderer never executes anything | Must | No shell calls, no `eval`. Documented. |
| FR-010 | Built-in variables list documented in user docs | Should | `docs/FEATURES.md` (if exists) or inline in `template.rs` rustdoc. |

---

## Built-in Variables

| Name | Type | Example |
|---|---|---|
| `${date}` | ISO date `YYYY-MM-DD` (local TZ) | `2026-06-17` |
| `${time}` | ISO time `HH:MM:SS` (local TZ) | `14:23:08` |
| `${datetime}` | `${date} ${time}` | `2026-06-17 14:23:08` |
| `${iso_date}` | RFC 3339 date `YYYY-MM-DD` (UTC) | `2026-06-17` |
| `${iso_time}` | RFC 3339 time `HH:MM:SSZ` (UTC) | `11:23:08Z` |
| `${iso_datetime}` | RFC 3339 full timestamp | `2026-06-17T11:23:08Z` |
| `${year}`, `${month}`, `${day}` | numeric (zero-padded) | `2026`, `06`, `17` |
| `${hour}`, `${minute}`, `${second}` | 24h clock, zero-padded | `14`, `23`, `08` |
| `${unix}` | Unix epoch seconds | `1718619788` |
| `${uuid}` | Fresh UUID v4 hex | `f47ac10b-58cc-4372-a567-0e02b2c3d479` |
| `${random:N}` | N-char alphanumeric (N in 1..=128) | `${random:8}` → `aZ39kPq1` |
| `${cursor}` | empty string + cursor offset | (zero-length marker) |
| `${clipboard}` | current clipboard text | whatever the daemon has |
| `${user}` | `$USER` env var | `namik` |
| `${hostname}` | hostname | `laptop` |

---

## Non-Functional Requirements

| ID | Requirement | Target | Notes |
|----|-------------|--------|-------|
| NFR-001 | Render latency per snippet | < 100 µs typical | Regex-based parser, no recursion. |
| NFR-002 | Memory per render | O(input length) | No AST allocation in v1. |
| NFR-003 | Test coverage for parser | ≥ 15 tests | Cover all variables + escape + edge cases. |
| NFR-004 | No new clippy warnings under `-D warnings` | 0 warnings | Project rule. |

---

## Edge Cases

| Case | Handling |
|---|---|
| Unclosed `${` | Preserve literally. |
| `${}` (empty name) | Preserve literally. |
| `${cursor}${cursor}` (two markers) | Last one wins for `cursor_offset`. |
| Unknown variable | Preserved verbatim (US-004). |
| `${random:0}` | Treat as `${random:1}` (clamped to 1). |
| `${random:999}` | Clamp to 128. |
| Recursion / nested `${${x}}` | Not supported; literal. |
| `$` followed by space | Literal `$`. |
| `$` at end of input | Literal `$`. |
| `${user}` with no `USER` env var | Empty string. |
| `${clipboard}` with no clipboard / very large clipboard | First 1 KiB; longer content truncates with `…` to keep IPC payload bounded. |

---

## Out of Scope

- Interactive prompts for missing variables.
- User-defined per-snippet variables (storage layer).
- Shell command substitution (security-sensitive).
- Snippet composition / includes / partials.
- Smart-paste that consumes the cursor offset (separate feature).
- Conditional logic (`${x?default}`, `${x|default}`).

---

**Last Updated**: Phase 15 completion (June 2026)
