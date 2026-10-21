//! Deletion from a [`PieceTable`]: drop or trim the pieces a byte range covers.

use std::ops::Range;

use crate::{
    error::{PositionError, TextError, TextResult},
    piece_table::{Piece, PieceTable, push_or_merge},
};

impl PieceTable {
    /// Delete the byte range `range` from the rendered document.
    ///
    /// Both ends must sit on UTF-8 char boundaries; splitting a multi-byte
    /// char would leave pieces that cannot be rendered, so it is rejected
    /// with [`TextError::InvalidDeleteRange`] instead of corrupting state.
    /// An empty range is a successful no-op.
    pub fn delete(&mut self, range: Range<usize>) -> TextResult<()> {
        // ---- 0. Validate ----
        if range.start > range.end {
            return Err(TextError::IndexOutOfBounds {
                index: range.start,
                len: self.doc_len,
            });
        }
        if range.end > self.doc_len {
            return Err(TextError::IndexOutOfBounds {
                index: range.end,
                len: self.doc_len,
            });
        }
        if range.start == range.end {
            return Ok(());
        }
        for boundary in [range.start, range.end] {
            if !self.is_char_boundary(boundary) {
                return Err(TextError::InvalidDeleteRange {
                    start: range.start,
                    end: range.end,
                    reason: PositionError::NotACharBoundary,
                });
            }
        }

        let del_start = range.start;
        let del_end = range.end;

        // ---- 1. Locate overlapping pieces in a single pass ----
        //
        // `first_overlap` = index of the first piece whose doc-span ends
        //                   strictly after `del_start`.
        // `last_overlap_excl` = one past the last piece whose doc-span starts
        //                       strictly before `del_end`.
        //
        // Both are exclusive/inclusive in the standard slice sense:
        // pieces[first_overlap..last_overlap_excl] is the affected region.
        //
        // `first_overlap_doc_start` = document byte offset of the first
        // affected piece — the starting point for the walk in step 2.
        let mut offset = 0usize;
        let mut first_overlap = None;
        let mut last_overlap_excl = 0usize;
        let mut first_overlap_doc_start = 0usize;

        for (i, piece) in self.pieces.iter().enumerate() {
            let piece_start_doc = offset;
            let piece_end_doc = offset + piece.len;
            offset = piece_end_doc;

            // Does this piece overlap [del_start, del_end)?
            let overlaps = piece_end_doc > del_start && piece_start_doc < del_end;
            if overlaps {
                if first_overlap.is_none() {
                    first_overlap = Some(i);
                    first_overlap_doc_start = piece_start_doc;
                }
                last_overlap_excl = i + 1;
            }
        }

        // At least one piece must overlap: del range is non-empty and within
        // [0, doc_len], and pieces cover the whole document contiguously.
        let first_overlap = match first_overlap {
            Some(i) => i,
            None => unreachable!(
                "delete: no overlapping piece for range {del_start}..{del_end}, doc_len={}, pieces={}",
                self.doc_len,
                self.pieces.len()
            ),
        };
        debug_assert!(last_overlap_excl > first_overlap);

        // ---- 2. Build the new list into scratch ----
        self.scratch.clear();
        self.scratch.reserve(self.pieces.len() + 1);

        // Prefix: untouched pieces before the affected region -> memcpy.
        self.scratch
            .extend_from_slice(&self.pieces[..first_overlap]);

        // Middle: for each affected piece, keep whatever survives on the left
        // and/or right of the deletion. Boundary pieces may contribute both;
        // fully-covered pieces contribute nothing.
        let mut doc = first_overlap_doc_start;
        for k in first_overlap..last_overlap_excl {
            let piece = self.pieces[k];
            let piece_start_doc = doc;
            let piece_end_doc = doc + piece.len;
            doc = piece_end_doc;

            // Left keep: bytes of this piece before `del_start`.
            let keep_left_len = del_start.saturating_sub(piece_start_doc);
            if keep_left_len > 0 {
                push_or_merge(
                    &mut self.scratch,
                    Piece {
                        start: piece.start,
                        len: keep_left_len,
                        kind: piece.kind,
                    },
                );
            }

            // Right keep: bytes of this piece after `del_end`.
            let keep_right_len = piece_end_doc.saturating_sub(del_end);
            if keep_right_len > 0 {
                // Bytes of this piece consumed before the right-keep begins.
                let consumed_before_right = del_end - piece_start_doc;
                push_or_merge(
                    &mut self.scratch,
                    Piece {
                        start: piece.start + consumed_before_right,
                        len: keep_right_len,
                        kind: piece.kind,
                    },
                );
            }
        }

        // Suffix: untouched pieces after the affected region -> memcpy.
        self.scratch
            .extend_from_slice(&self.pieces[last_overlap_excl..]);

        // ---- 3. Commit ----
        std::mem::swap(&mut self.pieces, &mut self.scratch);
        self.doc_len -= del_end - del_start;
        // Lazily recompute line breaks on next read — O(1) here, O(doc) only
        // when someone actually asks. Matches `insert`'s behavior.
        self.invalidate_line_breaks();
        Ok(())
    }
}

