//! Cursor movement helpers that need buffer contents.
//!
//! Unlike the pure `EditorPanelState` movers, these functions read line
//! lengths and the buffer bounds, so moving up/down can clamp the column and
//! moving to the end of the file lands on the real last line.

use domain::{buffer::Buffer, workspace::WorkspaceState};
use tui::state::editor_panel::EditorPanelState;

use super::text::cursor_char_offset;

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

/// Move to the start of the next small word (`w`).
pub fn move_word_start(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    apply_word_motion(workspace, eps, Buffer::next_word_start);
}

/// Move to the last character of the next small word (`e`).
pub fn move_word_end(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    apply_word_motion(workspace, eps, Buffer::next_word_end);
}

/// Move to the start of the previous small word (`b`).
pub fn move_word_backward(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    apply_word_motion(workspace, eps, Buffer::prev_word_start);
}

/// Move to the start of the next big word (`W`).
pub fn move_big_word_start(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    apply_word_motion(workspace, eps, Buffer::next_big_word_start);
}

/// Move to the last character of the next big word (`E`).
pub fn move_big_word_end(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    apply_word_motion(workspace, eps, Buffer::next_big_word_end);
}

/// Move to the start of the previous big word (`B`).
pub fn move_big_word_backward(workspace: &WorkspaceState, eps: &mut EditorPanelState) {
    apply_word_motion(workspace, eps, Buffer::prev_big_word_start);
}

/// Run a byte-offset word motion and translate the target back into a
/// character-based cursor position. A `None` target (no word that way) leaves
/// the cursor untouched.
fn apply_word_motion(
    workspace: &WorkspaceState,
    eps: &mut EditorPanelState,
    motion: fn(&Buffer, usize) -> Option<usize>,
) {
    let Some(buffer) = workspace.active_buffer() else {
        return;
    };
    let char_offset = cursor_char_offset(buffer, eps.cursor_line, eps.cursor_col);
    let byte_offset = buffer.char_to_byte_offset(char_offset);

    let Some(target) = motion(buffer, byte_offset) else {
        return;
    };
    if let Some((line, col)) = buffer.line_col_at(target) {
        eps.cursor_line = line;
        eps.cursor_col = col;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{buffer::Buffer, workspace::WorkspaceState};
    use tui::state::editor_panel::EditorPanelState;

    fn setup(text: &str) -> (WorkspaceState, EditorPanelState) {
        let mut workspace = WorkspaceState::new(".");
        workspace.push_buffer(Buffer::with_contents("test.txt", text));
        (workspace, EditorPanelState::default())
    }

    fn cursor(eps: &EditorPanelState) -> (usize, usize) {
        (eps.cursor_line, eps.cursor_col)
    }

    #[test]
    fn word_forward_moves_across_lines() {
        let (workspace, mut eps) = setup("one two\nthree four");

        move_word_start(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 4));
        move_word_start(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (1, 0));
        move_word_start(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (1, 6));

        // No word follows: the cursor must stay put.
        move_word_start(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (1, 6));
    }

    #[test]
    fn word_end_moves_to_the_last_character() {
        let (workspace, mut eps) = setup("one two\nthree four");

        move_word_end(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 2));
        move_word_end(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 6));
        move_word_end(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (1, 4));
        move_word_end(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (1, 9));
    }

    #[test]
    fn word_backward_moves_to_word_starts() {
        let (workspace, mut eps) = setup("one two\nthree four");
        eps.cursor_line = 1;
        eps.cursor_col = 9;

        move_word_backward(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (1, 6));
        move_word_backward(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (1, 0));
        move_word_backward(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 4));
        move_word_backward(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 0));

        // Already at the start of the document.
        move_word_backward(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 0));
    }

    #[test]
    fn big_word_motions_treat_hyphens_as_word_chars() {
        let (workspace, mut eps) = setup("hello-world foo");

        move_word_start(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 6)); // after "hello"

        let (workspace, mut eps) = setup("hello-world foo");
        move_big_word_start(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 12)); // after "hello-world"

        move_big_word_backward(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 0));
    }

    #[test]
    fn multibyte_motions_report_character_columns() {
        let (workspace, mut eps) = setup("héllo wörld");

        move_word_start(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 6)); // char column, not byte 7
        move_word_end(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 10)); // 'd'

        eps.cursor_col = 0;
        move_word_end(&workspace, &mut eps);
        assert_eq!(cursor(&eps), (0, 4)); // 'o'
    }
}
