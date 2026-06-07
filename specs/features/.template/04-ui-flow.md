# UI Flow: {feature-name}

> User interaction flows and UI behavior.

---

## Main Flow

```
User Action → System Response → Next State

1. User clicks X
   System shows Y
2. User types Z
   System filters results
3. User presses Enter
   System copies item and closes
```

---

## Keyboard Navigation

| Key | Action |
|-----|--------|
| ↑ / ↓ | Navigate selection |
| Enter | Confirm / Copy |
| Escape | Cancel / Close |
| Ctrl+D | Delete selected |
| Ctrl+1-9 | Quick select by position |
| Home / End | Jump to first / last |
| PgUp / PgDn | Page navigation |

---

## UI States

### Empty State
When there are no items, show:
- Icon: clipboard with X
- Text: "No clipboard history yet"
- Hint: "Copy something to get started"

### Loading State
- Spinner or skeleton UI
- "Loading history..."

### Error State
- Error icon
- Error message
- Retry button if applicable

---

## Accessibility

- All interactive elements focusable
- Screen reader labels for icons
- Keyboard-only operation fully supported
- High contrast mode compatible

---

**Last Updated**: {date}