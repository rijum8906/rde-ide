//! Word motions over a [`PieceTable`], in document byte offsets.
//!
//! "Small" words break on whitespace and `-`; "big" words break on whitespace
//! only. Both kinds scan the document linearly from the requested position.

pub const BIG_WORD_BOUNDARY: [char; 4] = [' ', '\t', '\n', '\r'];
pub const BIG_WORD_BOUNDARY_BYTES: [u8; 4] = [b' ', b'\t', b'\n', b'\r'];

pub const WORD_BOUNDARY: [char; 6] = [' ', '-', '.', '\t', '\n', '\r'];
pub const WORD_BOUNDARY_BYTES: [u8; 6] = [b' ', b'-', b'.', b'\t', b'\n', b'\r'];

pub fn is_big_word_boundary(c: char) -> bool {
    BIG_WORD_BOUNDARY.contains(&c)
}

pub fn is_word_boundary(c: char) -> bool {
    WORD_BOUNDARY.contains(&c)
}

use crate::piece_table::{PieceKind, PieceTable};

impl PieceTable {
    /// Byte range `[start, end)` of the "small word" you land on going forward
    /// from `pos` (Ctrl+Right semantics). `-` breaks words.
    ///
    /// - If `pos` is inside a word, returns the remainder of that word.
    /// - If only boundary chars (whitespace / `-` / newlines) remain, returns `None`.
    /// - If `pos >= doc_len`, returns `None`.
    ///
    /// O(bytes walked). Walk stops at the first word boundary after finding
    /// a word char, so worst case is the whole document if there are no
    /// boundaries — otherwise proportional to the distance to the next word end.
    pub fn next_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.next_word_range_with(pos, is_word_boundary)
    }

    /// Same as [`Self::next_word_range`] but using the "big word" boundary
    /// set (`-` does NOT break words; Ctrl+Shift+Right semantics).
    pub fn next_big_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.next_word_range_with(pos, is_big_word_boundary)
    }

    /// Byte range `[start, end)` of the "small word" you land on going backward
    /// from `pos` (Ctrl+Left semantics). `-` breaks words.
    ///
    /// - If `pos` is inside a word, returns the beginning of that word up to `pos`.
    /// - If only boundary chars precede `pos`, returns `None`.
    /// - If `pos == 0`, returns `None`.
    ///
    /// O(bytes walked) — symmetric to [`Self::next_word_range`].
    pub fn prev_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.prev_word_range_with(pos, is_word_boundary)
    }

    /// Same as [`Self::prev_word_range`] but using the "big word" boundary set.
    pub fn prev_big_word_range(&self, pos: usize) -> Option<(usize, usize)> {
        self.prev_word_range_with(pos, is_big_word_boundary)
    }

    // ---- shared cores ----------------------------------------------------

    fn next_word_range_with(
        &self,
        pos: usize,
        is_boundary: fn(char) -> bool,
    ) -> Option<(usize, usize)> {
        if pos >= self.doc_len {
            return None;
        }
        // Ensure we're on a char boundary (cursor indices should be, but be safe).
        debug_assert!(
            self.is_char_boundary(pos),
            "next_word_range: pos {pos} not on char boundary"
        );

        // 1. Skip boundary chars starting at `pos`.
        let start = self.skip_forward(pos, is_boundary);
        if start >= self.doc_len {
            return None;
        }

        // 2. Walk forward until the next boundary (or EOF); that is `end`.
        let end = self.skip_forward_non_boundary(start, is_boundary);
        Some((start, end))
    }

    fn prev_word_range_with(
        &self,
        pos: usize,
        is_boundary: fn(char) -> bool,
    ) -> Option<(usize, usize)> {
        if pos == 0 {
            return None;
        }
        debug_assert!(
            self.is_char_boundary(pos),
            "prev_word_range: pos {pos} not on char boundary"
        );

        // 1. Skip boundary chars going backward from `pos`.
        let end = self.skip_backward(pos, is_boundary);
        if end == 0 {
            return None;
        }

        // 2. Walk backward until the next boundary; that is `start`.
        let start = self.skip_backward_non_boundary(end, is_boundary);
        Some((start, end))
    }

    // ---- piece-aware character walks -------------------------------------
    //
    // Each helper locates `pos` in the piece list once, then walks the pieces
    // in order. Scanning piece slices directly keeps the cost proportional to
    // the bytes walked instead of resolving a piece from the start for every
    // character (which would be O(pieces) per step). Each helper assumes its
    // start position is on a char boundary (guaranteed by callers / cursor
    // invariants).

    /// Index of the piece containing `pos`, plus the byte offset within it.
    /// `pos == doc_len` maps to `(pieces.len(), 0)`.
    fn locate_piece(&self, pos: usize) -> (usize, usize) {
        let mut doc = 0usize;
        for (index, piece) in self.pieces.iter().enumerate() {
            let end = doc + piece.len;
            if pos < end {
                return (index, pos - doc);
            }
            doc = end;
        }
        (self.pieces.len(), 0)
    }

    /// Advance from `pos` past every char satisfying `is_boundary`.
    /// Returns the first position whose char does NOT satisfy it (or `doc_len`).
    fn skip_forward(&self, pos: usize, is_boundary: fn(char) -> bool) -> usize {
        let (mut index, mut local) = self.locate_piece(pos);
        let mut piece_start_doc = pos - local;

        while index < self.pieces.len() {
            let piece = self.pieces[index];
            let slice = &self.source(piece.kind)[piece.start..piece.end()];

            for ch in slice[local..].chars() {
                if !is_boundary(ch) {
                    return piece_start_doc + local;
                }
                local += ch.len_utf8();
            }

            piece_start_doc += piece.len;
            index += 1;
            local = 0;
        }
        self.doc_len
    }

    /// Advance from `pos` until a char satisfying `is_boundary` (or EOF).
    /// Returns the position of that char (or `doc_len`).
    fn skip_forward_non_boundary(&self, pos: usize, is_boundary: fn(char) -> bool) -> usize {
        let (mut index, mut local) = self.locate_piece(pos);
        let mut piece_start_doc = pos - local;

        while index < self.pieces.len() {
            let piece = self.pieces[index];
            let slice = &self.source(piece.kind)[piece.start..piece.end()];

            for ch in slice[local..].chars() {
                if is_boundary(ch) {
                    return piece_start_doc + local;
                }
                local += ch.len_utf8();
            }

            piece_start_doc += piece.len;
            index += 1;
            local = 0;
        }
        self.doc_len
    }

    /// Retreat from `pos` past every char satisfying `is_boundary`.
    /// Returns the position *after* the last boundary char skipped (or 0).
    fn skip_backward(&self, pos: usize, is_boundary: fn(char) -> bool) -> usize {
        if pos == 0 {
            return 0;
        }
        let (mut index, mut local) = self.locate_piece(pos);
        let mut piece_start_doc = pos - local;

        loop {
            if local == 0 {
                if index == 0 {
                    return 0;
                }
                index -= 1;
                let piece = self.pieces[index];
                piece_start_doc -= piece.len;
                local = piece.len;
            }

            let piece = self.pieces[index];
            let slice = &self.source(piece.kind)[piece.start..piece.end()][..local];
            let Some(ch) = slice.chars().next_back() else {
                return piece_start_doc;
            };
            if !is_boundary(ch) {
                return piece_start_doc + local;
            }
            local -= ch.len_utf8();
        }
    }

    /// Retreat from `pos` until a char satisfying `is_boundary` (or 0).
    /// Returns the position of the first non-boundary char's start (or 0).
    fn skip_backward_non_boundary(&self, pos: usize, is_boundary: fn(char) -> bool) -> usize {
        if pos == 0 {
            return 0;
        }
        let (mut index, mut local) = self.locate_piece(pos);
        let mut piece_start_doc = pos - local;

        loop {
            if local == 0 {
                if index == 0 {
                    return 0;
                }
                index -= 1;
                let piece = self.pieces[index];
                piece_start_doc -= piece.len;
                local = piece.len;
            }

            let piece = self.pieces[index];
            let slice = &self.source(piece.kind)[piece.start..piece.end()][..local];
            let Some(ch) = slice.chars().next_back() else {
                return piece_start_doc;
            };
            if is_boundary(ch) {
                return piece_start_doc + local;
            }
            local -= ch.len_utf8();
        }
    }

    // ---- piece-aware single-char accessors -------------------------------

    /// Byte offset of the char immediately before `pos`, or `None` at 0.
    ///
    /// This is the inclusive position of the last character of the range that
    /// ends at `pos`, which is where a cursor lands after a backward motion.
    pub fn prev_char_boundary(&self, pos: usize) -> Option<usize> {
        self.char_before(pos).map(|(_, start)| start)
    }

    /// Char ending immediately before document byte offset `pos`, plus the
    /// offset of that char's start. Returns `None` if `pos == 0`.
    ///
    /// Panics in debug if `pos` is not on a char boundary.
    fn char_before(&self, pos: usize) -> Option<(char, usize)> {
        if pos == 0 {
            return None;
        }
        let mut doc = 0usize;
        for piece in &self.pieces {
            let piece_end = doc + piece.len;
            if pos <= piece_end {
                let src = match piece.kind {
                    PieceKind::Original => &self.original_text,
                    PieceKind::Additive => &self.additive_text,
                };
                let src_end = piece.start + (pos - doc);
                debug_assert!(src.is_char_boundary(src_end));
                let c = src[..src_end].chars().next_back()?;
                return Some((c, pos - c.len_utf8()));
            }
            doc = piece_end;
        }
        None
    }
}

