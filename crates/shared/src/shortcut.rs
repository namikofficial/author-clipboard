//! Shortcut parsing and conflict detection module.
//!
//! Provides types for representing keyboard shortcuts, parsing them from
//! string representations like `"Super+V"`, and detecting conflicts with
//! known compositor shortcuts.

#![allow(dead_code)]

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Error type for shortcut operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ShortcutError {
    /// An invalid modifier key was encountered.
    #[error("invalid modifier key: {0}")]
    InvalidModifier(String),

    /// An invalid key code was encountered.
    #[error("invalid key code: {0}")]
    InvalidKeyCode(String),

    /// A shortcut must have at least one modifier key.
    #[error("shortcut must have at least one modifier")]
    NoModifiers,

    /// The shortcut string format is invalid.
    #[error("invalid shortcut format: {0}")]
    InvalidFormat(String),
}

/// Modifier keys for keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModifierKey {
    Super,
    Ctrl,
    Alt,
    Shift,
}

impl fmt::Display for ModifierKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Super => "Super",
            Self::Ctrl => "Ctrl",
            Self::Alt => "Alt",
            Self::Shift => "Shift",
        };
        f.write_str(s)
    }
}

impl FromStr for ModifierKey {
    type Err = ShortcutError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "super" | "meta" | "logo" => Ok(Self::Super),
            "ctrl" | "control" => Ok(Self::Ctrl),
            "alt" => Ok(Self::Alt),
            "shift" => Ok(Self::Shift),
            _ => Err(ShortcutError::InvalidModifier(s.to_string())),
        }
    }
}

/// Key codes for keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // Digits
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // Special keys
    Space,
    Tab,
    Return,
    Escape,
    Delete,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    PrintScreen,
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::Space => "Space",
            Self::Tab => "Tab",
            Self::Return => "Return",
            Self::Escape => "Escape",
            Self::Delete => "Delete",
            Self::Backspace => "Backspace",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::Insert => "Insert",
            Self::PrintScreen => "PrintScreen",
        };
        f.write_str(s)
    }
}

impl FromStr for KeyCode {
    type Err = ShortcutError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            "c" => Ok(Self::C),
            "d" => Ok(Self::D),
            "e" => Ok(Self::E),
            "f" => Ok(Self::F),
            "g" => Ok(Self::G),
            "h" => Ok(Self::H),
            "i" => Ok(Self::I),
            "j" => Ok(Self::J),
            "k" => Ok(Self::K),
            "l" => Ok(Self::L),
            "m" => Ok(Self::M),
            "n" => Ok(Self::N),
            "o" => Ok(Self::O),
            "p" => Ok(Self::P),
            "q" => Ok(Self::Q),
            "r" => Ok(Self::R),
            "s" => Ok(Self::S),
            "t" => Ok(Self::T),
            "u" => Ok(Self::U),
            "v" => Ok(Self::V),
            "w" => Ok(Self::W),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            "z" => Ok(Self::Z),
            "0" => Ok(Self::Digit0),
            "1" => Ok(Self::Digit1),
            "2" => Ok(Self::Digit2),
            "3" => Ok(Self::Digit3),
            "4" => Ok(Self::Digit4),
            "5" => Ok(Self::Digit5),
            "6" => Ok(Self::Digit6),
            "7" => Ok(Self::Digit7),
            "8" => Ok(Self::Digit8),
            "9" => Ok(Self::Digit9),
            "f1" => Ok(Self::F1),
            "f2" => Ok(Self::F2),
            "f3" => Ok(Self::F3),
            "f4" => Ok(Self::F4),
            "f5" => Ok(Self::F5),
            "f6" => Ok(Self::F6),
            "f7" => Ok(Self::F7),
            "f8" => Ok(Self::F8),
            "f9" => Ok(Self::F9),
            "f10" => Ok(Self::F10),
            "f11" => Ok(Self::F11),
            "f12" => Ok(Self::F12),
            "space" => Ok(Self::Space),
            "tab" => Ok(Self::Tab),
            "return" | "enter" => Ok(Self::Return),
            "escape" | "esc" => Ok(Self::Escape),
            "delete" | "del" => Ok(Self::Delete),
            "backspace" | "bs" => Ok(Self::Backspace),
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "home" => Ok(Self::Home),
            "end" => Ok(Self::End),
            "pageup" | "page_up" => Ok(Self::PageUp),
            "pagedown" | "page_down" => Ok(Self::PageDown),
            "insert" | "ins" => Ok(Self::Insert),
            "printscreen" | "print" | "prtsc" => Ok(Self::PrintScreen),
            _ => Err(ShortcutError::InvalidKeyCode(s.to_string())),
        }
    }
}

/// A parsed keyboard shortcut binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutBinding {
    /// Modifier keys (sorted, deduplicated).
    pub modifiers: Vec<ModifierKey>,
    /// The primary key.
    pub key: KeyCode,
}

impl ShortcutBinding {
    /// Parse a shortcut string like `"Super+V"` into a `ShortcutBinding`.
    pub fn parse(s: &str) -> Result<Self, ShortcutError> {
        s.parse()
    }

    /// Validate that the binding has at least one modifier key.
    pub fn validate(&self) -> Result<(), ShortcutError> {
        if self.modifiers.is_empty() {
            return Err(ShortcutError::NoModifiers);
        }
        Ok(())
    }
}

impl fmt::Display for ShortcutBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, modifier) in self.modifiers.iter().enumerate() {
            if i > 0 {
                f.write_str("+")?;
            }
            write!(f, "{modifier}")?;
        }
        if !self.modifiers.is_empty() {
            f.write_str("+")?;
        }
        write!(f, "{}", self.key)
    }
}

