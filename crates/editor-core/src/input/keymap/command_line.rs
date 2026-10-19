//! Default keybindings for `:` command-line mode.

use crate::{
    command::{Command, CommandLineCommand},
    input::{
        key::{Key, KeyCode},
        keymap::Keymap,
        mode::InputMode,
    },
};

pub(super) fn default_command_line_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let command = InputMode::Command;
    keymap.bind(
        command,
        vec![Key::esc()],
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
