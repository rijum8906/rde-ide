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
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Left)],
        Command::CommandLine(CommandLineCommand::CursorLeft),
    );
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Right)],
        Command::CommandLine(CommandLineCommand::CursorRight),
    );
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Home)],
        Command::CommandLine(CommandLineCommand::CursorHome),
    );
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::End)],
        Command::CommandLine(CommandLineCommand::CursorEnd),
    );
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Up)],
        Command::CommandLine(CommandLineCommand::HistoryPrev),
    );
    keymap.bind(
        command,
        vec![Key::plain(KeyCode::Down)],
        Command::CommandLine(CommandLineCommand::HistoryNext),
    );
    keymap.bind(
        command,
        vec![Key::ctrl('u')],
        Command::CommandLine(CommandLineCommand::Clear),
    );

    keymap
}
