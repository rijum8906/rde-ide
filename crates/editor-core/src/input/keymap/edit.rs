use crate::{
    command::{Command, EditCommand},
    input::{
        key::{Key, KeyCode},
        keymap::Keymap,
        mode::InputMode,
    },
};

pub(super) fn default_edit_keymap(keymap: &mut Keymap) -> &mut Keymap {
    keymap.bind(
        InputMode::Normal,
        vec![Key::char('d'), Key::char('d')],
        Command::Edit(EditCommand::DeleteLine),
    );
    keymap.bind(
        InputMode::Insert,
        vec![Key::plain(KeyCode::Enter)],
        Command::Edit(EditCommand::InsertNewline),
    );
    keymap.bind(
        InputMode::Insert,
        vec![Key::plain(KeyCode::Backspace)],
        Command::Edit(EditCommand::Backspace),
    );

    keymap
}