impl FromStr for ShortcutBinding {
    type Err = ShortcutError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();
        let key_str = parts
            .last()
            .ok_or_else(|| ShortcutError::InvalidFormat(s.to_string()))?;

        let key = KeyCode::from_str(key_str.trim())?;

        let mut modifiers: Vec<ModifierKey> = parts[..parts.len() - 1]
            .iter()
            .map(|p| ModifierKey::from_str(p.trim()))
            .collect::<Result<Vec<_>, _>>()?;
        modifiers.sort_unstable();
        modifiers.dedup();

        Ok(Self { modifiers, key })
    }
}

/// A detected shortcut conflict with a known compositor binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConflict {
    /// The conflicting binding.
    pub binding: ShortcutBinding,
    /// The application or compositor that uses this shortcut.
    pub application: String,
    /// Description of what the shortcut does.
    pub description: String,
}

/// An entry in the table of known compositor shortcuts.
struct KnownShortcut {
    modifiers: &'static [ModifierKey],
    key: KeyCode,
    application: &'static str,
    description: &'static str,
}

/// Known compositor shortcuts that may conflict with user bindings.
const KNOWN_SHORTCUTS: &[KnownShortcut] = &[
    KnownShortcut {
        modifiers: &[ModifierKey::Super],
        key: KeyCode::L,
        application: "COSMIC/GNOME",
        description: "Lock screen",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Ctrl, ModifierKey::Alt],
        key: KeyCode::T,
        application: "COSMIC/GNOME",
        description: "Open terminal",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Ctrl, ModifierKey::Alt],
        key: KeyCode::Delete,
        application: "COSMIC/GNOME",
        description: "Log out",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Super],
        key: KeyCode::D,
        application: "COSMIC",
        description: "Show desktop",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Alt],
        key: KeyCode::F4,
        application: "COSMIC/GNOME",
        description: "Close window",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Super],
        key: KeyCode::Tab,
        application: "COSMIC",
        description: "Task switcher",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Alt],
        key: KeyCode::Tab,
        application: "COSMIC/GNOME",
        description: "Window switcher",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Super],
        key: KeyCode::A,
        application: "COSMIC",
        description: "App launcher",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Super],
        key: KeyCode::E,
        application: "GNOME",
        description: "File manager",
    },
    KnownShortcut {
        modifiers: &[ModifierKey::Super],
        key: KeyCode::Space,
        application: "GNOME",
        description: "Input method switch",
    },
];

/// Check if a shortcut binding conflicts with any known compositor shortcuts.
pub fn check_conflicts(binding: &ShortcutBinding) -> Vec<ShortcutConflict> {
    let mut sorted = binding.modifiers.clone();
    sorted.sort_unstable();

    KNOWN_SHORTCUTS
        .iter()
        .filter(|known| binding.key == known.key && sorted.as_slice() == known.modifiers)
        .map(|known| ShortcutConflict {
            binding: binding.clone(),
            application: known.application.to_string(),
            description: known.description.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_super_v() {
        let binding = ShortcutBinding::parse("Super+V").unwrap();
        assert_eq!(binding.modifiers, vec![ModifierKey::Super]);
        assert_eq!(binding.key, KeyCode::V);
        assert!(binding.validate().is_ok());
    }

    #[test]
    fn test_parse_ctrl_shift_c() {
        let binding = ShortcutBinding::parse("Ctrl+Shift+C").unwrap();
        assert_eq!(
            binding.modifiers,
            vec![ModifierKey::Ctrl, ModifierKey::Shift]
        );
        assert_eq!(binding.key, KeyCode::C);
        assert!(binding.validate().is_ok());
    }

    #[test]
    fn test_parse_invalid() {
        assert!(ShortcutBinding::parse("Super+InvalidKey").is_err());
        assert!(ShortcutBinding::parse("Bogus+A").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let inputs = ["Super+V", "Ctrl+Shift+C", "Alt+F4", "Ctrl+Alt+Delete"];
        for input in inputs {
            let binding = ShortcutBinding::parse(input).unwrap();
            let output = binding.to_string();
            let reparsed = ShortcutBinding::parse(&output).unwrap();
            assert_eq!(binding, reparsed, "roundtrip failed for {input}");
        }
    }

    #[test]
    fn test_validate_no_modifiers() {
        let binding = ShortcutBinding::parse("V").unwrap();
        assert!(binding.modifiers.is_empty());
        assert!(matches!(
            binding.validate(),
            Err(ShortcutError::NoModifiers)
        ));
    }

    #[test]
    fn test_conflict_detection() {
        let binding = ShortcutBinding::parse("Super+L").unwrap();
        let conflicts = check_conflicts(&binding);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].application, "COSMIC/GNOME");
        assert_eq!(conflicts[0].description, "Lock screen");
    }

    #[test]
    fn test_no_conflicts() {
        let binding = ShortcutBinding::parse("Super+V").unwrap();
        let conflicts = check_conflicts(&binding);
        assert!(conflicts.is_empty(), "Super+V should not conflict");
    }

    #[test]
    fn test_modifier_case_insensitive() {
        let binding = ShortcutBinding::parse("super+v").unwrap();
        assert_eq!(binding.modifiers, vec![ModifierKey::Super]);
        assert_eq!(binding.key, KeyCode::V);
    }

    #[test]
    fn test_duplicate_modifiers_deduped() {
        let binding = ShortcutBinding::parse("Ctrl+Ctrl+A").unwrap();
        assert_eq!(binding.modifiers, vec![ModifierKey::Ctrl]);
    }

    #[test]
    fn test_display_formatting() {
        let binding = ShortcutBinding {
            modifiers: vec![ModifierKey::Ctrl, ModifierKey::Alt],
            key: KeyCode::T,
        };
        assert_eq!(binding.to_string(), "Ctrl+Alt+T");
    }
}
