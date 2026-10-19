//! Cursor movement helpers that need buffer contents.
//!
//! Unlike the pure `EditorPanelState` movers, these functions read line
//! lengths and the buffer bounds, so moving up/down can clamp the column and
//! moving to the end of the file lands on the real last line.

use domain::workspace::WorkspaceState;
use tui::state::editor_panel::EditorPanelState;

pub fn goto_file_end(workspace: &mut WorkspaceState, eps: &mut EditorPanelState) {
    if let Some(buffer) = workspace.active_buffer() {
        let last_line_idx = buffer.line_count().saturating_sub(1);
        eps.move_to_file_end(buffer.line_char_len(last_line_idx));
    }
}

pub fn goto_line_end(workspace: &mut WorkspaceState, eps: &mut EditorPanelState) {
    if let Some(buffer) = workspace.active_buffer() {
        eps.move_end(buffer.line_char_len(eps.cursor_line));
    }
}

/// Move the cursor up by one line, clamping the column to the new line's
/// length so multi-byte and short lines behave correctly.
pub fn move_cursor_up(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    if eps.cursor_line == 0 {
        return;
    }
    let target_line = eps.cursor_line - 1;
    let Some(buffer) = workspace.active_buffer() else {
        return;
    };
    eps.move_up(buffer.line_char_len(target_line));
}

/// Move the cursor down by one line, clamping the column to the new line's
/// length.
pub fn move_cursor_down(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    let Some(buffer) = workspace.active_buffer() else {
        return;
    };
    if eps.cursor_line + 1 >= buffer.line_count() {
        return;
    }
    let target_line = eps.cursor_line + 1;
    eps.move_down(buffer.line_char_len(target_line));
}

/// Move the cursor one character to the left. Stops at column 0.
pub fn move_cursor_left(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    if workspace.active_buffer().is_none() {
        return;
    }
    eps.move_left();
}

/// Move the cursor one character to the right. Stops at the end of the
/// current line.
pub fn move_cursor_right(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    let Some(buffer) = workspace.active_buffer() else {
        return;
    };
    let line_len = buffer.line_char_len(eps.cursor_line);
    eps.move_right(line_len);
}
