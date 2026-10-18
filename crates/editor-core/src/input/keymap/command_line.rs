use crate::{
    command::command::{Command, CommandLineCommand},
    input::{
        key::{Key, KeyCode, KeyExt},
        keymap::Keymap,
        mode::Mode,
    },
};

pub(super) fn default_command_line_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let command = Mode::Command;
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Esc)],
        Command::CommandLine(CommandLineCommand::Cancel),
    );
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Enter)],
        Command::CommandLine(CommandLineCommand::Submit),
    );
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Backspace)],
        Command::CommandLine(CommandLineCommand::Pop),
    );

    keymap
}
