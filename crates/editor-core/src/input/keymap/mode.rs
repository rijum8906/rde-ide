//! Default keybindings for mode transitions.

use crate::{
    command::{Command, ModeCommand},
    input::{
        key::{Key, KeyCode},
        keymap::Keymap,
        mode::InputMode,
    },
};

pub(super) fn default_mode_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = InputMode::Normal;
    keymap.bind(
        normal,
        vec![Key::char('i')],
        Command::Mode(ModeCommand::Enter(InputMode::Insert)),
    );
    keymap.bind(
        normal,
        vec![Key::char('a')],
        Command::Mode(ModeCommand::InsertAfter),
    );
    keymap.bind(
        normal,
        vec![Key::char('v')],
        Command::Mode(ModeCommand::Enter(InputMode::Visual)),
    );
    keymap.bind(
        normal,
        vec![Key::char('A')],
        Command::Mode(ModeCommand::InsertAtLineEnd),
    );
    keymap.bind(
        normal,
        vec![Key::char('I')],
        Command::Mode(ModeCommand::InsertAtLineStart),
    );
    keymap.bind(
        normal,
        vec![Key::char('o')],
        Command::Mode(ModeCommand::InsertNewLineBelow),
    );
    keymap.bind(
        normal,
        vec![Key::char('O')],
        Command::Mode(ModeCommand::InsertNewLineAbove),
    );
    keymap.bind(
        normal,
        vec![Key::char(':')],
        Command::Mode(ModeCommand::Enter(InputMode::Command)),
    );

    for mode in [InputMode::Insert, InputMode::Visual] {
        keymap.bind(
            mode,
            vec![Key::plain(KeyCode::Esc)],
            Command::Mode(ModeCommand::ExitToNormal),
        );
    }
    keymap.bind(
        InputMode::Insert,
        vec![Key::ctrl('j')],
        Command::Mode(ModeCommand::ExitToNormal),
    );
    keymap.bind(
        InputMode::Insert,
        vec![Key::ctrl('k')],
        Command::Mode(ModeCommand::ExitToNormal),
    );

    keymap
}
