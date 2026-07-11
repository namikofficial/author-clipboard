 Feature Brief: Wayland Clipboard Command Center

> Turn Author Clipboard from a capable clipboard-history utility into a
> private, keyboard-first clipboard command center for Wayland power users.

## Product Statement

**Author Clipboard** is a local-first clipboard command center for Hyprland,
COSMIC, Sway, and wlroots users. It lets users search copied content, recognize
what each item is, paste or transform it, protect secrets, manage reusable
snippets, and expose clipboard context to local AI tools through a safe MCP
boundary.

The product should feel closer to a fast launcher and developer workbench than
a traditional clipboard history list.

## Positioning

**Tagline**

> The private clipboard command center for Wayland.

**One-line pitch**

> Search, paste, transform, protect secrets, manage snippets, and let local AI
> safely use your clipboard — all on your machine.

**Long pitch**

Author Clipboard captures useful clipboard history locally, classifies copied
content into recognizable cards, protects sensitive values by default, and gives
keyboard-first users immediate actions such as paste, copy as plain text, copy
as Markdown link, format JSON, create snippet, pin, delete, reveal safely, or
send redacted context to an MCP-compatible local AI client.

## Why This Feature Exists

The project already has a strong foundation:

- Wayland clipboard capture.
- Local SQLite storage with FTS-style search.
- Support for text, HTML, images, and file URI lists.
- Sensitive-content detection.
- Optional encryption for sensitive items.
- Incognito mode.
- Snippets.
- External menu picker support.
- COSMIC/libadwaita and GTK4 UI work.
- Hyprland/wlroots picker path.
- CLI control surface.
- MCP server.
- Packaging and install work.

The product risk is not lack of raw capability. The risk is that the current
experience can still look and behave like a generic utility: a search field,
filter chips, rows, and settings. Users should immediately understand why this
is worth installing, starring, and keeping bound to `Super+V`.

This feature gives the product a sharper identity, finishes the shared GTK4
state foundation, and adds the smallest set of signature workflows that make
Author Clipboard feel unique.

## Target Users

### Primary

- Hyprland, COSMIC, Sway, and wlroots users.
- Developers who copy code, commands, stack traces, logs, JSON, URLs, secrets,
  and file paths many times per day.
- Keyboard-first Linux users who prefer launcher-style workflows.
- Privacy-sensitive users who want local-only clipboard history.

### Secondary

- Power users who want snippets, transformations, and quick-paste actions.
- Users of local AI tools who want safe clipboard search through MCP.
- Linux rice/customization users who want Waybar/Wayle integration and a
  polished desktop component.

## Product Pillars

### 1. Fast

The popup opens quickly, search feels instant, rows are model-backed rather
than rebuilt wholesale, and common actions are one keystroke away.

### 2. Recognizable

Every item tells the user what it is: URL, code, JSON, color, image, file,
plain text, HTML, snippet, or secret. Rows show compact previews; the manager
shows richer previews.

### 3. Private

Sensitive values are redacted, encrypted by default for new installs, excluded
from unsafe search/index/log/MCP output, and reveal only through explicit local
actions.

### 4. Actionable

The selected result exposes relevant actions: copy, quick paste, transform,
copy as Markdown, format JSON, create snippet, pin, star, delete, open file,
open URL, reveal secret, or copy redacted content.

### 5. Local AI Ready

The MCP server is not an afterthought. It is a safe local interface that lets
AI tools search recent clipboard context without silently leaking secrets.

### 6. Installable

A good product is not complete until users can install it, configure it, verify
it, and understand it from the README in under one minute.

## Signature Capabilities

The release should be demoed around these moments:

1. Press `Super+V` and see a polished command-center popup.
2. Type `json` and find a previous API payload instantly.
3. Select a row and see actions: copy, paste, pretty JSON, create snippet.
4. Copy a token and see it saved as a redacted secret card, not raw text.
5. Reveal a secret for five seconds, then watch it auto-redact.
6. Copy a URL and see a URL card with normalized domain.
7. Copy a color and see a swatch plus canonical HEX.
8. Copy code and see a compact code preview.
9. Use snippets with `{date}`, `{time}`, `{clipboard}`, and `{selection}`.
10. Ask an MCP client to search clipboard history and receive redacted-safe
    results by default.

## Desired User Experience

### First Launch

The first launch should not drop the user into a blank utility window with no
context. It should guide the user through:

- Daemon status.
- Clipboard capture status.
- Shortcut recommendation.
- Privacy defaults.
- Supported compositor detection.
- Optional Hyprland config snippet.
- Where history is stored.
- How to open the popup.

### Popup

The popup is the main daily surface. It should feel like a launcher:

- Search-first.
- Keyboard-first.
- Compact status pills.
- Grouped results when browsing.
- Chronological relevance when searching.
- Rich but compact rows.
- Visible selected-row action hints.
- Strong empty states.
- No direct database reads from page widgets.

### Manager

The manager is the workspace:

- Sidebar navigation.
- Preview pane.
- History browser.
- Pinned items.
- Secret cards.
- Snippets.
- Rules.
- Diagnostics.
- Settings.
- MCP status/setup.
- Import/export where appropriate.

### CLI

