use domain::error::DomainResult;
use editor_core::{
    command::command::ModeCommand::{self},
    input::mode::Mode,
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
            ModeCommand::ExitToNormal => self.set_mode(Mode::Normal),
            ModeCommand::InsertBefore => todo!(),
            ModeCommand::InsertAfter => {
                move_cursor_right(&self.workspace, &mut self.widget_states.editor_panel);
                self.set_mode(Mode::Insert);
            }
            ModeCommand::InsertAtLineStart => {
                self.widget_states.editor_panel.move_home();
                self.set_mode(Mode::Insert);
            }
            ModeCommand::InsertAtLineEnd => {
                goto_line_end(&mut self.workspace, &mut self.widget_states.editor_panel);
                self.set_mode(Mode::Insert);
            }
            ModeCommand::InsertNewLineBelow => {
                goto_line_end(&mut self.workspace, &mut self.widget_states.editor_panel);
                insert_new_line(&mut self.workspace, &mut self.widget_states);
                self.set_mode(Mode::Insert);
            }
            ModeCommand::InsertNewLineAbove => {
                if self.widget_states.editor_panel.cursor_line == 0 {
                    self.widget_states.editor_panel.cursor_line = 0;
                    self.widget_states.editor_panel.cursor_col = 0;
                    insert_new_line(&mut self.workspace, &mut self.widget_states);
                    move_cursor_up(&self.workspace, &mut self.widget_states.editor_panel);
                } else {
                    move_cursor_up(&self.workspace, &mut self.widget_states.editor_panel);
                    goto_line_end(&mut self.workspace, &mut self.widget_states.editor_panel);
                    insert_new_line(&mut self.workspace, &mut self.widget_states);
                }
                self.set_mode(Mode::Insert);
            }
            ModeCommand::OpenBelow => todo!(),
            ModeCommand::OpenAbove => todo!(),
        }

        Ok(())
    }
}
