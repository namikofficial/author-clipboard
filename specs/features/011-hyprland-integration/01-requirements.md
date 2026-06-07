# Requirements: Hyprland Integration

---

## User Stories

### US-001: External Menu Picker
**As a** Hyprland user
**I want to** open clipboard history in wofi/rofi/fuzzel
**So that** I can quickly select and paste items

**Acceptance Criteria**:
- Given wofi is installed, when `author-clipboard-ctl picker` is run, then items appear in wofi
- Given I select an item in the menu, then it is copied to clipboard

### US-002: Native Picker
**As a** Hyprland user
**I want to** open a native GTK4 layer-shell popup
**So that** I have a first-party picker experience

**Acceptance Criteria**:
- Given `author-clipboard-hypr-picker` is run, then popup appears at cursor
- Given keyboard navigation works (arrows, enter, escape, delete, ctrl+1-9)

### US-003: Quick Paste
**As a** user
**I want to** type selected text directly into my application
**So that** I don't need manual paste

**Acceptance Criteria**:
- Given `wtype` is available, when I press Ctrl+Enter, then text is typed
- Given quick paste is preferred, when I select item, then it auto-pastes

---

## Picker Backends

| Backend | Command | Notes |
|---------|---------|-------|
| wofi | `wofi --show dmenu` | Most common |
| fuzzel | `fuzzel -d` | Fast, minimal |
| rofi | `rofi -show drun` | Feature-rich |

---

## Keybinds

```ini
# External menu picker
bind = SUPER, V, exec, author-clipboard-ctl picker --menu auto

# First-party native picker
bind = SUPER SHIFT, V, exec, author-clipboard-hypr-picker
```

---

**Last Updated**: Phase 19 (Hyprland-native UX)