//! Insertion into a [`PieceTable`]: splice new text in at a byte offset.

use crate::{
    error::{PositionError, TextError, TextResult},
    piece_table::{Piece, PieceKind, PieceTable},
};

impl PieceTable {
    /// Insert `text` at byte offset `index` in the rendered document.
    ///
    /// `index` must be on a UTF-8 char boundary and at most `self.len()`.
    /// The line-break cache is invalidated, not rebuilt; callers that need it
    /// pay for it on the next read rather than on every insert.
    pub fn insert(&mut self, text: &str, index: usize) -> TextResult<()> {
        if index > self.doc_len {
            return Err(TextError::IndexOutOfBounds {
                index,
                len: self.doc_len,
            });
        }
        if text.is_empty() {
            return Err(TextError::EmptyInsertText);
        }
        if !self.is_char_boundary(index) {
            return Err(TextError::InvalidInsertPosition {
                at: index,
                reason: PositionError::NotACharBoundary,
            });
        }

        // Capture EOF-ness BEFORE mutating doc_len.
        let is_eof = index == self.doc_len;
        let add_start = self.additive_text.len();

        // After this point, no fallible operations — state is mutated only on success.
        self.additive_text.push_str(text);
        self.doc_len += text.len();

        // Fast path: appending at EOF, merge with trailing additive piece if contiguous.
        if is_eof {
            self.push_additive_at_end(add_start, text.len());
            self.invalidate_line_breaks();
            return Ok(());
        }

        // Walk pieces in document coordinates.
        let mut offset = 0usize;
        for i in 0..self.pieces.len() {
            let piece = self.pieces[i];
            let piece_start_doc = offset;
            let piece_end_doc = offset + piece.len;
            offset = piece_end_doc;

            if index == piece_start_doc {
                self.pieces.insert(
                    i,
                    Piece {
                        start: add_start,
                        len: text.len(),
                        kind: PieceKind::Additive,
                    },
                );
                self.invalidate_line_breaks();
                return Ok(());
            }
            if index == piece_end_doc {
                // At the end of this piece == start of the next; insert after.
                self.pieces.insert(
                    i + 1,
                    Piece {
                        start: add_start,
                        len: text.len(),
                        kind: PieceKind::Additive,
                    },
                );
                self.invalidate_line_breaks();
                return Ok(());
            }
            if index < piece_end_doc {
                // Strictly inside this piece -> split into left | additive | right.
                let left_len = index - piece_start_doc;
                let right_len = piece_end_doc - index;
                debug_assert!(left_len > 0 && right_len > 0);

                let left = Piece {
                    start: piece.start,
                    len: left_len,
                    kind: piece.kind,
                };
                let right = Piece {
                    start: piece.start + left_len,
                    len: right_len,
                    kind: piece.kind,
                };
                let middle = Piece {
                    start: add_start,
                    len: text.len(),
                    kind: PieceKind::Additive,
                };

                self.pieces[i] = left;
                self.pieces.insert(i + 1, middle);
                self.pieces.insert(i + 2, right);
                self.invalidate_line_breaks();
                return Ok(());
            }
        }

        // Unreachable if invariants hold: index <= doc_len and we covered every
        // position in [0, doc_len]. A crash here means a bug, not a user error.
        unreachable!(
            "insert fell through: index={index}, doc_len={}, pieces={}",
            self.doc_len,
            self.pieces.len()
        );
    }

    /// Append a new additive piece, merging with the last piece if it is
    /// additive and ends exactly where the new bytes start in the add buffer.
    fn push_additive_at_end(&mut self, add_start: usize, len: usize) {
        if let Some(last) = self.pieces.last_mut() {
            if last.kind == PieceKind::Additive && last.start + last.len == add_start {
                last.len += len;
                return;
            }
        }
        self.pieces.push(Piece {
            start: add_start,
            len,
            kind: PieceKind::Additive,
        });
    }

    /// Drop the cached line-break list; it is rebuilt on next access.
    #[inline]
    pub(super) fn invalidate_line_breaks(&mut self) {
        self.line_breaks = std::cell::OnceCell::new();
    }

