//! Word motions on a [`Buffer`], in document byte offsets.
//!
//! The piece table reports each word as an exclusive byte range
//! `[start, end)`. Positions here are cursor landing spots, not boundaries:
//!
//! - `next_*_start` skips the word under `pos` and returns the first
//!   character of the following word, so pressing `w`/`W` at a word start
//!   always makes progress.
//! - `prev_*_start` returns the first character of the word the backward scan
//!   lands on (`b`/`B`).
//! - `*_end` returns the offset of the word's **last character** (`e`/`E`),
//!   not the exclusive end. If `pos` is already on that character, the motion
//!   advances to the next word's end instead of standing still.
//!
//! "Small" words break on whitespace and `-`; "big" words break on whitespace
//! only. The underlying `*_range` methods keep the piece table's contract: a
//! position inside a word yields the remainder of that word.
//!
//! [`PieceTable::next_word_range`]: crate::piece_table::PieceTable::next_word_range

use super::Buffer;

impl Buffer {
    /// Byte range of the small word you land on going forward from `pos`.
    pub fn next_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.piece_table.next_word_range(pos)
    }

    /// Byte range of the big word you land on going forward from `pos`.
    pub fn next_big_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.piece_table.next_big_word_range(pos)
    }

    /// Byte range of the small word you land on going backward from `pos`.
    pub fn prev_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.piece_table.prev_word_range(pos)
    }

    /// Byte range of the big word you land on going backward from `pos`.
    pub fn prev_big_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.piece_table.prev_big_word_range(pos)
    }

    /// Byte offset of the first character of the next small word (`w`).
    ///
    /// Skips the word under `pos`; `None` once no word follows.
    pub fn next_word_start(&self, pos: usize) -> Option<usize> {
        let (start, end) = self.next_word_range(pos)?;
        if start > pos {
            return Some(start);
        }
        self.next_word_range(end).map(|(next_start, _)| next_start)
    }

    /// Byte offset of the last character of the next small word (`e`).
    pub fn next_word_end(&self, pos: usize) -> Option<usize> {
        let (start, end) = self.next_word_range(pos)?;
        let last = self.piece_table.prev_char_boundary(end)?;
        if start == pos && last == pos {
            let (_, next_end) = self.next_word_range(end)?;
            return self.piece_table.prev_char_boundary(next_end);
        }
        Some(last)
    }

    /// Byte offset of the first character of the previous small word (`b`).
    pub fn prev_word_start(&self, pos: usize) -> Option<usize> {
        self.prev_word_range(pos).map(|(start, _)| start)
    }

    /// Byte offset of the last character of the previous small word (`ge`).
    pub fn prev_word_end(&self, pos: usize) -> Option<usize> {
        self.prev_word_range(pos)
            .and_then(|(_, end)| self.piece_table.prev_char_boundary(end))
    }

    /// Byte offset of the first character of the next big word (`W`).
    ///
    /// Skips the big word under `pos`; `None` once no word follows.
    pub fn next_big_word_start(&self, pos: usize) -> Option<usize> {
        let (start, end) = self.next_big_word_range(pos)?;
        if start > pos {
            return Some(start);
        }
        self.next_big_word_range(end)
            .map(|(next_start, _)| next_start)
    }

    /// Byte offset of the last character of the next big word (`E`).
    pub fn next_big_word_end(&self, pos: usize) -> Option<usize> {
        let (start, end) = self.next_big_word_range(pos)?;
        let last = self.piece_table.prev_char_boundary(end)?;
        if start == pos && last == pos {
            let (_, next_end) = self.next_big_word_range(end)?;
            return self.piece_table.prev_char_boundary(next_end);
        }
        Some(last)
    }

    /// Byte offset of the first character of the previous big word (`B`).
    pub fn prev_big_word_start(&self, pos: usize) -> Option<usize> {
        self.prev_big_word_range(pos).map(|(start, _)| start)
    }

    /// Byte offset of the last character of the previous big word (`gE`).
    pub fn prev_big_word_end(&self, pos: usize) -> Option<usize> {
        self.prev_big_word_range(pos)
            .and_then(|(_, end)| self.piece_table.prev_char_boundary(end))
    }
}

