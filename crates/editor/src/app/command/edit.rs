use domain::error::DomainResult;
use editor_core::command::command::EditCommand;

use crate::app::{
    execute::{
        edit::delete_line,
        text::{delete, insert_char, insert_new_line},
    },
    App,
};

impl App {
    pub fn handle_edit_command(&mut self, cmd: EditCommand) -> DomainResult<()> {
        match cmd {
            EditCommand::InsertChar(c) => {
                insert_char(&mut self.workspace, &mut self.widget_states, c)
            }
            EditCommand::InsertNewline => {
                insert_new_line(&mut self.workspace, &mut self.widget_states);
            }
            EditCommand::InsertTab => todo!(),
            EditCommand::DeleteChar => todo!(),
            EditCommand::Backspace => delete(&mut self.workspace, &mut self.widget_states),
            EditCommand::DeleteWord => todo!(),
            EditCommand::DeleteToLineEnd => todo!(),
            EditCommand::DeleteLine => {
                delete_line(&mut self.workspace, &mut self.widget_states.editor_panel)?;
            }
            EditCommand::DeleteToFileEnd => todo!(),
            EditCommand::DeleteMotion(motion) => {}
            EditCommand::ChangeMotion(motion) => todo!(),
            EditCommand::YankMotion(motion) => todo!(),
            EditCommand::Yank => todo!(),
            EditCommand::Paste => todo!(),
            EditCommand::PasteBefore => todo!(),
            EditCommand::Undo => todo!(),
            EditCommand::Redo => todo!(),
            EditCommand::Repeat => todo!(),
        }

        Ok(())
    }
}
