//! Insertion into a [`Buffer`]: splice text in at a byte offset.

use crate::error::DomainResult;

use super::Buffer;

impl Buffer {
    /// Insert `text` at byte `offset`. Empty text is a successful no-op.
    ///
    /// `offset` must be on a UTF-8 char boundary and at most the document
    /// length; the piece table rejects anything else. A successful insert
    /// marks the buffer dirty.
    pub fn insert(&mut self, offset: usize, text: &str) -> DomainResult<()> {
        if text.is_empty() {
            return Ok(());
        }
        self.piece_table.insert(text, offset)?;
        self.mark_dirty();
        Ok(())
    }
}

#[cfg(test)]
mod insert_tests {
    use super::*;
    use crate::buffer::SaveState;

    #[test]
    fn insert_past_end_is_reported() {
        let mut buffer = Buffer::with_contents("test.txt", "abc");
        assert!(buffer.insert(4, "x").is_err());
        assert_eq!(buffer.to_string(), "abc");
        assert_eq!(buffer.save_state(), SaveState::Clean);
    }

    #[test]
    fn insert_inside_a_multibyte_char_is_reported() {
        let mut buffer = Buffer::with_contents("test.txt", "é");
        assert!(buffer.insert(1, "x").is_err());
        assert_eq!(buffer.to_string(), "é");
    }

    #[test]
    fn insert_splits_the_piece_and_marks_dirty() {
        let mut buffer = Buffer::with_contents("test.txt", "ac");
        buffer.insert(1, "b").unwrap();

        assert_eq!(buffer.to_string(), "abc");
        assert_eq!(buffer.line(0).as_deref(), Some("abc"));
        assert_eq!(buffer.save_state(), SaveState::Dirty);
    }
}
