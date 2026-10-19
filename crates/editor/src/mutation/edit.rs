use domain::{
    error::{DomainError, DomainResult},
    workspace::WorkspaceState,
};
use std::ops::Range;
use tui::state::editor_panel::EditorPanelState;
// Assuming EditorState is imported or defined in your crate

pub fn delete_line(wss: &mut WorkspaceState, eps: &mut EditorPanelState) -> DomainResult<()> {
    if let Some(buffer) = wss.active_buffer_mut() {
        let line_count = buffer.line_count();
        if line_count == 0 {
            return Ok(());
        }

        let Some(range) = buffer.line_range(eps.cursor_line) else {
            return Ok(());
        };

        let start_char = range.start;
        let line_len = range.end - range.start;

        // 2. Determine the character range to delete, including its newline boundary
        let (delete_start, delete_end) = if eps.cursor_line + 1 < line_count {
            // Case A: Not the last line. Delete the line contents plus its trailing newline.
            (start_char, start_char + line_len + 1)
        } else if eps.cursor_line > 0 {
            // Case B: The last line (and not the only line). Delete the preceding newline
            // and this line so we don't leave a trailing blank line at EOF.
            (start_char.saturating_sub(1), start_char + line_len)
        } else {
            // Case C: The only line in the document. Just clear its contents.
            (start_char, start_char + line_len)
        };

        // 3. Convert logical character offsets into safe UTF-8 byte offsets
        let start_byte = buffer.char_to_byte_offset(delete_start);
        let end_byte = buffer.char_to_byte_offset(delete_end);

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