#[cfg(test)]
mod word_tests {
    use super::*;

    fn buffer(text: &str) -> Buffer {
        Buffer::with_contents("test.txt", text)
    }

    #[test]
    fn next_word_start_skips_the_current_word() {
        let buffer = buffer("hello world");

        assert_eq!(buffer.next_word_start(0), Some(6));
        assert_eq!(buffer.next_word_start(2), Some(6)); // mid-word
        assert_eq!(buffer.next_word_start(5), Some(6)); // from the space
        assert_eq!(buffer.next_word_start(6), None); // last word
        assert_eq!(buffer.next_word_start(11), None);
    }

    #[test]
    fn next_word_end_advances_from_the_last_character() {
        let buffer = buffer("hello world");

        assert_eq!(buffer.next_word_end(0), Some(4)); // 'o'
        assert_eq!(buffer.next_word_end(2), Some(4)); // mid-word
        assert_eq!(buffer.next_word_end(5), Some(10)); // 'd'
        assert_eq!(buffer.next_word_end(4), Some(10)); // already on 'o'
        assert_eq!(buffer.next_word_end(10), None);
    }

    #[test]
    fn prev_word_start_and_end() {
        let buffer = buffer("hello world");

        assert_eq!(buffer.prev_word_start(11), Some(6));
        assert_eq!(buffer.prev_word_start(6), Some(0)); // from the space
        assert_eq!(buffer.prev_word_start(0), None);

        assert_eq!(buffer.prev_word_end(11), Some(10)); // 'd'
        assert_eq!(buffer.prev_word_end(6), Some(4)); // 'o'
    }

    #[test]
    fn ranges_match_the_piece_table() {
        let buffer = buffer("hello-world foo");

        assert_eq!(buffer.next_word_range(0), Some((0, 5)));
        assert_eq!(buffer.next_word_range(5), Some((6, 11)));
        assert_eq!(buffer.next_big_word_range(0), Some((0, 11)));
        assert_eq!(buffer.prev_word_range(12), Some((6, 11)));
        assert_eq!(buffer.prev_big_word_range(12), Some((0, 11)));
    }

    #[test]
    fn big_word_motions_ignore_hyphens() {
        let buffer = buffer("hello-world foo");

        assert_eq!(buffer.next_big_word_start(0), Some(12));
        assert_eq!(buffer.next_big_word_start(5), Some(12)); // '-' is part of the word
        assert_eq!(buffer.next_big_word_start(11), Some(12)); // from the space
        assert_eq!(buffer.next_big_word_end(0), Some(10)); // 'd'
        assert_eq!(buffer.next_big_word_end(10), Some(14)); // already on 'd'
        assert_eq!(buffer.prev_big_word_start(12), Some(0));
        assert_eq!(buffer.prev_big_word_end(12), Some(10));
    }

    #[test]
    fn word_motions_use_byte_offsets_for_multibyte_text() {
        let buffer = buffer("héllo wörld");

        assert_eq!(buffer.next_word_range(0), Some((0, 6)));
        assert_eq!(buffer.next_word_start(0), Some(7));
        assert_eq!(buffer.next_word_end(0), Some(5)); // 'o'
        assert_eq!(buffer.prev_word_start(13), Some(7));
        assert_eq!(buffer.prev_word_end(13), Some(12)); // 'd'
    }

    #[test]
    fn empty_document_has_no_word_motions() {
        let buffer = Buffer::scratch("scratch");

        assert_eq!(buffer.next_word_start(0), None);
        assert_eq!(buffer.next_word_end(0), None);
        assert_eq!(buffer.prev_word_start(0), None);
        assert_eq!(buffer.prev_word_end(0), None);
    }

    #[test]
    fn motions_read_across_piece_boundaries() {
        let mut buffer = Buffer::with_contents("test.txt", "hello");
        buffer.insert(5, " world").unwrap();

        assert_eq!(buffer.next_word_start(0), Some(6));
        assert_eq!(buffer.next_word_start(5), Some(6)); // from the space
        assert_eq!(buffer.next_word_end(0), Some(4));
        assert_eq!(buffer.prev_word_start(11), Some(6));
    }
}
