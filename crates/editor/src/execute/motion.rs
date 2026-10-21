//! Cursor motion commands.
//!
//! Every motion updates only [`EditorPanelState`](tui::state::editor_panel::EditorPanelState);
//! the buffer is read for line lengths but never modified.

use domain::workspace::WorkspaceState;
use editor_core::command::MotionCommand;
use tui::ui::Ui;

use crate::mutation::goto::{
    goto_file_end, goto_line_end, move_big_word_backward, move_big_word_end, move_big_word_start,
    move_cursor_down, move_cursor_left, move_cursor_right, move_cursor_up, move_word_backward,
    move_word_end, move_word_start,
};

pub fn handle_motion_command(workspace: &mut WorkspaceState, ui: &mut Ui, cmd: MotionCommand) {
    match cmd {
        MotionCommand::Left => move_cursor_left(workspace, &mut ui.widgets.editor_panel.state),
        MotionCommand::Right => move_cursor_right(workspace, &mut ui.widgets.editor_panel.state),
        MotionCommand::Up => move_cursor_up(workspace, &mut ui.widgets.editor_panel.state),
        MotionCommand::Down => move_cursor_down(workspace, &mut ui.widgets.editor_panel.state),
        MotionCommand::LineStart => ui.widgets.editor_panel.state.move_home(),
        MotionCommand::LineEnd => {
            goto_line_end(workspace, &mut ui.widgets.editor_panel.state);
        }
        MotionCommand::FirstNonBlank => todo!(),
        MotionCommand::LineNumber(_) => todo!(),
        MotionCommand::FileStart => ui.widgets.editor_panel.state.move_to_file_start(),
        MotionCommand::FileEnd => goto_file_end(workspace, &mut ui.widgets.editor_panel.state),
        MotionCommand::NextWordStart => {
            move_word_start(workspace, &mut ui.widgets.editor_panel.state)
        }
        MotionCommand::PrevWordStart => {
            move_word_backward(workspace, &mut ui.widgets.editor_panel.state)
        }
        MotionCommand::WordEnd => move_word_end(workspace, &mut ui.widgets.editor_panel.state),
        MotionCommand::NextBigWordStart => {
            move_big_word_start(workspace, &mut ui.widgets.editor_panel.state)
        }
        MotionCommand::PrevBigWordStart => {
            move_big_word_backward(workspace, &mut ui.widgets.editor_panel.state)
        }
        MotionCommand::BigWordEnd => {
            move_big_word_end(workspace, &mut ui.widgets.editor_panel.state)
        }
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
