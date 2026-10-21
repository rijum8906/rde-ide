//! Line queries on a [`Buffer`], in character units for the editor cursor.

use std::ops::Range;

use super::Buffer;

impl Buffer {
    /// Number of lines in the document. An empty document counts as one
    /// (empty) line.
    pub fn line_count(&self) -> usize {
        self.piece_table.line_count()
    }

    /// Character range of line `idx`, excluding the trailing newline.
    ///
    /// The editor's cursor model is character-based, so this is converted from
    /// the piece table's byte range. `None` if the line does not exist.
    pub fn line_range(&self, idx: usize) -> Option<Range<usize>> {
        let range = self.piece_table.line_range(idx)?;
        Some(
            self.piece_table.byte_to_char_offset(range.start)
                ..self.piece_table.byte_to_char_offset(range.end),
        )
    }

    /// Text of line `idx` without its trailing newline, or `None` if the line
    /// does not exist.
    pub fn line(&self, idx: usize) -> Option<String> {
        self.piece_table.line(idx)
    }

    /// Number of characters in line `idx` (0 if the line does not exist).
    pub fn line_char_len(&self, idx: usize) -> usize {
        self.line(idx).map(|line| line.chars().count()).unwrap_or(0)
    }

    /// Number of bytes in line `idx` (0 if the line does not exist).
    pub fn line_byte_len(&self, idx: usize) -> usize {
        self.line(idx).map(|line| line.len()).unwrap_or(0)
    }

    /// Map a document byte offset to a 0-based `(line, column)` pair, with the
    /// column counted in characters. `None` if the offset is past the end or
    /// splits a multi-byte character.
    pub fn line_col_at(&self, byte_offset: usize) -> Option<(usize, usize)> {
        self.piece_table.line_col_at(byte_offset)
    }
}

#[cfg(test)]
mod line_tests {
    use super::*;

    #[test]
    fn empty_document_has_one_empty_line() {
        let buffer = Buffer::scratch("scratch");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line(0).as_deref(), Some(""));
        assert_eq!(buffer.line_range(0), Some(0..0));
        assert_eq!(buffer.line(1), None);
    }

    #[test]
    fn line_queries_are_character_based_for_multibyte_text() {
        let buffer = Buffer::with_contents("test.txt", "héllo\nwörld");

        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line(0).as_deref(), Some("héllo"));
        assert_eq!(buffer.line(1).as_deref(), Some("wörld"));

        // "héllo" is 5 chars / 6 bytes, so line 1 starts at char 6 (byte 7).
        assert_eq!(buffer.line_range(0), Some(0..5));
        assert_eq!(buffer.line_range(1), Some(6..11));
        assert_eq!(buffer.line_char_len(0), 5);
        assert_eq!(buffer.line_byte_len(0), 6);
    }

    #[test]
    fn line_col_at_uses_character_columns() {
        let buffer = Buffer::with_contents("test.txt", "héllo\nwörld");

        assert_eq!(buffer.line_col_at(0), Some((0, 0)));
        assert_eq!(buffer.line_col_at(7), Some((1, 0)));
        assert_eq!(buffer.line_col_at(13), Some((1, 5)));
        assert_eq!(buffer.line_col_at(2), None);
    }

    #[test]
    fn inserting_newlines_updates_lines_and_ranges() {
        let mut buffer = Buffer::with_contents("test.txt", "one\ntwo");
        assert_eq!(buffer.line_count(), 2);

        buffer.insert(3, "\n\n").unwrap(); // "one\n\n\ntwo"
        assert_eq!(buffer.line_count(), 4);
        assert_eq!(buffer.line(1).as_deref(), Some(""));
        assert_eq!(buffer.line(2).as_deref(), Some(""));
        assert_eq!(buffer.line(3).as_deref(), Some("two"));
        assert_eq!(buffer.line_range(3), Some(6..9));
        assert!(buffer.is_dirty());

        buffer.delete(3..5).unwrap(); // back to "one\ntwo"
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line_range(1), Some(4..7));
    }
}
