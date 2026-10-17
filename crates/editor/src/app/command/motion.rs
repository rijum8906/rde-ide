use editor_core::command::command::Motion;

use crate::app::{
    App,
    execute::goto::{
        goto_file_end, goto_line_end, move_cursor_down, move_cursor_left, move_cursor_right,
        move_cursor_up,
    },
};

impl App {
    pub fn handle_motion_command(&mut self, cmd: Motion) {
        match cmd {
            Motion::Left => move_cursor_left(&self.workspace, &mut self.widget_states.editor_panel),
            Motion::Right => {
                move_cursor_right(&self.workspace, &mut self.widget_states.editor_panel)
            }
            Motion::Up => move_cursor_up(&self.workspace, &mut self.widget_states.editor_panel),
            Motion::Down => move_cursor_down(&self.workspace, &mut self.widget_states.editor_panel),
            Motion::LineStart => self.widget_states.editor_panel.move_home(),
            Motion::LineEnd => {
                goto_line_end(&mut self.workspace, &mut self.widget_states.editor_panel)
            }
            Motion::FirstNonBlank => todo!(),
            Motion::LineNumber(_) => todo!(),
            Motion::FileStart => self.widget_states.editor_panel.move_to_file_start(),
            Motion::FileEnd => {
                goto_file_end(&mut self.workspace, &mut self.widget_states.editor_panel)
            }
            Motion::NextWordStart => todo!(),
            Motion::PrevWordStart => todo!(),
            Motion::WordEnd => todo!(),
            Motion::NextBigWordStart => todo!(),
            Motion::PrevBigWordStart => todo!(),
            Motion::NextParagraphStart => todo!(),
            Motion::PrevParagraphStart => todo!(),
            Motion::PageUp => todo!(),
            Motion::PageDown => todo!(),
            Motion::HalfPageUp => todo!(),
            Motion::HalfPageDown => todo!(),
            Motion::ScreenTop => todo!(),
            Motion::ScreenMiddle => todo!(),
            Motion::ScreenBottom => todo!(),
            Motion::MatchBracket => todo!(),
        }
    }
}
