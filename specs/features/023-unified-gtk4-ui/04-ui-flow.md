# UI Flow: Unified GTK4 UI

---

## Popup Flow (default `super+shift+v`)

```
            ┌────────── GTK4 layer-shell, anchor top+left+right ──────────┐
            │                                                              │
   ┌────── SearchEntry ──── FilterBar ─── DaemonIcon ── IncognitoChip ──┐  │
   │                                                                 │  │
   │   ┌──────────────────────────────────────────────────────────┐  │  │
   │   │ 1  📌  super+shift+v binding loaded …         2m · 80 ch │  │  │
   │   │ 2     text/html content                        1m · 24 ch │  │  │
   │   │ 3  🔒  redacted [8 chars detected]              5m · 18 ch │  │  │
   │   │ …                                                       │  │  │
   │   └──────────────────────────────────────────────────────────┘  │  │
   │                                                                 │  │
   │   ↑↓ navigate · / search · Enter copy · Ctrl+1..9 quick · Esc   │  │
   └─────────────────────────────────────────────────────────────────┘  │
            └────────────────────────────────────────────────────────────┘
```

### Key flows

**Open popup, copy first item, close**:
1. `super+shift+v` → `ui_gtk::run_popup()` → popup appears, list focused,
   index 0 selected, status hint shown.
2. `Enter` → `Action::Copy` → IPC `Copy { id, mode: Copy }` → toast
   `Copied to clipboard` for 800ms → popup closes.

**Open popup, type a search**:
1. `super+shift+v` → popup.
2. `/` → search input focused.
3. Type `git` → debounced 150ms search → items filtered live → list
   re-renders.
4. `Esc` (1st) → search cleared, list focused, full list returns.
5. `Esc` (2nd) → popup closes.

**Open popup on sensitive item**:
1. `super+shift+v` → popup opens, sensitive items show `🔒 redacted`
   chip in the meta line and a red 3px left border.
2. `↓` skips to a non-sensitive item; `Enter` copies it.

**Manager reveal sensitive content**:
1. `author-clipboard --manager` → manager opens.
2. `Ctrl+Shift+R` → toast "Redacted view: 5s" + countdown chip.
3. After 5s, reverts. Or `Esc` to clear.

## Manager Flow

```
 ┌─ AdwApplicationWindow ─────────────────────────────────────────────────┐
 │ ← →   Clipboard Manager                                          ⓧ │
 ├──────────────────────────────────────────────────────────────────────┤
 │ Sidebar      │  Content                                              │
 │ ─────────    │  ┌─ Search ──────────────────────── FilterBar ─┐    │
 │ 📋 Clipboard │  │ 🔍 search…                         [All]    │    │
 │ 😀 Emoji     │  └──────────────────────────────────────────────┘    │
 │ 🔣 Symbols   │  ┌─ List ────────────────────┐ ┌─ Preview ──────┐    │
 │ 🎭 Kaomoji   │  │ 1 📌  …                    │ │ ┌────────────┐ │    │
 │ 📑 Snippets  │  │ 2     …                    │ │ │ highlighted│ │    │
 │ ⚙ Settings   │  │ 3  🔒 …                    │ │ │   text     │ │    │
 │              │  │ …                          │ │ └────────────┘ │    │
 │              │  └────────────────────────────┘ └────────────────┘    │
 ├──────────────────────────────────────────────────────────────────────┤
 │ 8 items · 2 pinned · ● Daemon · 🔒 Incognito       200ms · ✓ copied  │
 └──────────────────────────────────────────────────────────────────────┘
```

The right preview pane appears only in manager mode and only at
widths > 900px; otherwise the popup pattern applies.

## Settings Page

```
┌─ Preferences ──────────────────────────────────────────────┐
│ General                                                     │
│   Source app detection            [●  ]                     │
│   Default action on Enter         [Copy        ▾]          │
│   Quick-paste on Ctrl+Enter       [●  ]                     │
│ Privacy                                                     │
│   Sensitive content detection     [●  ]                     │
│   Encrypt sensitive at rest       [  ●  ]                   │
│   Redact in lists                 [●  ]                     │
│   Confirm before copy sensitive   [●  ]                     │
│   Clear unpinned on screen lock   [●  ]                     │
│ Storage                                                     │
│   Max items                       [100           ▾]         │
│   Max item size                   [1MB           ▾]         │
│   Keep history                    [30 days       ▾]         │
│   Dedup window                    [2s            ▾]         │
│   Cleanup interval                [5m            ▾]         │
│ Data                                                       │
│   [ Clear unpinned ]   [ Export ]   [ Import ]              │
│ About                                                      │
│   Author Clipboard v0.6.0  ·  GPL-3.0  ·  …                 │
└────────────────────────────────────────────────────────────┘
```

Every row writes through to the same `Config` and persists via
`Config::save()` (existing path).

## Empty States

| State | Illustration | Title | Subtitle | Action |
|---|---|---|---|---|
| No history | empty-clipboard.svg | "Your clipboard is empty" | "Copy something to get started" | hint: "Super+Shift+V to reopen" |
| No results | empty-search.svg | "No matches for `foo`" | "Try fewer words or remove filters" | "Clear search" button |
| No sensitive | (none) | "Nothing redacted to show" | — | — |
| Daemon down | empty-warning.svg | "Daemon is not running" | "Start with: `systemctl --user start author-clipboard-daemon`" | "Retry" button |
| No snippets | empty-clipboard.svg | "No snippets yet" | "Add reusable text above" | focus name input |
| No pinned | empty-pin.svg | "Nothing pinned" | "Pin items with Ctrl+P" | — |

## Focus Flow Diagram

```
                    ┌──────────────┐
   open popup ─────►│ List focused │◄── / key captures
                    └──────┬───────┘
                           │
            ┌──────────────┼──────────────┐
            │ /            │ click        │ Ctrl+F
            ▼              ▼              ▼
       ┌──────────┐  ┌──────────┐   ┌──────────┐
       │ Search   │  │ Search   │   │ Search   │
       │ focused  │  │ focused  │   │ focused  │
       └────┬─────┘  └────┬─────┘   └────┬─────┘
            │             │              │
            │ Esc (empty) │ Esc          │ Esc (text)
            ▼             ▼              ▼
       List focused  List focused    List focused
       (popup stays) (popup stays)   (search cleared)
            │             │              │
            │ Esc (any)  │ Esc (any)    │ Esc (any)
            ▼             ▼              ▼
          close         close          close
```

## State Transitions (Clipboard page)

```
                  set_filter(Pinned)        set_search("git")
[All] ───────────────────────────► [All|empty|Pinned] ─────────────► [Pinned|"git"]
   ▲                                       │                              │
   │                                       │ clear_search                 │ clear_search
   │ clear_filter                          ▼                              ▼
[empty] ◄────────────────────────── [Pinned]                  [Pinned|empty|"git"]
```

---

**Last Updated**: 2026-06-12
