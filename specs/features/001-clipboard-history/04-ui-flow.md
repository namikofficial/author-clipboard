# UI Flow: Clipboard History

> User interaction flows for clipboard history list and item actions.

---

## Main Flow: History Tab

```
[Open Picker - History Tab]
        |
        v
[Load recent items from daemon]
        |
        v
[Display virtualized list]
        |
        v
[User scrolls/searches]
        |
        v
[User selects item]
        |
        v
[Show preview in right pane]
        |
        v
[User presses Enter]
        |
        v
[Copy item to clipboard]
        |
        v
[Close picker or stay open based on config]
```

---

## Item Actions

### Copy Item

1. Select item in list
2. Press Enter (or click Copy button)
3. Item written to Wayland clipboard
4. Toast notification: "Copied"
5. If `close_after_copy` is true, picker closes

### Pin/Unpin Item

1. Select item in list
2. Press Ctrl+P (or click pin button)
3. Pin icon toggles
4. Item moves to/from Pinned section
5. Toast notification: "Pinned" or "Unpinned"

### Star/Unstar Item (Phase 15)

1. Select item in list
2. Press Ctrl+Shift+S (or click star button)
3. Star icon toggles
4. Item ranking updates
5. Toast notification: "Starred" or "Unstarred"

### Delete Item

1. Select item in list
2. Press Delete (or click delete button)
3. Confirmation if item is pinned
4. Item removed from list
5. Toast notification: "Deleted"

### Quick Paste

1. Select item in list
2. Press Ctrl+Enter (or click Quick Paste button)
3. Item written to clipboard
4. Item typed into active window via wtype
5. Toast notification: "Pasted"

---

## Search Flow

```
[Focus search box]
        |
        v
[Type search query]
        |
        v
[Real-time filter as you type (debounced 200ms)]
        |
        v
[Results update in list]
        |
        v
[Press Enter to confirm or Escape to clear]
```

---

## Keyboard Navigation

| Key | Action |
|-----|--------|
| ↑ / ↓ | Move selection |
| Enter | Copy selected item |
| Ctrl+Enter | Quick paste selected item |
| Ctrl+P | Toggle pin |
| Ctrl+Shift+S | Toggle star |
| Delete | Delete selected item |
| / | Focus search box |
| Escape | Clear search / Close picker |
| Home | Jump to first item |
| End | Jump to last item |
| PageUp / PageDown | Page navigation |
| Ctrl+1-9 | Quick select by position |

---

## Empty States

### No History

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

### No Search Results

```
┌─────────────────────────────────────────┐
│                                         │
│         🔍 (large icon)                 │
│                                         │
│      No items match "query"             │
│                                         │
│   Try:                                   │
│   • Different keywords                  │
│   • Remove filters                      │
│   • Clear search                        │
│                                         │
│   [Clear Search]                        │
│                                         │
└─────────────────────────────────────────┘
```

---

## Visual Indicators

| Indicator | Meaning |
|-----------|---------|
| 📌 | Pinned item |
| ⭐ | Starred item (Phase 15) |
| 🔒 | Sensitive item |
| 📷 | Image item |
| 📄 | HTML item |
| 📁 | Files item |
| [kitty] | Source app chip |
| [today] | Age chip |
| [text] | Type chip |

---

**Last Updated**: Phase 15 (Updated from draft)