    /// Is `index` a valid byte boundary in the rendered document?
    ///
    /// Used by `delete` to reject ranges that would split a multi-byte char.
    #[inline]
    pub fn is_char_boundary(&self, index: usize) -> bool {
        if index > self.doc_len {
            return false;
        }
        if index == 0 || index == self.doc_len {
            return true;
        }
        // Walk pieces to find the piece containing `index` and check its source
        // is a char boundary there.
        let mut offset = 0usize;
        for p in &self.pieces {
            let end = offset + p.len;
            if index <= end {
                let src = match p.kind {
                    PieceKind::Original => &self.original_text,
                    PieceKind::Additive => &self.additive_text,
                };
                return src.is_char_boundary(p.start + (index - offset));
            }
            offset = end;
        }
        false
    }
}

#[cfg(test)]
mod insert_tests {
    use super::*;

    // ---- helpers ----

    fn pt(initial: &str) -> PieceTable {
        PieceTable::from(initial) // adjust to your constructor
    }

    fn assert_doc(t: &PieceTable, expected: &str) {
        assert_eq!(t.text(), expected, "rendered text mismatch");
        assert_eq!(t.doc_len, expected.len(), "doc_len mismatch");
    }

    fn assert_no_empty_pieces(t: &PieceTable) {
        for (i, p) in t.pieces.iter().enumerate() {
            assert!(p.len > 0, "piece {i} is zero-length: {:?}", p);
        }
    }

    // ---- empty / errors ----

    #[test]
    fn insert_into_empty_document() {
        let mut t = pt("");
        t.insert("hello", 0).unwrap();
        assert_doc(&t, "hello");
    }

    #[test]
    fn insert_empty_text_errors() {
        let mut t = pt("abc");
        assert!(t.insert("", 0).is_err());
    }

    #[test]
    fn insert_index_past_eof_errors() {
        let mut t = pt("hello");
        assert!(t.insert("x", 6).is_err());
        // and must not mutate on error
        assert_doc(&t, "hello");
    }

    #[test]
    fn insert_index_at_doc_len_is_ok() {
        let mut t = pt("hello");
        t.insert("!", 5).unwrap();
        assert_doc(&t, "hello!");
    }

    #[test]
    fn insert_index_zero_is_ok() {
        let mut t = pt("world");
        t.insert("hello ", 0).unwrap();
        assert_doc(&t, "hello world");
    }

    // ---- single-piece (original) splits ----

    #[test]
    fn insert_at_start() {
        let mut t = pt("world");
        t.insert("hello ", 0).unwrap();
        assert_doc(&t, "hello world");
    }

    #[test]
    fn insert_at_end() {
        let mut t = pt("hello");
        t.insert(" world", 5).unwrap();
        assert_doc(&t, "hello world");
    }

    #[test]
    fn insert_in_middle_splits_piece() {
        let mut t = pt("hello world");
        t.insert("X", 5).unwrap();
        assert_doc(&t, "helloX world");
        // must have split into 3 pieces: left, additive, right
        assert_eq!(t.pieces.len(), 3);
        assert_no_empty_pieces(&t);
    }

    #[test]
    fn insert_single_byte_into_single_byte_doc() {
        let mut t = pt("a");
        t.insert("b", 0).unwrap();
        assert_doc(&t, "ba");
        t.insert("c", 2).unwrap();
        assert_doc(&t, "bac");
    }

    // ---- ordering / multi-insert ----

    #[test]
    fn sequential_inserts_at_eof() {
        let mut t = pt("");
        t.insert("a", 0).unwrap();
        t.insert("b", 1).unwrap();
        t.insert("c", 2).unwrap();
        assert_doc(&t, "abc");
    }

    #[test]
    fn sequential_inserts_at_start() {
        let mut t = pt("");
        t.insert("c", 0).unwrap();
        t.insert("b", 0).unwrap();
        t.insert("a", 0).unwrap();
        assert_doc(&t, "abc");
    }

    #[test]
    fn insert_at_same_point_preserves_order() {
        let mut t = pt("AB");
        t.insert("x", 1).unwrap(); // AxB
        t.insert("y", 1).unwrap(); // AyxB  (y inserted before x because index=1 is before x)
        assert_doc(&t, "AyxB");
    }

