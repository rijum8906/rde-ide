//! Line lookup: byte ranges and rendered text for a single line.

use std::ops::Range;

use crate::piece_table::PieceTable;

impl PieceTable {
    /// Byte range `[start, end)` of line `line` (0-based), excluding the
    /// trailing `\n`. `end == start` for an empty line.
    ///
    /// Returns `None` if `line >= self.line_count()`.
    ///
    /// O(pieces + log(line_breaks)) — walks pieces only up to the target line.
    pub fn line_range(&self, line: usize) -> Option<Range<usize>> {
        let breaks = self.line_breaks();
        if line > breaks.len() {
            return None;
        }

        let start = if line == 0 {
            0
        } else {
            // Line N starts right after the (N-1)th newline.
            breaks[line - 1] + 1
        };

        let end = if line < breaks.len() {
            // There is a `\n` at breaks[line]; the line ends there.
            breaks[line]
        } else {
            // Last line: extends to doc_len.
            self.len()
        };

        Some(start..end)
    }

    /// Text of line `line` (0-based) without its trailing `\n`, or `None` if
    /// the line does not exist. Empty lines yield an empty `String`.
    pub fn line(&self, line: usize) -> Option<String> {
        self.line_range(line).map(|range| self.slice(range))
    }

    /// Map a document byte offset to a 0-based `(line, column)` pair, where
    /// the column counts characters, not bytes.
    ///
    /// A position on a `\n` reports the end of that line. An offset past the
    /// document, or inside a multi-byte character, returns `None`.
    ///
    /// O(log lines + pieces + bytes on the line).
    pub fn line_col_at(&self, byte_offset: usize) -> Option<(usize, usize)> {
        if byte_offset > self.len() || !self.is_char_boundary(byte_offset) {
            return None;
        }

        let breaks = self.line_breaks();
        let line = breaks.partition_point(|&newline| newline < byte_offset);
        let line_start = if line == 0 { 0 } else { breaks[line - 1] + 1 };

        Some((line, self.char_count_in(line_start, byte_offset)))
    }

    /// Number of chars in the byte range `start..end`.
    fn char_count_in(&self, start: usize, end: usize) -> usize {
        let mut count = 0usize;
        let mut doc = 0usize;

        for piece in &self.pieces {
            let piece_start = doc;
            let piece_end = doc + piece.len;
            doc = piece_end;

            if piece_end <= start {
                continue;
            }
            if piece_start >= end {
                break;
            }

            let from = start.saturating_sub(piece_start);
            let to = (end - piece_start).min(piece.len);
            let src = self.source(piece.kind);
            count += src[piece.start + from..piece.start + to].chars().count();
        }

        count
    }
}

#[cfg(test)]
mod line_range_tests {
    use super::*;

    fn pt(s: &str) -> PieceTable {
        PieceTable::from(s)
    }

    #[test]
    fn single_line_no_newline() {
        let t = pt("hello");
        assert_eq!(t.line_range(0), Some(0..5));
        assert_eq!(t.line_range(1), None);
    }

    #[test]
    fn empty_document() {
        let t = pt("");
        // one empty line, range (0, 0)
        assert_eq!(t.line_range(0), Some(0..0));
        assert_eq!(t.line_range(1), None);
    }

    #[test]
    fn two_lines() {
        let t = pt("hello\nworld");
        assert_eq!(t.line_range(0), Some(0..5));
        assert_eq!(t.line_range(1), Some(6..11));
        assert_eq!(t.line_range(2), None);
    }

    #[test]
    fn empty_middle_line() {
        let t = pt("a\n\nb");
        // lines: "a", "", "b"
        assert_eq!(t.line_range(0), Some(0..1));
        assert_eq!(t.line_range(1), Some(2..2)); // empty: start == end
        assert_eq!(t.line_range(2), Some(3..4));
    }

    #[test]
    fn line_range_excludes_newline() {
        let t = pt("ab\ncd");
        assert_eq!(t.line_range(0), Some(0..2)); // "ab", not "ab\n"
        assert_eq!(t.line_range(1), Some(3..5)); // "cd"
    }

    #[test]
    fn trailing_newline_creates_empty_last_line() {
        // Matches line_count() = line_breaks.len() + 1
        let t = pt("ab\n");
        assert_eq!(t.line_count(), 2);
        assert_eq!(t.line_range(0), Some(0..2));
        assert_eq!(t.line_range(1), Some(3..3)); // empty last line
        assert_eq!(t.line_range(2), None);
    }

    #[test]
    fn reflects_inserts() {
        let mut t = pt("hello world");
        assert_eq!(t.line_range(0), Some(0..11));

        let res = t.insert("\n", 5); // "hello\n world"
        assert!(res.is_ok());
        assert_eq!(t.line_range(0), Some(0..5));
        assert_eq!(t.line_range(1), Some(6..12));
    }

    #[test]
    fn reflects_deletes() {
        let mut t = pt("a\nb\nc");
        let res = t.delete(1..2); // "ab\nc"
        assert!(res.is_ok());
        assert_eq!(t.line_range(0), Some(0..2));
        assert_eq!(t.line_range(1), Some(3..4));
    }

    #[test]
    fn multi_piece_no_render() {
        // Ensure it works when the line spans multiple pieces.
        let mut t = pt("abc");
        let res = t.insert("X", 1); // "aXbc"
        assert!(res.is_ok());
        let res = t.insert("Y", 3); // "aXbYc"
        assert!(res.is_ok());
        assert_eq!(t.line_range(0), Some(0..5));
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let t = pt("a\nb");
        assert_eq!(t.line_range(2), None);
        assert_eq!(t.line_range(usize::MAX), None);
    }

    #[test]
    fn line_col_at_maps_bytes_to_char_columns() {
        let t = pt("héllo\nwörld");

        assert_eq!(t.line_col_at(0), Some((0, 0)));
        assert_eq!(t.line_col_at(3), Some((0, 2)));
        assert_eq!(t.line_col_at(6), Some((0, 5))); // the newline
        assert_eq!(t.line_col_at(7), Some((1, 0)));
        assert_eq!(t.line_col_at(13), Some((1, 5))); // end of document

        assert_eq!(t.line_col_at(2), None); // inside 'é'
        assert_eq!(t.line_col_at(14), None);
    }
}
