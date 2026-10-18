use domain::error::DomainResult;
use editor_core::{
    command::ModeCommand::{self},
    input::mode::InputMode,
};

use crate::app::{
    App,
    execute::{
        goto::{goto_line_end, move_cursor_right, move_cursor_up},
        text::insert_new_line,
    },
};

impl App {
    pub fn handle_mode_command(&mut self, cmd: ModeCommand) -> DomainResult<()> {
        match cmd {
            ModeCommand::Enter(mode) => self.set_mode(mode),
            ModeCommand::ExitToNormal => self.set_mode(InputMode::Normal),
            ModeCommand::InsertBefore => todo!(),
            ModeCommand::InsertAfter => {
                move_cursor_right(&self.workspace, &mut self.ui.widgets.editor_panel.state);
                self.set_mode(InputMode::Insert);
            }
            ModeCommand::InsertAtLineStart => {
                self.ui.widgets.editor_panel.state.move_home();
                self.set_mode(InputMode::Insert);
            }
            ModeCommand::InsertAtLineEnd => {
                goto_line_end(&mut self.workspace, &mut self.ui.widgets.editor_panel.state);
                self.set_mode(InputMode::Insert);
            }
            ModeCommand::InsertNewLineBelow => {
                goto_line_end(&mut self.workspace, &mut self.ui.widgets.editor_panel.state);
                insert_new_line(&mut self.workspace, &mut self.ui.widgets);
                self.set_mode(InputMode::Insert);
            }
            ModeCommand::InsertNewLineAbove => {
                if self.ui.widgets.editor_panel.state.cursor_line == 0 {
                    self.ui.widgets.editor_panel.state.cursor_line = 0;
                    self.ui.widgets.editor_panel.state.cursor_col = 0;
                    insert_new_line(&mut self.workspace, &mut self.ui.widgets);
                    move_cursor_up(&self.workspace, &mut self.ui.widgets.editor_panel.state);
                } else {
                    move_cursor_up(&self.workspace, &mut self.ui.widgets.editor_panel.state);
                    goto_line_end(&mut self.workspace, &mut self.ui.widgets.editor_panel.state);
                    insert_new_line(&mut self.workspace, &mut self.ui.widgets);
                }
                self.set_mode(InputMode::Insert);
            }
            ModeCommand::OpenBelow => todo!(),
            ModeCommand::OpenAbove => todo!(),
        }

        Ok(())
    }
}
