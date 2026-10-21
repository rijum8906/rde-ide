//! Piece-table document storage.
//!
//! A [`PieceTable`] is the text engine behind [`crate::buffer::Buffer`]. It
//! keeps the loaded file (`original_text`) and user-typed text
//! (`additive_text`) immutable and describes the document as an ordered list
//! of [`Piece`]s that reference byte ranges in either buffer. Editing rewrites
//! the piece list instead of copying the document, so an insert or delete
//! costs O(pieces touched) rather than O(document length).
//!
//! Derived data (rendered text, char counts, line breaks) is computed from the
//! pieces on demand; only the line-break offsets are cached.

pub mod delete;
pub mod insert;
pub mod line;
pub mod word;

use std::cell::OnceCell;
use std::ops::Range;

/// Which backing buffer a piece references.
///
/// The piece table uses two immutable-after-load buffers:
/// - `Original` — bytes loaded from disk, never modified (allows sharing/mmap).
/// - `Additive` — append-only scratch for text the user types/pastes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PieceKind {
    Additive,
    Original,
}

/// A contiguous slice of one backing buffer, presented as part of the document.
///
/// `start` and `len` are byte offsets into the buffer indicated by `kind`.
/// A piece is a *reference*, never owning content — the bytes stay in the
/// backing buffer, which is why delete/undo are cheap.
#[derive(Debug, Copy, Clone)]
pub struct Piece {
    pub kind: PieceKind,
    pub start: usize,
    pub len: usize,
}

impl Piece {
    #[inline]
    pub fn end(&self) -> usize {
        self.start + self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[derive(Debug, Clone)]
pub struct PieceTable {
    /// The piece list, in document order. Concatenating pieces renders the document.
    pub pieces: Vec<Piece>,
    /// File contents as loaded. Never mutated after construction.
    pub original_text: String,
    /// Append-only buffer for user-inserted text.
    pub additive_text: String,

    // ----- Cache -----
    /// Cached rendered length in bytes. Must stay in sync with `pieces`.
    doc_len: usize,
    /// Byte offsets of each `\n` in the rendered document, in increasing
    /// order. Empty until the first read; edits replace it with a fresh
    /// empty cell, so a stale list is never observed.
    line_breaks: OnceCell<Vec<usize>>,

    /// Reusable scratch buffer for `delete` to avoid one allocation per call.
    scratch: Vec<Piece>,
}

// ============================ Constructors ============================

impl PieceTable {
    /// Build a table from an explicit piece list.
    ///
    /// `pieces` must be in document order and reference `original_text`.
    /// Zero-length pieces are dropped: they carry no text and the edit paths
    /// assume every piece is non-empty. The document length is derived from
    /// the pieces, and the line-break cache is rebuilt on first access.
    pub fn new(mut pieces: Vec<Piece>, original_text: String) -> Self {
        pieces.retain(|p| !p.is_empty());
        let doc_len = pieces.iter().map(|p| p.len).sum();
        Self {
            pieces,
            doc_len,
            line_breaks: OnceCell::new(),
            additive_text: String::new(),
            original_text,
            scratch: Vec::new(),
        }
    }
}

impl From<&str> for PieceTable {
    fn from(value: &str) -> Self {
        Self::new(
            vec![Piece {
                kind: PieceKind::Original,
                start: 0,
                len: value.len(),
            }],
            value.to_string(),
        )
    }
}

impl From<String> for PieceTable {
    fn from(value: String) -> Self {
        Self::new(
            vec![Piece {
                kind: PieceKind::Original,
                start: 0,
                len: value.len(),
            }],
            value,
        )
    }
}

// ============================ Getters ============================

impl PieceTable {
    /// The backing buffer a piece kind refers to.
    #[inline]
    fn source(&self, kind: PieceKind) -> &str {
        match kind {
            PieceKind::Additive => &self.additive_text,
            PieceKind::Original => &self.original_text,
        }
    }

    /// Render the entire document into a fresh `String`.
    ///
    /// O(total bytes). This is the canonical way to check the document's
    /// contents; the piece layout is an implementation detail.
    pub fn text(&self) -> String {
        self.slice(0..self.doc_len)
    }

    /// Render the document byte range `range` into a fresh `String`.
    ///
    /// `range` is clamped to the document. Both ends must be on char
    /// boundaries; callers get them from [`Self::line_range`], which is.
    /// O(bytes in the range + pieces before it).
    pub fn slice(&self, range: Range<usize>) -> String {
        let start = range.start.min(self.doc_len);
        let end = range.end.min(self.doc_len);
        if start >= end {
            return String::new();
        }

        let mut out = String::with_capacity(end - start);
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
            out.push_str(&src[piece.start + from..piece.start + to]);
        }
        out
    }

    /// Number of characters (not bytes) in the document. O(document).
    pub fn char_count(&self) -> usize {
        self.pieces
            .iter()
            .map(|piece| {
                let src = self.source(piece.kind);
                src[piece.start..piece.end()].chars().count()
            })
            .sum()
    }

    /// Convert a character offset into a byte offset.
    ///
    /// Offsets past the end are clamped to the document length, so callers do
    /// not have to pre-check bounds. O(document).
    pub fn char_to_byte_offset(&self, char_offset: usize) -> usize {
        let mut chars_seen = 0usize;
        let mut doc = 0usize;

        for piece in &self.pieces {
            let src = self.source(piece.kind);
            let piece_str = &src[piece.start..piece.end()];
            let piece_chars = piece_str.chars().count();

            if chars_seen + piece_chars >= char_offset {
                let local = piece_str
                    .char_indices()
                    .nth(char_offset - chars_seen)
                    .map(|(byte, _)| byte)
                    .unwrap_or(piece_str.len());
                return doc + local;
            }

            chars_seen += piece_chars;
            doc += piece.len;
        }

        self.doc_len
    }

    /// Convert a byte offset into a character offset. O(document).
    ///
    /// Offsets past the end are clamped to the document length.
    pub fn byte_to_char_offset(&self, byte_offset: usize) -> usize {
        let limit = byte_offset.min(self.doc_len);
        let mut chars = 0usize;
        let mut doc = 0usize;

        for piece in &self.pieces {
            let piece_end = doc + piece.len;
            let src = self.source(piece.kind);

            if limit >= piece_end {
                chars += src[piece.start..piece.end()].chars().count();
                doc = piece_end;
                continue;
            }

            if limit > doc {
                let local = limit - doc;
                chars += src[piece.start..piece.start + local].chars().count();
            }
            break;
        }

        chars
    }

    /// Number of lines (newline count + 1).
    ///
    /// An empty document has one (empty) line, matching [`Self::line_range`].
    /// The first call rebuilds the line-break cache in O(document); later
    /// calls are O(1) until the next edit.
    pub fn line_count(&self) -> usize {
        self.line_breaks().len() + 1
    }

    /// Byte offsets of every `\n` in the document, in increasing order.
    ///
    /// Computed from the pieces on first access after an edit, then cached.
    pub fn line_breaks(&self) -> &[usize] {
        self.line_breaks.get_or_init(|| self.compute_line_breaks())
    }

    /// Rebuild the line-break list from the current pieces. O(total bytes).
    fn compute_line_breaks(&self) -> Vec<usize> {
        let mut breaks = Vec::new();
        let mut doc_offset = 0usize;

        for piece in &self.pieces {
            let src = self.source(piece.kind);
            debug_assert!(
                src.is_char_boundary(piece.start) && src.is_char_boundary(piece.end()),
                "piece {:?} is not on char boundaries",
                piece
            );
            for (i, _) in src[piece.start..piece.end()].match_indices('\n') {
                breaks.push(doc_offset + i);
            }
            doc_offset += piece.len;
        }

        breaks
    }

    /// Document length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.doc_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.doc_len == 0
    }

