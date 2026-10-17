use domain::workspace::WorkspaceState;
use tui::state::editor_panel::EditorState;

pub fn goto_file_end(workspace: &mut WorkspaceState, editor_state: &mut EditorState) {
    if let Some(buffer) = workspace.active_buffer() {
        let last_line_idx = buffer.line_count().saturating_sub(1);
        editor_state.move_to_file_end(buffer.line_char_len(last_line_idx));
    }
}

pub fn goto_line_end(workspace: &mut WorkspaceState, editor_state: &mut EditorState) {
    if let Some(buffer) = workspace.active_buffer() {
        editor_state.move_end(buffer.line_char_len(editor_state.cursor_line));
    }
}

/// Move the cursor up by one line, clamping the column to the new line's
/// length so multi-byte and short lines behave correctly.
pub fn move_cursor_up(workspace: &WorkspaceState, editor_state: &mut EditorState) {
    if editor_state.cursor_line == 0 {
        return;
    }
    let target_line = editor_state.cursor_line - 1;
    let Some(buffer) = workspace.active_buffer() else {
        return;
    };
    editor_state.move_up(buffer.line_char_len(target_line));
}

/// Move the cursor down by one line, clamping the column to the new line's
/// length.
pub fn move_cursor_down(workspace: &WorkspaceState, editor_state: &mut EditorState) {
    let Some(buffer) = workspace.active_buffer() else {
        return;
    };
    if editor_state.cursor_line + 1 >= buffer.line_count() {
        return;
    }
    let target_line = editor_state.cursor_line + 1;
    editor_state.move_down(buffer.line_char_len(target_line));
}

/// Move the cursor one character to the left. Stops at column 0.
pub fn move_cursor_left(workspace: &WorkspaceState, editor_state: &mut EditorState) {
    if workspace.active_buffer().is_none() {
        return;
    }
    editor_state.move_left();
}

/// Move the cursor one character to the right. Stops at the end of the
/// current line.
pub fn move_cursor_right(workspace: &WorkspaceState, editor_state: &mut EditorState) {
    let Some(buffer) = workspace.active_buffer() else {
        return;
    };
    let line_len = buffer.line_char_len(editor_state.cursor_line);
    editor_state.move_right(line_len);
}
