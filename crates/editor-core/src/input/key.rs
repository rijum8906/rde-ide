//! Crossterm key re-exports and input helpers.

pub use crossterm::event::{
    KeyCode, KeyEvent, KeyEvent as Key, KeyEventKind, KeyEventState, KeyModifiers,
};

/// Helper to create a plain key event with no modifiers.
pub const fn plain(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Helper to create a char key event.
/// Automatically adds SHIFT if the character is uppercase.
pub const fn char(c: char) -> KeyEvent {
    let modifiers = if c.is_uppercase() {
        KeyModifiers::SHIFT
    } else {
        KeyModifiers::NONE
    };
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Helper to create a Ctrl+<char> key event.
pub const fn ctrl(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c.to_ascii_lowercase()),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Helper to create a Shift+<char> key event.
pub const fn shift(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::SHIFT,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Helper to create an Alt+<char> key event.
pub const fn alt(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c.to_ascii_lowercase()),
        modifiers: KeyModifiers::ALT,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Extension trait providing associated constructor functions and helpers on `Key`/`KeyEvent`.
pub trait KeyExt {
    fn plain(code: KeyCode) -> Self;
    fn char(c: char) -> Self;
    fn ctrl(c: char) -> Self;
    fn shift(c: char) -> Self;
    fn alt(c: char) -> Self;
    fn is_ctrl(&self) -> bool;
    fn is_alt(&self) -> bool;
    fn is_shift(&self) -> bool;
}

impl KeyExt for KeyEvent {
    fn plain(code: KeyCode) -> Self {
        plain(code)
    }

    fn char(c: char) -> Self {
        char(c)
    }

    fn ctrl(c: char) -> Self {
        ctrl(c)
    }

    fn shift(c: char) -> Self {
        shift(c)
    }

    fn alt(c: char) -> Self {
        alt(c)
    }

    fn is_ctrl(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_alt(&self) -> bool {
        self.modifiers.contains(KeyModifiers::ALT)
    }

    fn is_shift(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SHIFT)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_plain_key() {
        let k = plain(KeyCode::Enter);
        assert_eq!(k.code, KeyCode::Enter);
        assert_eq!(k.modifiers, KeyModifiers::NONE);
        assert_eq!(k.kind, KeyEventKind::Press);
    }

    #[test]
    fn test_char_key_case() {
        let lower = char('a');
        assert_eq!(lower.code, KeyCode::Char('a'));
        assert_eq!(lower.modifiers, KeyModifiers::NONE);

        let upper = char('A');
        assert_eq!(upper.code, KeyCode::Char('A'));
        assert_eq!(upper.modifiers, KeyModifiers::SHIFT);
        assert!(upper.is_shift());
    }

    #[test]
    fn test_ctrl_alt_shift() {
        let c = ctrl('c');
        assert_eq!(c.code, KeyCode::Char('c'));
        assert_eq!(c.modifiers, KeyModifiers::CONTROL);
        assert!(c.is_ctrl());
        assert!(!c.is_alt());

        let a = alt('x');
        assert_eq!(a.code, KeyCode::Char('x'));
        assert_eq!(a.modifiers, KeyModifiers::ALT);
        assert!(a.is_alt());
        assert!(!a.is_ctrl());
    }

    #[test]
    fn test_key_ext_associated_functions() {
        let k = Key::plain(KeyCode::Esc);
        assert_eq!(k.code, KeyCode::Esc);
        assert_eq!(k.modifiers, KeyModifiers::NONE);

        let k2 = Key::ctrl('q');
        assert_eq!(k2.code, KeyCode::Char('q'));
        assert!(k2.is_ctrl());
    }
}
