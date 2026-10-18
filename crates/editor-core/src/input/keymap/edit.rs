use crate::{
    command::command::{Command, EditCommand},
    input::{
        key::{Key, KeyCode, KeyExt},
        keymap::Keymap,
        mode::Mode,
    },
};

pub(super) fn default_edit_keymap(keymap: &mut Keymap) -> &mut Keymap {
    keymap.bind(
        Mode::Normal,
        vec![Key::char('d'), Key::char('d')],
        Command::Edit(EditCommand::DeleteLine),
    );
    keymap.bind(
        Mode::Insert,
        vec![Key::plain(KeyCode::Enter)],
        Command::Edit(EditCommand::InsertNewline),
    );
    keymap.bind(
        Mode::Insert,
        vec![Key::plain(KeyCode::Backspace)],
        Command::Edit(EditCommand::Backspace),
    );

    keymap
}
