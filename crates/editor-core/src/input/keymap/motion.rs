//! Default keybindings for cursor motion.

use crate::{
    command::{
        Command,
        MotionCommand::{
            BigWordEnd, Down, FileEnd, FileStart, Left, LineEnd, LineStart, NextBigWordStart,
            NextWordStart, PrevBigWordStart, PrevWordStart, Right, Up, WordEnd,
        },
    },
    input::{
        key::{Key, KeyCode},
        keymap::Keymap,
        mode::InputMode,
    },
};

pub(super) fn default_motion_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = InputMode::Normal;
    keymap.bind(normal, vec![Key::char('h')], Command::Motion(Left));
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Left)],
        Command::Motion(Left),
    );
    keymap.bind(normal, vec![Key::char('j')], Command::Motion(Down));
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Down)],
        Command::Motion(Down),
    );
    keymap.bind(normal, vec![Key::char('k')], Command::Motion(Up));
    keymap.bind(normal, vec![Key::plain(KeyCode::Up)], Command::Motion(Up));
    keymap.bind(normal, vec![Key::char('l')], Command::Motion(Right));
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Right)],
        Command::Motion(Right),
    );
    keymap.bind(
        normal,
        vec![Key::char('g'), Key::char('g')],
        Command::Motion(FileStart),
    );
    keymap.bind(normal, vec![Key::char('G')], Command::Motion(FileEnd));
    keymap.bind(normal, vec![Key::char('0')], Command::Motion(LineStart));
    keymap.bind(normal, vec![Key::char('$')], Command::Motion(LineEnd));

    // Word motions. Uppercase keys arrive as distinct chars (SHIFT is folded
    // into the character), so they need their own bindings.
    keymap.bind(normal, vec![Key::char('w')], Command::Motion(NextWordStart));
    keymap.bind(normal, vec![Key::char('b')], Command::Motion(PrevWordStart));
    keymap.bind(normal, vec![Key::char('e')], Command::Motion(WordEnd));
    keymap.bind(
        normal,
        vec![Key::char('W')],
        Command::Motion(NextBigWordStart),
    );
    keymap.bind(
        normal,
        vec![Key::char('B')],
        Command::Motion(PrevBigWordStart),
    );
    keymap.bind(normal, vec![Key::char('E')], Command::Motion(BigWordEnd));

    let insert = InputMode::Insert;
    keymap.bind(
        insert,
        vec![Key::plain(KeyCode::Left)],
        Command::Motion(Left),
    );
    keymap.bind(
        insert,
        vec![Key::plain(KeyCode::Down)],
        Command::Motion(Down),
    );
    keymap.bind(insert, vec![Key::plain(KeyCode::Up)], Command::Motion(Up));
    keymap.bind(
        insert,
        vec![Key::plain(KeyCode::Right)],
        Command::Motion(Right),
    );

    keymap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_motions_are_bound_in_normal_mode() {
        let keymap = Keymap::default();
        let normal = InputMode::Normal;

        assert_eq!(
            keymap.lookup(normal, &[Key::char('w')]),
            Some(&Command::Motion(NextWordStart))
        );
        assert_eq!(
            keymap.lookup(normal, &[Key::char('b')]),
            Some(&Command::Motion(PrevWordStart))
        );
        assert_eq!(
            keymap.lookup(normal, &[Key::char('e')]),
            Some(&Command::Motion(WordEnd))
        );
    }

    #[test]
    fn shifted_word_motions_are_bound_separately() {
        let keymap = Keymap::default();
        let normal = InputMode::Normal;

        assert_eq!(
            keymap.lookup(normal, &[Key::char('W')]),
            Some(&Command::Motion(NextBigWordStart))
        );
        assert_eq!(
            keymap.lookup(normal, &[Key::char('B')]),
            Some(&Command::Motion(PrevBigWordStart))
        );
        assert_eq!(
            keymap.lookup(normal, &[Key::char('E')]),
            Some(&Command::Motion(BigWordEnd))
        );
    }

    #[test]
    fn word_motions_are_not_bound_in_insert_mode() {
        let keymap = Keymap::default();
        assert_eq!(keymap.lookup(InputMode::Insert, &[Key::char('w')]), None);
    }
}
