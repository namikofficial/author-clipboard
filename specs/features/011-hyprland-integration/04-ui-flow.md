# UI Flow: Hyprland Integration

> User interaction flows for Hyprland-native picker.

---

## Picker Flow

```
[Super+Shift+V pressed]
        |
        v
[author-clipboard-hypr-picker starts]
        |
        v
[Load items via IPC from daemon]
        |
        v
[GTK4 window opens as layer-shell overlay]
        |
        v
[User types to search / navigates with arrows]
        |
        v
[User presses Enter to select]
        |
        v
[Item copied to clipboard]
        |
        v
[Picker closes (if close_after_copy)]
```

---

## Keyboard Navigation

| Key | Action |
|-----|--------|
| ↑ / ↓ | Navigate items |
| Enter | Copy selected item |
| Escape | Close picker |
| / | Focus search |
| Backspace | Clear search |
| Delete | Delete selected item |
| Ctrl+P | Toggle pin |

---

**Last Updated**: Phase 15 (Updated from draft)