use crate::{
    command::command::{Command, ModeCommand},
    input::{
        key::{Key, KeyCode, KeyExt},
        keymap::Keymap,
        mode::Mode,
    },
};

pub(super) fn default_mode_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = Mode::Normal;
    keymap.bind(
        normal,
        vec![Key::char('i')],
        Command::Mode(ModeCommand::Enter(Mode::Insert)),
    );
    keymap.bind(
        normal,
        vec![Key::char('v')],
        Command::Mode(ModeCommand::Enter(Mode::Visual)),
    );
    keymap.bind(
        normal,
        vec![Key::char('a')],
        Command::Mode(ModeCommand::InsertAfter),
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
        Command::Mode(ModeCommand::Enter(Mode::Command)),
    );

    for mode in [Mode::Insert, Mode::Visual] {
        keymap.bind(
            mode,
            vec![Key::plain(KeyCode::Esc)],
            Command::Mode(ModeCommand::ExitToNormal),
        );
    }
    keymap.bind(
        Mode::Insert,
        vec![Key::ctrl('j')],
        Command::Mode(ModeCommand::ExitToNormal),
    );
    keymap.bind(
        Mode::Insert,
        vec![Key::ctrl('k')],
        Command::Mode(ModeCommand::ExitToNormal),
    );

    keymap
}
