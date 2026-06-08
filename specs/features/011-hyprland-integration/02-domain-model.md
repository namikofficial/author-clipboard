# Domain Model: Hyprland Integration

> Data structures and architecture for native Hyprland picker.

---

## Architecture

```
Hyprland
    |
    v
wlr-data-control (Wayland protocol)
    |
    v
author-clipboard-daemon (capture)
    |
    v
IPC Socket
    |
    v
author-clipboard-hypr-picker (GTK4 layer-shell picker)
```

---

## Hypr-Picker Components

```rust
// In hypr-picker/src/main.rs

pub struct HyprPicker {
    pub source: PickerSource,  // history, snippets, emoji, etc.
    pub count: usize,
    pub action: PickerAction,  // copy, quick-paste
    pub include_sensitive: bool,
}

pub enum PickerSource {
    History,
    Snippets,
    Emoji,
    Symbols,
    Kaomoji,
    All,
}

pub enum PickerAction {
    Copy,
    QuickPaste,
}
```

---

## Hyprland Configuration

### Keybinds

```lua
# ~/.config/hypr/hyprland.conf

# External menu picker
bind = SUPER, V, exec, author-clipboard-ctl picker --menu rofi --source history

# First-party Hyprland-native picker
bind = SUPER SHIFT, V, exec, author-clipboard-hypr-picker
```

### Window Rules

```lua
# Make picker appear as floating layer
windowrulev2 = float, title:author-clipboard-hypr-picker
windowrulev2 = pin, title:author-clipboard-hypr-picker
windowrulev2 = opacity 0.95 0.95, title:author-clipboard-hypr-picker
```

---

**Last Updated**: Phase 15 (Updated from draft)