    /// Replace the whole document with `text`.
    ///
    /// The old pieces are dropped, so no history or backing text survives.
    pub fn replace_all(&mut self, text: &str) {
        self.pieces = if text.is_empty() {
            Vec::new()
        } else {
            vec![Piece {
                kind: PieceKind::Original,
                start: 0,
                len: text.len(),
            }]
        };
        self.doc_len = text.len();
        self.original_text.clear();
        self.original_text.push_str(text);
        self.additive_text.clear();
        self.scratch.clear();
        self.line_breaks = OnceCell::new();
    }
}

// ============================ Insert and Delete helpers ============================

/// Push `p` onto `out`, merging with the last piece when the two are
/// adjacent in the same backing buffer.
///
/// This keeps the piece count low when many small inserts accumulate,
/// which keeps every O(pieces) operation cheap.
#[inline]
pub(crate) fn push_or_merge(out: &mut Vec<Piece>, p: Piece) {
    if let Some(last) = out.last_mut() {
        if last.kind == p.kind && last.start + last.len == p.start {
            last.len += p.len;
            return;
        }
    }
    out.push(p);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_count_reflects_edits() {
        let mut t = PieceTable::from("a\nb"); // 2 lines
        assert_eq!(t.line_count(), 2);
        let res = t.insert("\n", 1); // "a\n\nb" -> 3 lines
        assert!(res.is_ok());
        assert_eq!(t.line_count(), 3, "line_count must reflect inserts");
        let res = t.delete(1..2); // back to "a\nb"
        assert!(res.is_ok());
        assert_eq!(t.line_count(), 2, "line_count must reflect deletes");
    }
}
