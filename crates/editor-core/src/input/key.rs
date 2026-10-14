//! Backend-agnostic key representation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Enter,
    Escape,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F(u8),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        alt: true,
        shift: false,
    };
    pub const SHIFT: Self = Self {
        ctrl: false,
        alt: false,
        shift: true,
    };
}

impl Key {
    /// Convenience constructor for plain characters.
    pub const fn char(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: Modifiers::NONE,
        }
    }

    pub const fn ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: Modifiers::CTRL,
        }
    }

    pub const fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: Modifiers::NONE,
        }
    }

    /// A sentinel that can never be produced by a terminal.
    /// Used internally for buffer initialization.
    pub const fn none() -> Self {
        Self {
            code: KeyCode::Null,
            modifiers: Modifiers::NONE,
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self.code, KeyCode::Null)
    }
}

// Readable Debug: prints `C-w`, `Esc`, `a`, `S-Tab`, etc.
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.ctrl {
            write!(f, "C-")?;
        }
        if self.modifiers.alt {
            write!(f, "M-")?;
        }
        if self.modifiers.shift {
            write!(f, "S-")?;
        }
        match self.code {
            KeyCode::Char(' ') => write!(f, "Space"),
            KeyCode::Char(c) => write!(f, "{c}"),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::Escape => write!(f, "Esc"),
            KeyCode::Tab => write!(f, "Tab"),
            KeyCode::BackTab => write!(f, "BackTab"),
            KeyCode::Backspace => write!(f, "BS"),
            KeyCode::Delete => write!(f, "Del"),
            KeyCode::Insert => write!(f, "Ins"),
            KeyCode::Home => write!(f, "Home"),
            KeyCode::End => write!(f, "End"),
            KeyCode::PageUp => write!(f, "PgUp"),
            KeyCode::PageDown => write!(f, "PgDn"),
            KeyCode::Up => write!(f, "Up"),
            KeyCode::Down => write!(f, "Down"),
            KeyCode::Left => write!(f, "Left"),
            KeyCode::Right => write!(f, "Right"),
            KeyCode::F(n) => write!(f, "F{n}"),
            KeyCode::Null => write!(f, "Nul"),
        }
    }
}
