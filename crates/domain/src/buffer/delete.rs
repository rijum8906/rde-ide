//! Deletion from a [`Buffer`], plus whole-document replacement.

use std::ops::Range;

use crate::error::DomainResult;

use super::Buffer;

impl Buffer {
    /// Delete the byte range `range`. The range must lie on char boundaries.
    ///
    /// An empty range is a successful no-op. A successful delete marks the
    /// buffer dirty; a rejected range leaves the document untouched.
    pub fn delete(&mut self, range: Range<usize>) -> DomainResult<()> {
        if range.start == range.end {
            return Ok(());
        }
        self.piece_table.delete(range)?;
        self.mark_dirty();
        Ok(())
    }

    /// Replace the whole document with `text`.
    pub fn replace_all(&mut self, text: &str) {
        self.piece_table.replace_all(text);
        self.mark_dirty();
    }
}

#[cfg(test)]
mod delete_tests {
    use super::*;
    use crate::buffer::SaveState;

    #[test]
    fn delete_rejects_splitting_a_multibyte_char() {
        let mut buffer = Buffer::with_contents("test.txt", "é");
        assert!(buffer.delete(1..2).is_err());
        assert_eq!(buffer.to_string(), "é");
        assert_eq!(buffer.save_state(), SaveState::Clean);
    }

    #[test]
    fn delete_out_of_bounds_leaves_document_unchanged() {
        let mut buffer = Buffer::with_contents("test.txt", "abc");
        assert!(buffer.delete(1..4).is_err());
        assert_eq!(buffer.to_string(), "abc");
        assert_eq!(buffer.save_state(), SaveState::Clean);
    }

    #[test]
    fn replace_all_swaps_the_document() {
        let mut buffer = Buffer::with_contents("test.txt", "old\ntext");

        buffer.replace_all("new");
        assert_eq!(buffer.to_string(), "new");
        assert_eq!(buffer.char_count(), 3);
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line_range(0), Some(0..3));
        assert!(buffer.is_dirty());
    }
}
