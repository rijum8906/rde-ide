//! Mode transitions.
//!
//! Besides switching `Input::mode`, this handles transitions that imply an
//! edit (`a`, `A`, `I`, `o`, `O`): the cursor is moved and the newline is
//! inserted before insert mode becomes active.

use domain::{error::DomainResult, workspace::WorkspaceState};
use editor_core::{
    command::ModeCommand::{self},
    input::{Input, mode::InputMode},
};
use tui::ui::Ui;

use crate::mutation::{
    goto::{goto_line_end, move_cursor_right, move_cursor_up},
    text::insert_new_line,
};

pub fn handle_mode_command(
    ws: &mut WorkspaceState,
    input: &mut Input,
    ui: &mut Ui,
    cmd: ModeCommand,
) -> DomainResult<()> {
    match cmd {
        ModeCommand::Enter(mode) => input.mode = mode,
        ModeCommand::ExitToNormal => input.mode = InputMode::Normal,
        ModeCommand::InsertBefore => todo!(),
        ModeCommand::InsertAfter => {
            move_cursor_right(ws, &mut ui.widgets.editor_panel.state);
            input.mode = InputMode::Insert;
        }
        ModeCommand::InsertAtLineStart => {
            ui.widgets.editor_panel.state.move_home();
            input.mode = InputMode::Insert;
        }
        ModeCommand::InsertAtLineEnd => {
            goto_line_end(ws, &mut ui.widgets.editor_panel.state);
            input.mode = InputMode::Insert;
        }
        ModeCommand::InsertNewLineBelow => {
            goto_line_end(ws, &mut ui.widgets.editor_panel.state);
            insert_new_line(ws, &mut ui.widgets);
            input.mode = InputMode::Insert;
        }
        ModeCommand::InsertNewLineAbove => {
            if ui.widgets.editor_panel.state.cursor_line == 0 {
                ui.widgets.editor_panel.state.cursor_line = 0;
                ui.widgets.editor_panel.state.cursor_col = 0;
                insert_new_line(ws, &mut ui.widgets);
                move_cursor_up(ws, &mut ui.widgets.editor_panel.state);
            } else {
                move_cursor_up(ws, &mut ui.widgets.editor_panel.state);
                goto_line_end(ws, &mut ui.widgets.editor_panel.state);
                insert_new_line(ws, &mut ui.widgets);
            }
            input.mode = InputMode::Insert;
        }
        ModeCommand::OpenBelow => todo!(),
        ModeCommand::OpenAbove => todo!(),
    }

    Ok(())
}