#[cfg(test)]
mod delete_tests {
    use crate::error::{PositionError, TextError};

    use super::*;

    // ---- helpers ----

    fn pt(initial: &str) -> PieceTable {
        PieceTable::from(initial)
    }

    /// Assert the rendered document, the cached length, the line cache, and
    /// the no-empty-piece invariant all agree with `expected`.
    fn assert_doc(t: &mut PieceTable, expected: &str) {
        assert_eq!(t.text(), expected, "rendered text mismatch");
        assert_eq!(t.len(), expected.len(), "doc_len mismatch");
        assert_eq!(t.is_empty(), expected.is_empty());
        assert_eq!(
            t.line_count(),
            expected.matches('\n').count() + 1,
            "line_count mismatch"
        );
        let expected_breaks: Vec<usize> = expected.match_indices('\n').map(|(i, _)| i).collect();
        assert_eq!(t.line_breaks(), expected_breaks, "line_breaks mismatch");
        for (i, p) in t.pieces.iter().enumerate() {
            assert!(p.len > 0, "piece {i} is zero-length: {p:?}");
        }
    }

    // ---- empty / errors ----

    #[test]
    fn delete_empty_range_is_noop() {
        let mut t = pt("hello");
        let pieces_before = t.pieces.len();
        t.delete(2..2).unwrap();
        assert_doc(&mut t, "hello");
        assert_eq!(t.pieces.len(), pieces_before, "no-op must not touch pieces");
    }

    #[test]
    fn delete_end_past_doc_len_errors() {
        let mut t = pt("hello");
        let err = t.delete(0..6).unwrap_err();
        assert!(
            matches!(err, TextError::IndexOutOfBounds { index: 6, len: 5 }),
            "unexpected error: {err:?}"
        );
        assert_doc(&mut t, "hello");
    }

    #[test]
    fn delete_reversed_range_errors() {
        let mut t = pt("hello");
        // Bind first: a literal `3..2` is a clippy `reversed_empty_ranges` error.
        let (start, end) = (3, 2);
        assert!(t.delete(start..end).is_err());
        assert_doc(&mut t, "hello");
    }

    #[test]
    fn delete_inside_multibyte_char_errors() {
        let mut t = pt("aébc");
        let err = t.delete(1..2).unwrap_err();
        assert!(
            matches!(
                err,
                TextError::InvalidDeleteRange {
                    start: 1,
                    end: 2,
                    reason: PositionError::NotACharBoundary,
                }
            ),
            "unexpected error: {err:?}"
        );
        // A rejected delete must leave the document untouched.
        assert_doc(&mut t, "aébc");
    }

    // ---- whole document / edges ----

    #[test]
    fn delete_whole_document_leaves_it_empty() {
        let mut t = pt("hello\nworld");
        t.delete(0..11).unwrap();
        assert_doc(&mut t, "");
        assert!(t.pieces.is_empty(), "pieces: {:?}", t.pieces);
        assert_eq!(t.line_count(), 1);
    }

    #[test]
    fn delete_at_document_start() {
        let mut t = pt("hello world");
        t.delete(0..6).unwrap();
        assert_doc(&mut t, "world");
    }

    #[test]
    fn delete_at_document_end() {
        let mut t = pt("hello world");
        t.delete(5..11).unwrap();
        assert_doc(&mut t, "hello");
    }

    #[test]
    fn delete_inside_one_piece_splits_it() {
        let mut t = pt("abcdef");
        t.delete(2..4).unwrap();
        assert_doc(&mut t, "abef");
        assert_eq!(t.pieces.len(), 2, "expected left and right keep pieces");
    }

    // ---- multi-piece documents ----

    #[test]
    fn delete_spanning_original_and_additive_pieces() {
        // pieces: [Original "abc"][Additive "def"]
        let mut t = pt("abc");
        t.insert("def", 3).unwrap();
        t.delete(2..5).unwrap();
        assert_doc(&mut t, "abf");
    }

