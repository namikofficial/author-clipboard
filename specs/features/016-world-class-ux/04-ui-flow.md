# UI Flow: World-Class UX

> Detailed interaction flows and state transitions for the premium UX.

---

## List Item States

### Default State
- Background: transparent
- Content: truncated at 80 chars with "..."
- Chips: source app, age, type
- Actions: hidden

### Hover State
- Background: subtle highlight (system highlight)
- Actions: visible (pin, star, delete, copy)
- Preview pane: shows item details

### Selected State
- Background: primary color at 10% opacity
- Border: 2px primary color on left
- Preview pane: shows full content

### Pinned State
- Pin icon: 📌 at top right
- Background: subtle green tint
- Position: always at top of list

### Starred State
- Star icon: ⭐ at top right
- Background: subtle yellow tint

### Sensitive State
- Red ribbon: 4px red border on left
- Lock icon: 🔒 next to content
- Preview: masked content

---

## Preview Pane Transitions

### Text Item Selected

```
[Select item in list]
        |
        v
[Preview pane fades in] (100ms)
        |
        v
[Content appears with syntax highlighting]
        |
        v
[Line numbers appear on left]
```

### Image Item Selected

```
[Select image in list]
        |
        v
[Preview pane shows loading spinner] (50ms)
        |
        v
[Image loads from thumbnail path]
        |
        v
[Dimensions badge appears: "1920x1080 • 2.4MB"]
```

### Sensitive Item Selected

```
[Select sensitive item in list]
        |
        v
[Preview pane shows lock icon]
        |
        v
[Content masked: "••••••••••••••••"]
        |
        v
[Warning text: "Sensitive content detected ••••"]
```

---

## Tab Navigation

### Tab Switch Animation

```
[Click "Snippets" tab]
        |
        v
[Current content fades out] (100ms)
        |
        v
[New content fades in] (100ms)
        |
        v
[Tab indicator slides to new position] (150ms)
```

---

## Keyboard Shortcuts Overlay

### Activation

```
[Press "?"]
        |
        v
[Overlay fades in] (100ms)
        |
        v
[Focus trap in overlay]
        |
        v
[Press Escape to close]
```

### Overlay Layout

```
┌─────────────────────────────────────────┐
│ Keyboard Shortcuts                   ✕ │
├─────────────────────────────────────────┤
│ Navigation                              │
│   ↑/↓     Move selection                │
│   Enter    Copy selected item            │
│   Tab      Next tab                      │
│   Esc      Close picker                  │
│                                         │
│ Actions                                 │
│   Ctrl+P   Pin/unpin item               │
│   Ctrl+S   Star/unstar item             │
│   Ctrl+D   Delete item                 │
│   Ctrl+,   Open settings                │
│                                         │
│ Search                                  │
│   /         Focus search                 │
│   Ctrl+F   Open filters                 │
│   Ctrl+S   Save current search          │
├─────────────────────────────────────────┤
│ Press Escape to close                   │
└─────────────────────────────────────────┘
```

---

## Empty States

### No Clipboard History

```
┌─────────────────────────────────────────┐
│                                         │
│            📋 (large icon)              │
│                                         │
│      No clipboard history yet           │
│                                         │
│   Copy something to get started!        │
│                                         │
│   Tip: Press Super+V to open picker     │
│                                         │
└─────────────────────────────────────────┘
```

### No Pinned Items

```
┌─────────────────────────────────────────┐
│                                         │
│            📌 (large icon)              │
│                                         │
│        No pinned items                  │
│                                         │
│   Pin items you want to keep forever    │
│                                         │
│   Press Ctrl+P to pin selected item     │
│                                         │
└─────────────────────────────────────────┘
```

### No Search Results

```
┌─────────────────────────────────────────┐
│                                         │
│         🔍 (large icon)                 │
│                                         │
│      No items match your search         │
│                                         │
│   Try:                                   │
│   • Different keywords                  │
│   • Remove filters                      │
│   • Clear search to see all             │
│                                         │
│   [Clear Search]                        │
│                                         │
└─────────────────────────────────────────┘
```

---

## Loading States

### Initial Load

```
┌─────────────────────────────────────────┐
│                                         │
│         ◌ Loading history...             │
│                                         │
│                                         │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

### Search in Progress

```
┌─────────────────────────────────────────┐
│ [Search box with spinner]                │
├─────────────────────────────────────────┤
│                                         │
│         ◌ Searching...                  │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

### Item Copying

```
┌─────────────────────────────────────────┐
│ [Selected item with checkmark]           │
│                                         │
│      ✓ Copied to clipboard              │
│         (fades after 1s)                │
│                                         │
└─────────────────────────────────────────┘
```

---

## Error States

### Daemon Not Running

```
┌─────────────────────────────────────────┐
│                                         │
│         ⚠️ (large icon)                 │
│                                         │
│       Daemon not running                 │
│                                         │
│   Start the daemon with:                │
│   systemctl --user start                │
│   author-clipboard-daemon                │
│                                         │
│   [Retry]                                │
│                                         │
└─────────────────────────────────────────┘
```

### Copy Failed

```
┌─────────────────────────────────────────┐
│ [Selected item]                         │
│                                         │
│   ⚠️ Copy failed                        │
│      Could not write to clipboard        │
│                                         │
│   [Retry] [Copy as Plain Text]          │
│                                         │
└─────────────────────────────────────────┘
```

---

## Status Bar

### Normal State

```
│ 150 items | 12 pinned | ● Daemon | 🔒 Incognito │
```

- `150 items`: Total item count
- `12 pinned`: Pinned item count
- `●`: Daemon running (green dot)
- `🔒`: Incognito mode active

### Daemon Down

```
│ 150 items | 12 pinned | ○ Daemon | 🔒 Incognito │
```

- `○`: Daemon not running (gray dot)

---

**Last Updated**: Phase 15