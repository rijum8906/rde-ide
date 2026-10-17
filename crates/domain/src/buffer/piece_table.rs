//! Piece Table
//!
//! A piece table is an immutable text buffer plus an append-only add buffer,
//! with a list of "pieces" that reference ranges in either. Insert and delete
//! are O(number of pieces touched) rather than O(document length), which makes
//! them cheap regardless of how big the file is.
//!
//! ## Layout
//!
//! ```text
//!   original:  "Hello, world\nSecond line\n"
//!              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  (never mutated)
//!
//!   add:       ""                          (append-only; grows on insert)
//!
//!   pieces:    [ (Original, 0..27) ]       (logical view: the whole doc)
//! ```
//!
//! Inserting "X" at offset 5:
//!
//! ```text
//!   original:  "Hello, world\nSecond line\n"
//!   add:       "X"
//!   pieces:    [ (Original, 0..5), (Add, 0..1), (Original, 5..27) ]
//! ```
//!
//! Deleting a range removes or trims the pieces that overlap it.
//!
//! ## Complexity
//!
//! | Operation      | Cost                                          |
//! |----------------|-----------------------------------------------|
//! | `insert`       | O(pieces) to find the splice point            |
//! | `delete`       | O(pieces) to find and trim overlapping pieces |
//! | `char_at`      | O(pieces) to locate the piece, then O(1)      |
//! | `to_string`    | O(total length)                               |
//! | `line_count`   | O(total length) unless cached                 |
//!
//! For a text editor, this is fine. If you ever need O(log n) operations,
//! swap the `Vec<Piece>` for a balanced tree (splay/AVL) keyed by cumulative
//! length. Everything below the `PieceTable` API stays the same.

use std::ops::Range;

use crate::error::{DomainError, DomainResult, IdeError, Result};

/// Which buffer a piece refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    /// The original document text. Never mutated.
    Original,
    /// The append-only buffer that receives new text from inserts.
    Add,
}

/// A single contiguous span of text, identified by its source and range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Piece {
    source: Source,
    /// Byte offset into the chosen source buffer.
    start: usize,
    /// Byte length of the span.
    len: usize,
}

impl Piece {
    fn end(&self) -> usize {
        self.start + self.len
    }

    /// The byte range within the source buffer.
    fn range(&self) -> Range<usize> {
        self.start..self.end()
    }
}

/// A piece-table-backed text buffer.
#[derive(Debug, Clone)]
pub struct PieceTable {
    /// The original document text, never mutated after construction.
    original: String,

    /// Append-only buffer. Inserted text is appended here; pieces reference
    /// ranges of it.
    add: String,

    /// The logical view of the document, in order.
    pieces: Vec<Piece>,

    /// Cached total byte length of all pieces. Kept in sync on every
    /// mutation so `len()` is O(1).
    total_len: usize,
}

impl Default for PieceTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceTable {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    /// Create an empty piece table.
    pub fn new() -> Self {
        Self {
            original: String::new(),
            add: String::new(),
            pieces: Vec::new(),
            total_len: 0,
        }
    }

    /// Create a piece table from the given text.
    ///
    /// The text becomes the "original" buffer; the piece list is a single
    /// piece covering all of it.
    pub fn from_str(text: &str) -> Self {
        if text.is_empty() {
            return Self::new();
        }
        Self {
            original: text.to_string(),
            add: String::new(),
            pieces: vec![Piece {
                source: Source::Original,
                start: 0,
                len: text.len(),
            }],
            total_len: text.len(),
        }
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    /// Total byte length of the logical document.
    pub fn len(&self) -> usize {
        self.total_len
    }

    /// True if the document is empty.
    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }

    /// Total character count. O(n) — walks the pieces.
    pub fn char_count(&self) -> usize {
        self.chars().count()
    }

    /// Number of logical lines. A line ends at `\n`; the final line is
    /// counted even if it doesn't end with `\n`.
    ///
    /// An empty buffer has 0 lines. A buffer with one `\n` has 2 lines
    /// (the part before and the empty part after).
    ///
    /// O(n).
    pub fn line_count(&self) -> usize {
        if self.total_len == 0 {
            return 0;
        }
        // Every '\n' starts a new line, plus the first line.
        let newlines = self.chars().filter(|&c| c == '\n').count();
        newlines + 1
    }