#[cfg(test)]
mod word_nav_tests {
    use super::*;

    fn pt(s: &str) -> PieceTable {
        PieceTable::from(s)
    }

    #[test]
    fn next_word_basic() {
        let t = pt("hello world");
        assert_eq!(t.next_word_range(0), Some((0, 5)));
        assert_eq!(t.next_word_range(5), Some((6, 11))); // from the space
        assert_eq!(t.next_word_range(6), Some((6, 11))); // from start of "world"
        assert_eq!(t.next_word_range(11), None);
    }

    #[test]
    fn next_word_from_mid_word_returns_remainder() {
        let t = pt("hello world");
        assert_eq!(t.next_word_range(2), Some((2, 5))); // "llo"
    }

    #[test]
    fn next_word_skips_leading_boundaries() {
        let t = pt("   hello");
        assert_eq!(t.next_word_range(0), Some((3, 8)));
    }

    #[test]
    fn next_word_only_boundaries_returns_none() {
        let t = pt("   \n\t ");
        assert_eq!(t.next_word_range(0), None);
    }

    #[test]
    fn next_word_hyphen_is_boundary() {
        let t = pt("hello-world");
        assert_eq!(t.next_word_range(0), Some((0, 5)));
        assert_eq!(t.next_word_range(5), Some((6, 11))); // skips '-' then "world"
    }

