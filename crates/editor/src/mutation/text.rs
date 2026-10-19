//! Text insertion and deletion.
//!
//! Implements `insert_char`, `insert_new_line`, and `delete` for the editor
//! panel. Cursor positions are 2D and character-based, while the piece table
//! works in UTF-8 byte offsets, so every edit goes through the same
//! translation: `(cursor_line, cursor_col) -> char offset -> byte offset`.
//!
//! The buffer and the cursor state are always updated together, so they cannot
//! drift apart after an edit.

use std::ops::Range;

use domain::{
    error::{DomainError, DomainResult},
    workspace::WorkspaceState,
};
use tui::widgets::Widgets;

/// Inserts a standard character at the current cursor position.
///
/// On success, advances the horizontal cursor column (`cursor_col`) by 1.
pub fn insert_char(workspace: &mut WorkspaceState, widget_states: &mut Widgets, c: char) {
    if insert(workspace, widget_states, c).is_ok() {
        widget_states.editor_panel.state.cursor_col += 1;
    }
}

/// Inserts a newline character (`\n`) at the current cursor position.
///
/// On success, moves the cursor down to the start of the next line
/// (`cursor_line += 1`, `cursor_col = 0`).
pub fn insert_new_line(workspace: &mut WorkspaceState, widget_states: &mut Widgets) {
    if insert(workspace, widget_states, '\n').is_ok() {
        widget_states.editor_panel.state.cursor_line += 1;
        widget_states.editor_panel.state.cursor_col = 0;
    }
}

/// Computes the linear character offset in the document for the cursor position.
pub fn cursor_char_offset(
    buffer: &domain::buffer::Buffer,
    cursor_line: usize,
    cursor_col: usize,
) -> usize {
    buffer
        .line_range(cursor_line)
        .map(|r| r.start + cursor_col)
        .unwrap_or(cursor_col)
}

/// Core helper function that handles inserting any character into the active buffer.
///
/// # How coordinate translation works:
/// 1. **2D to 1D Character Offset**: We use `cursor_char_offset` to find the linear
///    character position from the line's start offset plus `cursor_col`.
/// 2. **Character to Byte Offset**: Because Rust strings use variable-length UTF-8 encoding,
///    we convert the logical character offset into the exact byte offset required by the Piece Table.
pub(super) fn insert(
    workspace: &mut WorkspaceState,
    widget_states: &mut Widgets,
    c: char,
) -> DomainResult<()> {
    if let Some(buffer) = workspace.active_buffer_mut() {
        let char_offset = cursor_char_offset(
            buffer,
            widget_states.editor_panel.state.cursor_line,
            widget_states.editor_panel.state.cursor_col,
        );

        // Convert the logical character offset into a absolute byte offset
        let byte_offset = buffer.char_to_byte_offset(char_offset);

        return buffer.insert(byte_offset, &c.to_string());
    }

    Err(DomainError::NoActiveBuffer)
}

/// Deletes the character immediately preceding the current cursor (Backspace functionality).
///
/// # Handling Line Boundaries:
/// - If the cursor is mid-line (`cursor_col > 0`), it simply moves back one column.
/// - If the cursor is at the start of a line (`cursor_col == 0`) and not on the first line,
///   backspacing deletes the newline character of the previous line, merging the current
///   line onto the end of the previous one. The cursor must then hop up to the end
///   of that previous line.
pub fn delete(workspace: &mut WorkspaceState, widget_states: &mut Widgets) {
    // Guard: Cannot delete if we are at the very top-left of the document
    if widget_states.editor_panel.state.cursor_line == 0
        && widget_states.editor_panel.state.cursor_col == 0
    {
        return;
    }

    if let Some(buffer) = workspace.active_buffer_mut() {
        let char_offset = cursor_char_offset(
            buffer,
            widget_states.editor_panel.state.cursor_line,
            widget_states.editor_panel.state.cursor_col,
        );

        if char_offset == 0 {
            return;
        }

        // Determine byte boundaries for the single character/newline to be deleted
        let end_byte = buffer.char_to_byte_offset(char_offset);
        let start_byte = buffer.char_to_byte_offset(char_offset - 1);

        // Pre-fetch the length of the line above us in case we merge lines across a newline
        let prev_line_len = if widget_states.editor_panel.state.cursor_line > 0 {
            buffer.line_char_len(widget_states.editor_panel.state.cursor_line - 1)
        } else {
            0
        };

        // Perform the deletion in the buffer
        if buffer
            .delete(Range {
                start: start_byte,
                end: end_byte,
            })
            .is_ok()
        {
            // Update cursor position based on where the backspace occurred
            if widget_states.editor_panel.state.cursor_col > 0 {
                // Case A: Normal backspace within the same line
                widget_states.editor_panel.state.cursor_col -= 1;
            } else if widget_states.editor_panel.state.cursor_line > 0 {
                // Case B: Backspacing at the start of a line (deleting a newline)
                widget_states.editor_panel.state.cursor_line -= 1;
                widget_states.editor_panel.state.cursor_col = prev_line_len;
            }
        }
    }
}
