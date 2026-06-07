# UI Flow: UI Polish

> User interaction flows for keyboard navigation and visual polish.

---

## Keyboard Navigation Flow

```
[Picker Opens]
        |
        v
[First item selected]
        |
        v
 [↑ pressed] --> [Move selection up]
        |
[v pressed] --> [Move selection down]
        |
        v
[End pressed] --> [Jump to last item]
[Home pressed] --> [Jump to first item]
[PageUp pressed] --> [Move up one page]
[PageDown pressed] --> [Move down one page]
```

---

## Quick Select Flow

```
[Ctrl+1-9 pressed]
        |
        v
[Calculate item index from key]
        |
        v
[If item exists at index]
        |
        v
[Select item]
        |
        v
[Copy item (Enter pressed automatically)]
```

---

## Tab Cycling Flow

```
[Ctrl+Tab pressed]
        |
        v
[Move to next tab]
        |
        v
[If past last tab, wrap to first]
        |
        v
[Update tab indicator]
        |
        v
[Load tab content]
```

---

## Delete Flow

```
[Delete pressed]
        |
        v
[Is item pinned?]
        |
        Yes
        v
[Show confirmation dialog]
        |
        v
[User confirms]
        |
        v
[Delete item]
        |
        v
[No]
        v
[Delete item]
```

---

**Last Updated**: Phase 15 (Updated from draft)