use domain::{
    error::{DomainError, DomainResult},
    workspace::WorkspaceState,
};
use std::ops::Range;
use tui::state::editor_panel::EditorState;
// Assuming EditorState is imported or defined in your crate

pub fn delete_line(wss: &mut WorkspaceState, eps: &mut EditorState) -> DomainResult<()> {
    if let Some(buffer) = wss.active_buffer_mut() {
        let line_count = buffer.line_count();
        if line_count == 0 {
            return Ok(());
        }

        // 1. Calculate the starting character offset of the target line
        let mut start_char = 0;
        for i in 0..eps.cursor_line {
            start_char += buffer.line_char_len(i) + 1; // +1 accounts for the newline char
        }

        let line_len = buffer.line_char_len(eps.cursor_line);

        // 2. Determine the character range to delete, including its newline boundary
        let (start_char, end_char) = if eps.cursor_line + 1 < line_count {
            // Case A: Not the last line. Delete the line contents plus its trailing newline.
            (start_char, start_char + line_len + 1)
        } else if eps.cursor_line > 0 {
            // Case B: The last line (and not the only line). Delete the preceding newline
            // and this line so we don't leave a trailing blank line at EOF.
            let prev_line_len = buffer.line_char_len(eps.cursor_line - 1);
            let prev_start = start_char - prev_line_len - 1;
            (prev_start + prev_line_len, start_char + line_len)
        } else {
            // Case C: The only line in the document. Just clear its contents.
            (start_char, start_char + line_len)
        };

        // 3. Convert logical character offsets into safe UTF-8 byte offsets
        let start_byte = buffer.char_to_byte_offset(start_char);
        let end_byte = buffer.char_to_byte_offset(end_char);

        // 4. Perform the deletion on the buffer
        if buffer
            .delete(Range {
                start: start_byte,
                end: end_byte,
            })
            .is_ok()
        {
            // Clamp cursor line and reset column to 0
            eps.cursor_line = eps.cursor_line.min(buffer.line_count().saturating_sub(1));
            eps.cursor_col = 0;
            return Ok(());
        }
    }

    Err(DomainError::NoActiveBuffer)
}