    #[test]
    fn delete_exactly_one_piece() {
        let mut t = pt("abc");
        t.insert("def", 3).unwrap();
        t.delete(3..6).unwrap();
        assert_doc(&mut t, "abc");
        assert_eq!(t.pieces.len(), 1, "additive piece should be gone");
    }

    #[test]
    fn delete_across_three_pieces() {
        // Build [Original "abc"][Additive "d"][Additive "X"][Additive "ef"].
        let mut t = pt("abc");
        t.insert("def", 3).unwrap();
        t.insert("X", 4).unwrap();
        assert_eq!(t.text(), "abcdXef");
        assert_eq!(t.pieces.len(), 4);

        t.delete(2..6).unwrap();
        assert_doc(&mut t, "abf");
    }

    #[test]
    fn delete_leaves_neighbouring_survivors_in_order() {
        let mut t = pt("one two three");
        t.insert("!", 7).unwrap(); // "one two! three"
        t.delete(3..7).unwrap(); // drop " two"
        assert_doc(&mut t, "one! three");
    }

    // ---- line breaks ----

    #[test]
    fn delete_newline_updates_line_breaks() {
        let mut t = pt("a\nb\nc");
        t.delete(1..2).unwrap();
        assert_doc(&mut t, "ab\nc");
    }

    #[test]
    fn delete_line_leaves_adjacent_lines_joined() {
        let mut t = pt("first\nsecond\nthird");
        // Delete "\nsecond"
        t.delete(5..12).unwrap();
        assert_doc(&mut t, "first\nthird");
        assert_eq!(t.line_count(), 2);
    }

    #[test]
    fn delete_trailing_newline() {
        let mut t = pt("abc\n");
        t.delete(3..4).unwrap();
        assert_doc(&mut t, "abc");
        assert_eq!(t.line_count(), 1);
    }

    // ---- unicode ----

    #[test]
    fn delete_multibyte_char() {
        let mut t = pt("aébc");
        t.delete(1..3).unwrap();
        assert_doc(&mut t, "abc");
    }

    #[test]
    fn delete_multibyte_text_spanning_pieces() {
        let mut t = pt("héllo");
        t.insert("wörld", 6).unwrap();
        t.delete(1..9).unwrap(); // drop "éllo" + "wö"
        assert_doc(&mut t, "hrld");
    }

    // ---- interaction with insert ----

    #[test]
    fn delete_then_insert_at_same_offset() {
        let mut t = pt("hello world");
        t.delete(6..11).unwrap();
        assert_doc(&mut t, "hello ");
        t.insert("there", 6).unwrap();
        assert_doc(&mut t, "hello there");
    }

    #[test]
    fn repeated_deletes_drain_document() {
        let mut t = pt("abcdefghij");
        let mut model = String::from("abcdefghij");
        while !model.is_empty() {
            t.delete(0..1).unwrap();
            model.remove(0);
            assert_eq!(t.text(), model);
        }
        assert_doc(&mut t, "");
    }

    // ---- model-based random ----

    #[test]
    fn random_insert_and_delete_against_string_model() {
        let mut t = pt("");
        let mut model = String::new();
        let mut seed: u64 = 0x0123_4567_89AB_CDEF;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        let fragments = ["a", "bc", "def", "\n", "héllo", "é"];

        for iter in 0..2000 {
            let len = model.len();
            if len == 0 || next() % 3 == 0 {
                // Insert on a char boundary.
                let mut pos = next() % (len + 1);
                while !model.is_char_boundary(pos) {
                    pos -= 1;
                }
                let frag = fragments[next() % fragments.len()];
                t.insert(frag, pos).unwrap();
                model.insert_str(pos, frag);
            } else {
                // Delete a non-empty, char-boundary-aligned range.
                let mut start = next() % len;
                while !model.is_char_boundary(start) {
                    start -= 1;
                }
                let mut end = start + next() % (len - start + 1);
                while !model.is_char_boundary(end) {
                    end += 1;
                }
                t.delete(start..end).unwrap();
                model.replace_range(start..end, "");
            }

            assert_eq!(t.text(), model, "mismatch at iteration {iter}");
            assert_eq!(t.len(), model.len(), "doc_len mismatch at {iter}");
            assert_eq!(
                t.line_count(),
                model.matches('\n').count() + 1,
                "line_count mismatch at {iter}"
            );
            for (i, p) in t.pieces.iter().enumerate() {
                assert!(p.len > 0, "zero-length piece {i} at iteration {iter}");
            }
        }
    }
}