    /// The contents of the given line, without the trailing newline.
    ///
    /// Returns `None` if `idx` is out of range.
    ///
    /// O(n) in the worst case because it materializes the line. If you call
    /// this for every line in a render loop, add a line index cache.
    pub fn get_line_contents(&self, idx: usize) -> Option<String> {
        self.line_range(idx).map(|range| self.slice_chars(range))
    }

    /// The character range (start..end, exclusive) of the given line,
    /// measured in **characters**, not bytes.
    pub fn line_range(&self, idx: usize) -> Option<Range<usize>> {
        if self.total_len == 0 {
            return None;
        }

        let mut line_start_char = 0usize;
        let mut current_line = 0usize;
        let mut char_pos = 0usize;

        for ch in self.chars() {
            if current_line == idx {
                // We're inside the target line. Find where it ends.
                return Some(self.find_line_end_from(char_pos, line_start_char));
            }
            if ch == '\n' {
                current_line += 1;
                line_start_char = char_pos + 1;
                if current_line == idx {
                    // Target line starts here; find its end.
                    return Some(self.find_line_end_from(char_pos + 1, line_start_char));
                }
            }
            char_pos += 1;
        }

        // Edge case: the last line (no trailing newline).
        if current_line == idx {
            Some(line_start_char..char_pos)
        } else {
            None
        }
    }

    /// Starting at character position `from_char`, find the character
    /// range of the line that starts at `line_start_char`.
    fn find_line_end_from(&self, from_char: usize, line_start_char: usize) -> Range<usize> {
        let mut char_pos = from_char;
        for ch in self.chars().skip(from_char) {
            if ch == '\n' {
                return line_start_char..char_pos;
            }
            char_pos += 1;
        }
        // Last line without trailing newline.
        line_start_char..char_pos
    }

    // ------------------------------------------------------------------
    // Mutation
    // ------------------------------------------------------------------

    /// Insert `text` at the given **byte** offset.
    ///
    /// Returns an error if `offset` is not a valid UTF-8 boundary or is
    /// out of bounds.
    pub fn insert(&mut self, offset: usize, text: &str) -> DomainResult<()> {
        if text.is_empty() {
            return Ok(());
        }
        if offset > self.total_len {
            return Err(DomainError::OffsetOutOfBounds {
                offset,
                len: self.total_len,
            });
        }
        if !self.is_char_boundary(offset) {
            return Err(DomainError::NotCharBoundary { offset });
        }

        // Append the text to the add buffer.
        let add_start = self.add.len();
        self.add.push_str(text);

        // Create a piece for it.
        let new_piece = Piece {
            source: Source::Add,
            start: add_start,
            len: text.len(),
        };

        // Find the piece and byte offset where the insert lands.
        let (piece_idx, byte_in_piece) = self.locate(offset);

        if piece_idx >= self.pieces.len() {
            // Insert at the very end.
            self.pieces.push(new_piece);
        } else {
            let target = self.pieces[piece_idx];
            if byte_in_piece == 0 {
                // Insert before the target piece.
                self.pieces.insert(piece_idx, new_piece);
            } else if byte_in_piece == target.len {
                // Insert right after the target piece (which is fine —
                // just insert at the next index).
                self.pieces.insert(piece_idx + 1, new_piece);
            } else {
                // Split the target piece into two, and insert the new piece
                // between them.
                let left = Piece {
                    source: target.source,
                    start: target.start,
                    len: byte_in_piece,
                };
                let right = Piece {
                    source: target.source,
                    start: target.start + byte_in_piece,
                    len: target.len - byte_in_piece,
                };
                self.pieces
                    .splice(piece_idx..=piece_idx, [left, new_piece, right]);
            }
        }

        self.total_len += text.len();
        Ok(())
    }