    #[test]
    fn insert_into_additive_piece_splits_it() {
        let mut t = pt("");
        t.insert("hello", 0).unwrap();
        t.insert("X", 2).unwrap(); // heXllo
        assert_doc(&t, "heXllo");
        assert_no_empty_pieces(&t);
    }

    // ---- boundaries between pieces ----

    #[test]
    fn insert_at_boundary_between_original_and_additive() {
        // pieces: [Original "abc"][Additive "def"]
        let mut t = pt("abc");
        t.insert("def", 3).unwrap();
        assert_doc(&t, "abcdef");

        // insert exactly at the seam (doc index 3)
        t.insert("X", 3).unwrap();
        assert_doc(&t, "abcXdef");
        assert_no_empty_pieces(&t);
    }

    #[test]
    fn insert_at_boundary_between_two_additive_pieces() {
        let mut t = pt("");
        t.insert("abc", 0).unwrap();
        t.insert("def", 3).unwrap();
        // insert at seam
        t.insert("X", 3).unwrap();
        assert_doc(&t, "abcXdef");
        assert_no_empty_pieces(&t);
    }

    // ---- merge / piece count growth ----

    #[test]
    fn typing_at_eof_merges_pieces() {
        let mut t = pt("");
        for (i, ch) in "hello".chars().enumerate() {
            t.insert(&ch.to_string(), i).unwrap();
        }
        assert_doc(&t, "hello");
        // With merge-on-append, should be a single additive piece.
        assert_eq!(t.pieces.len(), 1, "expected EOF appends to merge");
    }

    #[test]
    fn typing_100_chars_at_eof_keeps_piece_count_low() {
        let mut t = pt("");
        let mut model = String::new();
        for i in 0..100 {
            t.insert("a", i).unwrap();
            model.push('a');
        }
        assert_eq!(t.text(), model);
        assert!(
            t.pieces.len() <= 2,
            "piece count exploded: {}",
            t.pieces.len()
        );
    }

    #[test]
    fn no_zero_length_pieces_after_boundary_inserts() {
        // A battery of boundary inserts; none should ever create a 0-len piece.
        let mut t = pt("abc");
        for (i, s) in ["X", "Y", "Z", "W"].iter().enumerate() {
            t.insert(s, i).unwrap();
            assert_no_empty_pieces(&t);
        }
        assert_doc(&t, "XYZWabc");
    }

    // ---- line breaks ----

    #[test]
    fn insert_newline_updates_line_breaks() {
        let mut t = pt("ab");
        assert_eq!(t.line_breaks().len(), 0);
        t.insert("\n", 1).unwrap(); // "a\nb"
        assert_eq!(t.text(), "a\nb");
        assert_eq!(t.line_breaks(), &[1]);
    }

    #[test]
    fn insert_text_with_newlines_at_start() {
        let mut t = pt("world");
        t.insert("a\nb\n", 0).unwrap();
        assert_doc(&t, "a\nb\nworld");
        assert_eq!(t.line_breaks(), &[1, 3]);
        assert_eq!(t.line_count(), 3);
    }

    // ---- unicode ----

    #[test]
    fn insert_multibyte_on_char_boundary() {
        let mut t = pt("ab");
        t.insert("é", 1).unwrap(); // aéb
        assert_doc(&t, "aéb");
    }

    // ---- model-based random ----

    #[test]
    fn insert_random_against_string_model() {
        let mut t = pt("");
        let mut model = String::new();
        let mut seed: u64 = 0xDEAD_BEEF_CAFE_1234;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        let fragments = ["a", "bc", "def", "\n", "hello", "é"];

        for iter in 0..1000 {
            let len = model.len();
            // insert-only random test; positions must be on char boundaries
            let mut pos = next() % (len + 1);
            // snap to a char boundary to satisfy &str insertion into String model
            while !model.is_char_boundary(pos) {
                pos -= 1;
            }
            let frag = fragments[next() % fragments.len()];
            t.insert(frag, pos).unwrap();
            model.insert_str(pos, frag);

            assert_eq!(t.text(), model, "mismatch at iteration {iter}");
            assert_eq!(t.doc_len, model.len());
            assert_no_empty_pieces(&t);
        }
    }
}
