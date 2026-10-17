//! Editor Input Handler
//!
//! Manages text insertions (`insert_char`, `insert_new_line`) and deletions (`delete`)
//! for the text editor panel. It translates 2D UI cursor positions (`cursor_line` and
//! `cursor_col`) into 1D character offsets, converts them to UTF-8 byte offsets,
//! and safely updates both the underlying buffer and the cursor state.

use std::ops::Range;

use domain::{
    error::{DomainError, DomainResult},
    workspace::WorkspaceState,
};
use tui::widgets::WidgetStates;

/// Inserts a standard character at the current cursor position.
///
/// On success, advances the horizontal cursor column (`cursor_col`) by 1.
pub fn insert_char(workspace: &mut WorkspaceState, widget_states: &mut WidgetStates, c: char) {
    if insert(workspace, widget_states, c).is_ok() {
        widget_states.editor_panel.cursor_col += 1;
    }
}

/// Inserts a newline character (`\n`) at the current cursor position.
///
/// On success, moves the cursor down to the start of the next line
/// (`cursor_line += 1`, `cursor_col = 0`).
pub fn insert_new_line(workspace: &mut WorkspaceState, widget_states: &mut WidgetStates) {
    if insert(workspace, widget_states, '\n').is_ok() {
        widget_states.editor_panel.cursor_line += 1;
        widget_states.editor_panel.cursor_col = 0;
    }
}

pub fn calculate_byte_offset(
    workspace: &mut WorkspaceState,
    ws: &mut WidgetStates,
) -> DomainResult<usize> {
    if let Some(buffer) = workspace.active_buffer_mut() {
        let mut char_offset = 0;

        // Accumulate characters and newlines from all lines above the cursor
        for i in 0..ws.editor_panel.cursor_line {
            char_offset += buffer.line_char_len(i) + 1;
        }

        // Add horizontal column offset within the current line
        char_offset += ws.editor_panel.cursor_col;

        // Convert the logical character offset into a absolute byte offset
        let byte_offset = buffer.char_to_byte_offset(char_offset);
        return Ok(byte_offset);
    }

    Err(DomainError::NoActiveBuffer)
}

/// Core helper function that handles inserting any character into the active buffer.
///
/// # How coordinate translation works:
/// 1. **2D to 1D Character Offset**: To find where the cursor points in the linear document,
///    we sum up the character lengths of all previous lines. We add `+ 1` to account for
///    the hidden newline character (`\n`) that separates each line. We then add the
///    current `cursor_col`.
/// 2. **Character to Byte Offset**: Because Rust strings use variable-length UTF-8 encoding,
///    character indices do not equal byte indices. We pass the logical character offset
///    into the buffer to get the exact safe byte offset required by the Piece Table.
pub(super) fn insert(
    workspace: &mut WorkspaceState,
    widget_states: &mut WidgetStates,
    c: char,
) -> DomainResult<()> {
    if let Some(buffer) = workspace.active_buffer_mut() {
        let mut char_offset = 0;

        // Accumulate characters and newlines from all lines above the cursor
        for i in 0..widget_states.editor_panel.cursor_line {
            char_offset += buffer.line_char_len(i) + 1;
        }

        // Add horizontal column offset within the current line
        char_offset += widget_states.editor_panel.cursor_col;

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
pub fn delete(workspace: &mut WorkspaceState, widget_states: &mut WidgetStates) {
    // Guard: Cannot delete if we are at the very top-left of the document
    if widget_states.editor_panel.cursor_line == 0 && widget_states.editor_panel.cursor_col == 0 {
        return;
    }

    if let Some(buffer) = workspace.active_buffer_mut() {
        let mut char_offset = 0;

        // Calculate the linear character position of the cursor
        for i in 0..widget_states.editor_panel.cursor_line {
            char_offset += buffer.line_char_len(i) + 1;
        }
        char_offset += widget_states.editor_panel.cursor_col;

        if char_offset == 0 {
            return;
        }

        // Determine byte boundaries for the single character/newline to be deleted
        let end_byte = buffer.char_to_byte_offset(char_offset);
        let start_byte = buffer.char_to_byte_offset(char_offset - 1);

        // Pre-fetch the length of the line above us in case we merge lines across a newline
        let prev_line_len = if widget_states.editor_panel.cursor_line > 0 {
            buffer.line_char_len(widget_states.editor_panel.cursor_line - 1)
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
            if widget_states.editor_panel.cursor_col > 0 {
                // Case A: Normal backspace within the same line
                widget_states.editor_panel.cursor_col -= 1;
            } else if widget_states.editor_panel.cursor_line > 0 {
                // Case B: Backspacing at the start of a line (deleting a newline)
                widget_states.editor_panel.cursor_line -= 1;
                widget_states.editor_panel.cursor_col = prev_line_len;
            }
        }
    }
}
