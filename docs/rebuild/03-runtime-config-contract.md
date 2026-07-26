# Runtime Configuration Contract

## Problem

Several CLI and config values were parsed but never affected runtime
behaviour:
- `--source` only showed a chip label (data source stayed as history)
- `--action` only changed a header badge (copy mode stayed as copy)
- `--include-sensitive` was silently ignored
- `ManagerConfig.initial_page` went unused
- Manager clipboard page used `ClipboardPageProps::default()` and ignored
  all CLI arguments
- `--source` default was hardcoded to `History` instead of reading
  `config.picker.default_source`
- `--count` default was hardcoded to `50` instead of reading
  `config.picker.max_results`
- Popup default dimensions ignored `PickerConfig.width`/`height`
- `--page` only accepted `PickerSource` values (not `home`, `collections`,
  `settings`)

## Solution

All configuration values now flow through a typed pipeline:

```
CLI / config file
  → validated PopupConfig / ManagerConfig
  → ClipboardPageProps (page level)
  → PageState (in-memory page state)
  → correct data source, filter, action, query
```

## Affected Values

| Value | Before | After |
|---|---|---|
| `--source snippets` | Ignored (chip only) | Loads snippets in list |
| `--source emoji` | Ignored | Loads emoji grid/list |
| `--source symbols` | Ignored | Loads symbols grid/list |
| `--source kaomoji` | Ignored | Loads kaomoji grid/list |
| `--source all` | Ignored | Loads history + snippets |
| `--query <text>` | ✅ Works | ✅ Works (unchanged) |
| `--count <n>` | Passed through | Reads config `picker.max_results` as default |
| `--include-sensitive` | Ignored | Passed to data query |
| `--action quick-paste` | Badge only | Uses `CopyMode::QuickPaste` |
| `--action copy` | Badge only | Uses `CopyMode::Copy` |
| `picker.prefer_quick_paste` | Never read | Applied as action default |
| `picker.close_after_copy` | Never read | Popup closes or stays |
| `picker.max_results` | Default 50 | Used as `count` default |
| `picker.default_source` | Never read | Used as `--source` default |
| `--page <name>` (applet) | N/A | Deep-links manager to any page |
| `ManagerConfig.initial_page` | Ignored | Selected in sidebar |
| Manager clipboard page | `ClipboardPageProps::default()` | Wired from CLI/config args |
| Popup initial dimensions | Hardcoded (780,620) | GSettings override (was already configurable) |

## Default Resolution

For each value, CLI explicit wins, then config, then code default:

```
--source <value>         → wins
config.default_source    → fallback (parsed from string)
PickerSource::History    → final fallback

--count <value>          → wins
config.max_results       → fallback
50                       → final fallback

--action <value>         → wins
config.prefer_quick_paste → fallback (QuickPaste if true)
PickerAction::Copy       → final fallback

--page <value>           → wins
--source <value>         → mapped to PageId
config.default_source    → mapped to PageId
GSettings last_page      → saved page
PageId::Clipboard        → final fallback
```

## Typed Config Flow

### Popup path (`applet --mode popup`, `hypr-picker`)

```
Args → PopupConfig → ClipboardPageProps → PageState → load_entries_for()
                                                          ↓
                                              PickerEntry[] → ListBox
```

### Manager path (`applet --mode manager`, `applet --page emoji`)

```
Args ──→ ManagerConfig ──→ build_manager_window()
  │                          │
  ├── initial_page ─────────→ sidebar selection → stack.set_visible_child()
  ├── clipboard_source ─────→ ClipboardPageProps.source
  ├── clipboard_filter ─────→ ClipboardPageProps.initial_filter
  ├── clipboard_query ──────→ ClipboardPageProps.initial_query
  ├── clipboard_action ─────→ ClipboardPageProps.action
  ├── clipboard_count ──────→ ClipboardPageProps.count
  └── clipboard_include_ ───→ ClipboardPageProps.include_sensitive
        sensitive
```

## Test Coverage

All new behaviour is tested at the parser and data-modelling layer:

- `crates/applet/src/main.rs` — 29 tests covering CLI argument parsing
  for source (optional, config fallback, invalid config), filter, query,
  action (optional), count (optional, config fallback), include-sensitive,
  page (all values), mode, round-trips
- `crates/hypr-picker/src/main.rs` — 23 tests: CLI parsing, PopupConfig
  reflection for source, action, query, count, filter, xdg-window,
  include-sensitive
- `crates/shared/src/picker.rs` — 64 tests: source behaviour (emoji,
  symbol, kaomoji entries, snippet entries), filter behaviour (pinned,
  sensitive, all), action→mode mapping, invalid-value rejection,
  expression entry properties, preview rendering, entry-to-item mapping
- `crates/ui-gtk/src/pages/clipboard.rs` — ClipboardPageProps defaults
  (source, action, include_sensitive, query, filter), PageState
  construction, count clamping, ClipboardCopyRequest mode
- `crates/clipboard-daemon/src/main.rs` — 6 tests: sensitive copy
  flow, confirmation logic, MCP output redaction

## File Changes (this round)

- `crates/ui-gtk/src/lib.rs` — **ManagerConfig now carries full clipboard
  page configuration** (`clipboard_source`, `clipboard_filter`,
  `clipboard_query`, `clipboard_action`, `clipboard_count`,
  `clipboard_include_sensitive`); `initial_page` is now `Option<PageId>`
  (supports home, collections, settings as deep-link targets)
- `crates/ui-gtk/src/window/manager.rs` — **Manager clipboard page now
  wired from config** instead of `ClipboardPageProps::default()`; initial
  page uses `PageId` directly; removed `PickerSource→PageId` mapping
  function
- `crates/applet/src/main.rs` — `--source` is now `Option<SourceArg>`
  (falls back to config `picker.default_source` then History); `--count`
  is now `Option<usize>` (falls back to config `picker.max_results` than
  50); `--page` parsed as `PageId` (supports all pages); manager mode
  populates all `ManagerConfig` fields; added `parse_source_from_config()`
  and `PageId::from_str()` import
- `crates/ui-gtk/src/pages/clipboard.rs` — `load_entries_for` refactored
  into 6 focused helper functions (was 110 lines, split to stay under
  clippy limit)
