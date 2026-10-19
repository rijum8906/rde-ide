use domain::{error::DomainResult, workspace::WorkspaceState};
use editor_core::command::EditCommand;
use tui::ui::Ui;

use crate::mutation::{
    edit::delete_line,
    text::{delete, insert_char, insert_new_line},
};

pub fn handle_edit_command(
    ws: &mut WorkspaceState,
    ui: &mut Ui,
    cmd: EditCommand,
) -> DomainResult<()> {
    match cmd {
        EditCommand::InsertChar(c) => insert_char(ws, &mut ui.widgets, c),
        EditCommand::InsertNewline => {
            insert_new_line(ws, &mut ui.widgets);
        }
        EditCommand::InsertTab => todo!(),
        EditCommand::DeleteChar => todo!(),
        EditCommand::Backspace => delete(ws, &mut ui.widgets),
        EditCommand::DeleteWord => todo!(),
        EditCommand::DeleteToLineEnd => todo!(),
        EditCommand::DeleteLine => {
            delete_line(ws, &mut ui.widgets.editor_panel.state)?;
        }
        EditCommand::DeleteToFileEnd => todo!(),
        EditCommand::DeleteMotion(_motion) => {}
        EditCommand::ChangeMotion(_motion) => todo!(),
        EditCommand::YankMotion(_motion) => todo!(),
        EditCommand::Yank => todo!(),
        EditCommand::Paste => todo!(),
        EditCommand::PasteBefore => todo!(),
        EditCommand::Undo => todo!(),
        EditCommand::Redo => todo!(),
        EditCommand::Repeat => todo!(),
    }

    Ok(())
}
