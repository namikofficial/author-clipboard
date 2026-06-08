# Technical Design: UI Polish

> Implementation approach for keyboard navigation and visual refinements.

---

## Overview

UI polish focuses on keyboard navigation and visual refinements that make the picker feel premium.

---

## Affected Files

| File | Change |
|------|--------|
| `crates/applet/src/keyboard.rs` | New keyboard handler module |
| `crates/applet/src/main.rs` | Keyboard event handling |
| `crates/applet/src/icons.rs` | Icon definitions |
| `crates/hypr-picker/src/main.rs` | Similar keyboard handling |

---

## Implementation Details

### Keyboard Handler

```rust
// In applet/src/keyboard.rs

pub enum KeyAction {
    MoveUp,
    MoveDown,
    MoveToFirst,
    MoveToLast,
    PageUp,
    PageDown,
    SelectByPosition(usize),
    Copy,
    Delete,
    TabNext,
    TabPrevious,
    Close,
}

impl App {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<KeyAction> {
        match (key.modifiers, key.code) {
            (NONE, KeyCode::ArrowUp) => Some(KeyAction::MoveUp),
            (NONE, KeyCode::ArrowDown) => Some(KeyAction::MoveDown),
            (NONE, KeyCode::Home) => Some(KeyAction::MoveToFirst),
            (NONE, KeyCode::End) => Some(KeyAction::MoveToLast),
            (NONE, KeyCode::PageUp) => Some(KeyAction::PageUp),
            (NONE, KeyCode::PageDown) => Some(KeyAction::PageDown),
            (CONTROL, KeyCode::Digit1..=Digit9) => {
                let pos = (key.code - KeyCode::Digit1) as usize + 1;
                Some(KeyAction::SelectByPosition(pos))
            }
            (NONE, KeyCode::Enter) => Some(KeyAction::Copy),
            (NONE, KeyCode::Delete) => Some(KeyAction::Delete),
            (CONTROL, KeyCode::Tab) => Some(KeyAction::TabNext),
            (CONTROL | SHIFT, KeyCode::Tab) => Some(KeyAction::TabPrevious),
            (NONE, KeyCode::Escape) => Some(KeyAction::Close),
            _ => None,
        }
    }

    fn execute_action(&mut self, action: KeyAction) {
        match action {
            KeyAction::MoveUp => self.move_selection(-1),
            KeyAction::MoveDown => self.move_selection(1),
            KeyAction::MoveToFirst => self.selected_index = Some(0),
            KeyAction::MoveToLast => self.selected_index = Some(self.items.len() - 1),
            // ... etc
        }
    }
}
```

---

## Testing

1. Manual keyboard navigation tests
2. Verify all shortcuts work as expected
3. Test quick select with Ctrl+1-9

---

**Last Updated**: Phase 15 (Updated from draft)