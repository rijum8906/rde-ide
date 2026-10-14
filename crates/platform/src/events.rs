use crossterm::event::{KeyCode as CtKeyCode, KeyEvent, KeyModifiers as CtModifiers};
use editor_core::input::key::{Key, KeyCode, Modifiers};

pub fn to_key(ev: KeyEvent) -> Option<Key> {
    let code = match ev.code {
        CtKeyCode::Char(c) => KeyCode::Char(c),
        CtKeyCode::Enter => KeyCode::Enter,
        CtKeyCode::Esc => KeyCode::Escape,
        CtKeyCode::Tab => KeyCode::Tab,
        CtKeyCode::BackTab => KeyCode::BackTab,
        CtKeyCode::Backspace => KeyCode::Backspace,
        CtKeyCode::Delete => KeyCode::Delete,
        CtKeyCode::Insert => KeyCode::Insert,
        CtKeyCode::Home => KeyCode::Home,
        CtKeyCode::End => KeyCode::End,
        CtKeyCode::PageUp => KeyCode::PageUp,
        CtKeyCode::PageDown => KeyCode::PageDown,
        CtKeyCode::Up => KeyCode::Up,
        CtKeyCode::Down => KeyCode::Down,
        CtKeyCode::Left => KeyCode::Left,
        CtKeyCode::Right => KeyCode::Right,
        CtKeyCode::F(n) => KeyCode::F(n),
        CtKeyCode::Null => KeyCode::Null,
        _ => return None,
    };

    let modifiers = Modifiers {
        ctrl: ev.modifiers.contains(CtModifiers::CONTROL),
        alt: ev.modifiers.contains(CtModifiers::ALT),
        shift: ev.modifiers.contains(CtModifiers::SHIFT),
    };

    Some(Key { code, modifiers })
}
