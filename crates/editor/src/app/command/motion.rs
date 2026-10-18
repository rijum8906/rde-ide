use editor_core::command::MotionCommand;

use crate::app::{
    App,
    execute::goto::{
        goto_file_end, goto_line_end, move_cursor_down, move_cursor_left, move_cursor_right,
        move_cursor_up,
    },
};

impl App {
    pub fn handle_motion_command(&mut self, cmd: MotionCommand) {
        match cmd {
            MotionCommand::Left => {
                move_cursor_left(&self.workspace, &mut self.ui.widgets.editor_panel.state)
            }
            MotionCommand::Right => {
                move_cursor_right(&self.workspace, &mut self.ui.widgets.editor_panel.state)
            }
            MotionCommand::Up => {
                move_cursor_up(&self.workspace, &mut self.ui.widgets.editor_panel.state)
            }
            MotionCommand::Down => {
                move_cursor_down(&self.workspace, &mut self.ui.widgets.editor_panel.state)
            }
            MotionCommand::LineStart => self.ui.widgets.editor_panel.state.move_home(),
            MotionCommand::LineEnd => {
                goto_line_end(&mut self.workspace, &mut self.ui.widgets.editor_panel.state);
            }
            MotionCommand::FirstNonBlank => todo!(),
            MotionCommand::LineNumber(_) => todo!(),
            MotionCommand::FileStart => self.ui.widgets.editor_panel.state.move_to_file_start(),
            MotionCommand::FileEnd => {
                goto_file_end(&mut self.workspace, &mut self.ui.widgets.editor_panel.state)
            }
            MotionCommand::NextWordStart => todo!(),
            MotionCommand::PrevWordStart => todo!(),
            MotionCommand::WordEnd => todo!(),
            MotionCommand::NextBigWordStart => todo!(),
            MotionCommand::PrevBigWordStart => todo!(),
            MotionCommand::NextParagraphStart => todo!(),
            MotionCommand::PrevParagraphStart => todo!(),
            MotionCommand::PageUp => todo!(),
            MotionCommand::PageDown => todo!(),
            MotionCommand::HalfPageUp => todo!(),
            MotionCommand::HalfPageDown => todo!(),
            MotionCommand::ScreenTop => todo!(),
            MotionCommand::ScreenMiddle => todo!(),
            MotionCommand::ScreenBottom => todo!(),
            MotionCommand::MatchBracket => todo!(),
        }
    }
}
