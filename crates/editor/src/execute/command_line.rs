//! Command-line (`:`) editing commands.

use editor_core::command::CommandLineCommand;

pub fn handle_command_line_command(cmd: CommandLineCommand) {
    match cmd {
        CommandLineCommand::Open => todo!(),
        CommandLineCommand::Submit => todo!(),
        CommandLineCommand::Cancel => todo!(),
        CommandLineCommand::Push(_) => todo!(),
        CommandLineCommand::Pop => todo!(),
        CommandLineCommand::Clear => todo!(),
        CommandLineCommand::CursorLeft => todo!(),
        CommandLineCommand::CursorRight => todo!(),
        CommandLineCommand::CursorHome => todo!(),
        CommandLineCommand::CursorEnd => todo!(),
        CommandLineCommand::HistoryPrev => todo!(),
        CommandLineCommand::HistoryNext => todo!(),
    }
}
