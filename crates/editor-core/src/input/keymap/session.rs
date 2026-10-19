use crate::{
    command::{Command, SessionCommand},
    input::{
        key::{Key, KeyCode},
        keymap::Keymap,
        mode::InputMode,
    },
};

pub(super) fn default_session_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = InputMode::Normal;
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Esc)],
        Command::Session(SessionCommand::Quit),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('q')],
        Command::Session(SessionCommand::ForceQuit),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('c')],
        Command::Session(SessionCommand::ForceQuit),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('s')],
        Command::Session(SessionCommand::SaveBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('n')],
        Command::Session(SessionCommand::NewScratchBuffer),
    );

    keymap
}
