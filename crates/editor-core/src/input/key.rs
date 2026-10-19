//! Keyboard types.
//!
//! Defines the editor's own key vocabulary so the rest of the codebase never
//! depends on crossterm. [`Key`] is `Copy`, `Eq`, and `Hash`, which lets a
//! keymap use it directly as a lookup key.
//!
//! `From<CtKeyEvent>` is the single boundary where terminal events are
//! normalized; SHIFT is folded into the character itself for `Char` keys
//! (`Shift+w` arrives as `'W'`) and kept as a modifier for everything else.

use crossterm::event::{KeyCode as CtKeyCode, KeyEvent as CtKeyEvent, KeyModifiers as CtMods};

/// The editor's normalized key type. `Copy`, `Eq`, `Hash` — usable as a trie key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key {
    pub code: KeyCode,
    pub mods: Mods,
}

/// The editor's key vocabulary (a subset of crossterm, plus anything we add).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyCode {
    Char(char),
    Esc,
    Enter,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    // Add variants here as the keymap needs them.
}

/// Modifiers as a compact bitflag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Mods(u8);

impl std::ops::BitOr for Mods {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Mods(self.0 | rhs.0)
    }
}

impl Mods {
    pub const NONE: Mods = Mods(0);
    pub const CTRL: Mods = Mods(1 << 0);
    pub const ALT: Mods = Mods(1 << 1);
    pub const SHIFT: Mods = Mods(1 << 2);
    pub const SUPER: Mods = Mods(1 << 3);
    // Combine modifiers with `|`; add `contains` here if membership tests
    // are needed.
}

impl Key {
    pub fn new(code: KeyCode, mods: Mods) -> Self {
        Self { code, mods }
    }

    pub const fn plain(code: KeyCode) -> Self {
        Self {
            code,
            mods: Mods::NONE,
        }
    }

    pub const fn char(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            mods: Mods::NONE,
        }
    }
    pub const fn ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            mods: Mods::CTRL,
        }
    }
    pub const fn esc() -> Self {
        Self {
            code: KeyCode::Esc,
            mods: Mods::NONE,
        }
    }
}

/// The only place crossterm is touched for input. Normalizes at the boundary.
impl From<CtKeyEvent> for Key {
    fn from(e: CtKeyEvent) -> Self {
        // Normalize modifiers: drop SHIFT for Char (Shift+w == 'W'), keep for non-char.
        let mut mods = Mods::NONE;
        if e.modifiers.contains(CtMods::CONTROL) {
            mods = mods | Mods::CTRL;
        }
        if e.modifiers.contains(CtMods::ALT) {
            mods = mods | Mods::ALT;
        }
        if e.modifiers.contains(CtMods::SUPER) {
            mods = mods | Mods::SUPER;
        }
        // Note: deliberately ignore SHIFT on Char — 'W' already encodes it.
        if !matches!(e.code, CtKeyCode::Char(_)) && e.modifiers.contains(CtMods::SHIFT) {
            mods = mods | Mods::SHIFT;
        }

        let code = match e.code {
            CtKeyCode::Char(c) => KeyCode::Char(c),
            CtKeyCode::Esc => KeyCode::Esc,
            CtKeyCode::Enter => KeyCode::Enter,
            CtKeyCode::Tab => KeyCode::Tab,
            CtKeyCode::Backspace => KeyCode::Backspace,
            CtKeyCode::Delete => KeyCode::Delete,
            CtKeyCode::Up => KeyCode::Up,
            CtKeyCode::Down => KeyCode::Down,
            CtKeyCode::Left => KeyCode::Left,
            CtKeyCode::Right => KeyCode::Right,
            CtKeyCode::Home => KeyCode::Home,
            CtKeyCode::End => KeyCode::End,
            CtKeyCode::PageUp => KeyCode::PageUp,
            CtKeyCode::PageDown => KeyCode::PageDown,
            CtKeyCode::F(n) => KeyCode::F(n),
            _ => KeyCode::Char('\0'), // or a KeyCode::Unsupported variant
        };
        Self { code, mods }
    }
}