    /// Delete the given **byte** range.
    ///
    /// Returns an error if the range is out of bounds or not on char
    /// boundaries.
    pub fn delete(&mut self, range: Range<usize>) -> DomainResult<()> {
        let start = range.start;
        let end = range.end;

        if start > end {
            return Err(DomainError::InvalidRange { start, end });
        }
        if end > self.total_len {
            return Err(DomainError::OffsetOutOfBounds {
                offset: end,
                len: self.total_len,
            });
        }
        if start == end {
            return Ok(());
        }
        if !self.is_char_boundary(start) {
            return Err(DomainError::NotCharBoundary { offset: start });
        }
        if !self.is_char_boundary(end) {
            return Err(DomainError::NotCharBoundary { offset: end });
        }

        // Walk pieces, keeping/trimming/splitting as needed.
        let mut new_pieces: Vec<Piece> = Vec::with_capacity(self.pieces.len() + 2);
        let mut byte_cursor = 0usize;

        for &piece in &self.pieces {
            let piece_start = byte_cursor;
            let piece_end = byte_cursor + piece.len;

            if piece_end <= start || piece_start >= end {
                // Piece is entirely outside the delete range.
                new_pieces.push(piece);
            } else if piece_start >= start && piece_end <= end {
                // Piece is entirely inside the delete range. Drop it.
            } else {
                // Piece straddles a boundary. Keep the parts outside.
                if piece_start < start {
                    let keep_len = start - piece_start;
                    new_pieces.push(Piece {
                        source: piece.source,
                        start: piece.start,
                        len: keep_len,
                    });
                }
                if piece_end > end {
                    let trim = end - piece_start;
                    new_pieces.push(Piece {
                        source: piece.source,
                        start: piece.start + trim,
                        len: piece.len - trim,
                    });
                }
            }

            byte_cursor = piece_end;
        }

        self.pieces = new_pieces;
        self.total_len -= end - start;
        Ok(())
    }

    /// Replace the entire buffer contents.
    pub fn replace_all(&mut self, text: &str) {
        *self = Self::from_str(text);
    }

    // ------------------------------------------------------------------
    // Access
    // ------------------------------------------------------------------

    /// The full document as a `String`.
    pub fn to_string(&self) -> String {
        let mut out = String::with_capacity(self.total_len);
        for piece in &self.pieces {
            out.push_str(self.piece_str(piece));
        }
        out
    }

    /// Iterate the logical characters of the document.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.pieces.iter().flat_map(|p| self.piece_str(p).chars())
    }

    /// The character at the given **character** offset (not byte).
    pub fn char_at(&self, char_offset: usize) -> Option<char> {
        self.chars().nth(char_offset)
    }

    /// The byte at the given byte offset.
    pub fn byte_at(&self, byte_offset: usize) -> Option<u8> {
        if byte_offset >= self.total_len {
            return None;
        }
        let mut cursor = 0usize;
        for piece in &self.pieces {
            if byte_offset < cursor + piece.len {
                let local = byte_offset - cursor;
                return self.piece_str(piece).as_bytes().get(local).copied();
            }
            cursor += piece.len;
        }
        None
    }

    /// True if the given byte offset is on a UTF-8 char boundary.
    pub fn is_char_boundary(&self, byte_offset: usize) -> bool {
        if byte_offset == 0 || byte_offset == self.total_len {
            return true;
        }
        if byte_offset > self.total_len {
            return false;
        }
        // A char boundary is one where the next byte is NOT a continuation
        // byte (0b10xxxxxx).
        match self.byte_at(byte_offset) {
            Some(b) => (b & 0b1100_0000) != 0b1000_0000,
            None => false,
        }
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Borrow the string backing a piece.
    fn piece_str(&self, piece: &Piece) -> &str {
        let source = match piece.source {
            Source::Original => &self.original,
            Source::Add => &self.add,
        };
        // Slicing a String by a known-valid range. Pieces are always
        // created from valid ranges of valid UTF-8, so this is safe.
        &source[piece.range()]
    }

    /// Find which piece contains the given byte offset, and the offset
    /// *within* that piece.
    ///
    /// If `offset == total_len`, returns `(pieces.len(), 0)` to signal
    /// "past the end."
    fn locate(&self, offset: usize) -> (usize, usize) {
        if offset == self.total_len {
            return (self.pieces.len(), 0);
        }
        let mut cursor = 0usize;
        for (i, piece) in self.pieces.iter().enumerate() {
            if offset < cursor + piece.len {
                return (i, offset - cursor);
            }
            cursor += piece.len;
        }
        // Shouldn't happen if offset <= total_len, but be defensive.
        (self.pieces.len(), 0)
    }

    /// Extract a range of characters as a `String`.
    fn slice_chars(&self, range: Range<usize>) -> String {
        self.chars()
            .skip(range.start)
            .take(range.end - range.start)
            .collect()
    }
}

impl std::str::FromStr for PieceTable {
    type Err = IdeError;
    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::from_str(s))
    }
}