The CLI is for setup, automation, and diagnostics:

- `author-clipboard-ctl doctor`
- `author-clipboard-ctl doctor --fix`
- `author-clipboard-ctl hyprland-config`
- `author-clipboard-ctl hyprland-config --write <path>`
- `author-clipboard-ctl status --json`
- `author-clipboard-ctl transform <id> --as json-pretty`
- `author-clipboard-ctl ignore-next-copy`
- Existing history/copy/clear/export/import commands remain compatible.

### MCP

The MCP server exposes useful local clipboard context while enforcing the same
privacy rules as the UI and CLI. The default MCP behavior should never return
raw sensitive content.

## In Scope

### Foundation

- Authoritative UI item model.
- ID-correct selection.
- Removal of fixed-delay refresh as the primary sync mechanism.
- Model-backed or reuse-backed list rendering.
- Shared privacy policy used by UI, CLI, daemon, and MCP.

### UI/UX

- Launcher-style popup.
- Manager workspace.
- Content-aware result cards.
- Rich preview pane.
- Grouped results.
- Action bar.
- Shortcut overlay.
- Better empty states.
- Responsive sidebar and layout polish.

### Content Intelligence

- Deterministic local classification for:
  - URL.
  - Color.
  - JSON.
  - Code-like text.
  - Image.
  - File URI.
  - HTML/rich text.
  - Plain text.
  - Secret/sensitive item.
- No network lookups for core classification.
- Safe fallback for malformed or oversized content.

### Privacy

- Sensitive encryption default for new profiles.
- Migration preserving explicit existing user choices.
- Redacted UI/CLI/MCP previews.
- Reveal timeout.
- Per-application capture rules.
- Ignore-next-copy.
- Safe logging.

### Developer Workflows

- Transformations:
  - Plain text.
  - Markdown link.
  - Fenced code.
  - Quote.
  - JSON pretty.
  - JSON minified.
  - Redacted copy.
- Snippet variables:
  - `{date}`
  - `{time}`
  - `{clipboard}`
  - `{selection}`
- Snippet escaping rules.

### Adoption

- README rewrite.
- Screenshots and short demo.
- Install truthfulness.
- Doctor command.
- Hyprland setup generator.
- MCP documentation.
- Release validation checklist.

## Out of Scope

These are valuable, but not required for this feature:

- Cloud sync.
- Accounts.
- Remote MCP HTTP/TLS/OAuth server.
- Telemetry.
- AI-generated summaries inside the app.
- Network favicon/title scraping.
- OCR.
- Full plugin/scripting runtime.
- Cross-device encrypted sync.
- X11 parity.
- Browser-extension integration.
- Mobile companion app.
- Theming marketplace.
- CopyQ-compatible script system.
- Flatpak/Flathub promise before sandbox limitations are validated.

## Success Criteria

The feature is successful when:

- The popup opens and displays usable results quickly with 1,000 local entries.
- Popup and manager show the same authoritative item model.
- Selecting by ID never maps to a placeholder row.
- Rich cards exist for the first supported content set.
- Secrets are private by default in UI, CLI, logs, and MCP.
- A user can install/configure on Hyprland from docs without guessing.
- The README communicates the value proposition in the first screen.
- MCP setup is documented with safe examples.
- All P0/P1 tests and manual smoke checks pass.
- Screenshots/demo show the real UI, not mockups.

## Risk Register

| Risk | Impact | Mitigation |
|---|---:|---|
| UI state/data rewrite causes regressions | High | Land as focused foundation tasks before visual work. |
| GTK model/list factory complexity slows delivery | Medium | Benchmark model approach; keep a tested row-reuse fallback if needed. |
| Sensitive encryption migration breaks existing users | High | Versioned migration, fixture tests, explicit decision doc. |
| MCP safety becomes inconsistent with UI | High | Centralize privacy policy in shared crate. |
| Feature creep delays release | High | Split P0/P1/P2/P3 and do not ship P3 under this feature. |
| README overpromises packaging | Medium | Document only verified release paths. |
| Rich preview parsing becomes slow | Medium | Bounded parsers, size limits, no network, measured tests. |

## Dependencies

- Existing feature-023 GTK4 UI foundation.
- Existing daemon/IPC command surface.
- Existing sensitive detection and encryption modules.
- Existing snippets and MCP server.
- Existing packaging/release scripts where available.
- Current GTK4/libadwaita dependency versions.

## Required Decisions

Create or update `09-decisions.md` when any of these are resolved:

1. Whether GTK list rendering uses `ListView`/factory or a row-reuse fallback.
2. Exact daemon-to-UI refresh protocol.
3. Encryption migration behavior for existing config files without an explicit
   `encrypt_sensitive` key.
4. Rule precedence for app capture rules.
5. MCP confirmation shape.
6. Whether any preview feature needs a new dependency.
7. Which package formats are release-blocking versus documentation-only.

## Delivery Principle

Do not build visual polish on top of broken state. The order is:

1. Make data authoritative.
2. Make privacy centralized.
3. Make rows recognizable.
4. Make actions obvious.
5. Make install/demo excellent.

**Created**: 2026-07-11  
**Updated**: 2026-07-12  
**Status**: Proposed