    #[test]
    fn next_big_word_ignores_hyphen() {
        let t = pt("hello-world");
        assert_eq!(t.next_big_word_range(0), Some((0, 11)));
    }

    #[test]
    fn next_word_crlf_skipped_as_one_run() {
        let t = pt("foo\r\nbar");
        assert_eq!(t.next_word_range(0), Some((0, 3))); // "foo"
        assert_eq!(t.next_word_range(3), Some((5, 8))); // skip "\r\n", get "bar"
    }

    #[test]
    fn prev_word_basic() {
        let t = pt("hello world");
        assert_eq!(t.prev_word_range(11), Some((6, 11)));
        assert_eq!(t.prev_word_range(6), Some((0, 5))); // from space
        assert_eq!(t.prev_word_range(0), None);
    }

    #[test]
    fn prev_word_from_mid_word_returns_prefix() {
        let t = pt("hello world");
        assert_eq!(t.prev_word_range(8), Some((6, 8))); // "wo"
    }

    #[test]
    fn prev_word_only_boundaries_returns_none() {
        let t = pt("   ");
        assert_eq!(t.prev_word_range(3), None);
    }

    #[test]
    fn prev_word_crlf() {
        let t = pt("foo\r\nbar");
        assert_eq!(t.prev_word_range(8), Some((5, 8))); // "bar"
        assert_eq!(t.prev_word_range(5), Some((0, 3))); // skip "\r\n", get "foo"
    }

    #[test]
    fn next_prev_roundtrip() {
        let t = pt("alpha beta gamma");
        let (s, e) = t.next_word_range(0).unwrap(); // "alpha"
        assert_eq!((s, e), (0, 5));
        let (s2, e2) = t.next_word_range(e).unwrap(); // "beta"
        assert_eq!((s2, e2), (6, 10));
        assert_eq!(t.prev_word_range(e2), Some((6, 10))); // back to "beta"
        assert_eq!(t.prev_word_range(s2), Some((0, 5))); // back to "alpha"
    }

    #[test]
    fn multibyte_words() {
        let t = pt("héllo wörld");
        // 'é' is 2 bytes, 'ö' is 2 bytes.
        assert_eq!(t.next_word_range(0), Some((0, 6))); // "héllo"
        // doc bytes: h(1) é(2) l l o = 6 bytes; space = 1; w(1) ö(2) r l d = 6
        assert_eq!(t.next_word_range(6), Some((7, 13))); // "wörld"
        assert_eq!(t.prev_word_range(13), Some((7, 13)));
    }

    #[test]
    fn works_across_pieces() {
        let mut t = pt("hello");
        t.insert(" world", 5).unwrap(); // "hello world"
        // "hello" is Original, " world" is Additive.
        assert_eq!(t.next_word_range(0), Some((0, 5)));
        assert_eq!(t.next_word_range(5), Some((6, 11)));
        assert_eq!(t.prev_word_range(11), Some((6, 11)));
    }

    #[test]
    fn empty_document() {
        let t = pt("");
        assert_eq!(t.next_word_range(0), None);
        assert_eq!(t.prev_word_range(0), None);
    }
}
