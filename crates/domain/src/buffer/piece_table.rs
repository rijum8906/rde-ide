//! Piece Table
//!
//! A piece table is an immutable text buffer plus an append-only add buffer,
//! with a list of "pieces" that reference ranges in either. Insert and delete
//! are O(number of pieces touched) rather than O(document length).

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
    original: String,
    add: String,
    pieces: Vec<Piece>,
    total_len: usize,
}

impl Default for PieceTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceTable {
    pub fn new() -> Self {
        Self {
            original: String::new(),
            add: String::new(),
            pieces: Vec::new(),
            total_len: 0,
        }
    }

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

    pub fn len(&self) -> usize {
        self.total_len
    }

    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }

    pub fn char_count(&self) -> usize {
        self.chars().count()
    }

    pub fn line_count(&self) -> usize {
        if self.total_len == 0 {
            return 0;
        }
        let newlines = self.chars().filter(|&c| c == '\n').count();
        newlines + 1
    }

    pub fn get_line_contents(&self, idx: usize) -> Option<String> {
        self.line_range(idx).map(|range| self.slice_chars(range))
    }

    pub fn line_range(&self, idx: usize) -> Option<Range<usize>> {
        if self.total_len == 0 {
            return None;
        }

        let mut line_start_char = 0usize;
        let mut current_line = 0usize;
        let mut char_pos = 0usize;

        for ch in self.chars() {
            if current_line == idx {
                return Some(self.find_line_end_from(char_pos, line_start_char));
            }
            if ch == '\n' {
                current_line += 1;
                line_start_char = char_pos + 1;
                if current_line == idx {
                    return Some(self.find_line_end_from(char_pos + 1, line_start_char));
                }
            }
            char_pos += 1;
        }

        if current_line == idx {
            Some(line_start_char..char_pos)
        } else {
            None
        }
    }

    fn find_line_end_from(&self, from_char: usize, line_start_char: usize) -> Range<usize> {
        let mut char_pos = from_char;
        for ch in self.chars().skip(from_char) {
            if ch == '\n' {
                return line_start_char..char_pos;
            }
            char_pos += 1;
        }
        line_start_char..char_pos
    }

    /// Convert a character offset to a byte offset.
    pub fn char_to_byte_offset(&self, char_offset: usize) -> usize {
        self.chars().take(char_offset).map(|c| c.len_utf8()).sum()
    }

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

        let add_start = self.add.len();
        self.add.push_str(text);

        let new_piece = Piece {
            source: Source::Add,
            start: add_start,
            len: text.len(),
        };

        let (piece_idx, byte_in_piece) = self.locate(offset);

        if piece_idx >= self.pieces.len() {
            self.pieces.push(new_piece);
        } else {
            let target = self.pieces[piece_idx];
            if byte_in_piece == 0 {
                self.pieces.insert(piece_idx, new_piece);
            } else if byte_in_piece == target.len {
                self.pieces.insert(piece_idx + 1, new_piece);
            } else {
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

        let mut new_pieces: Vec<Piece> = Vec::with_capacity(self.pieces.len() + 2);
        let mut byte_cursor = 0usize;

        for &piece in &self.pieces {
            let piece_start = byte_cursor;
            let piece_end = byte_cursor + piece.len;

            if piece_end <= start || piece_start >= end {
                new_pieces.push(piece);
            } else if piece_start >= start && piece_end <= end {
                // Drop piece entirely
            } else {
                if piece_start < start {
                    let keep_len = start - piece_start;
                    new_pieces.push(Piece {
                        source: piece.source,
                        start: piece.start,
                        len: keep_len,
                    });
                }
                if piece_end > end {
                    let keep_start_doc = std::cmp::max(piece_start, end);
                    let trim_len = keep_start_doc - piece_start;
                    let keep_len = piece_end - keep_start_doc;
                    new_pieces.push(Piece {
                        source: piece.source,
                        start: piece.start + trim_len,
                        len: keep_len,
                    });
                }
            }

            byte_cursor = piece_end;
        }

        self.pieces = new_pieces;
        self.total_len -= end - start;
        Ok(())
    }

    pub fn replace_all(&mut self, text: &str) {
        *self = Self::from_str(text);
    }

    pub fn string(&self) -> String {
        let mut out = String::with_capacity(self.total_len);
        for piece in &self.pieces {
            out.push_str(self.piece_str(piece));
        }
        out
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.pieces.iter().flat_map(|p| self.piece_str(p).chars())
    }

    pub fn char_at(&self, char_offset: usize) -> Option<char> {
        self.chars().nth(char_offset)
    }

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

    pub fn is_char_boundary(&self, byte_offset: usize) -> bool {
        if byte_offset == 0 || byte_offset == self.total_len {
            return true;
        }
        if byte_offset > self.total_len {
            return false;
        }
        match self.byte_at(byte_offset) {
            Some(b) => (b & 0b1100_0000) != 0b1000_0000,
            None => false,
        }
    }

    fn piece_str(&self, piece: &Piece) -> &str {
        let source = match piece.source {
            Source::Original => &self.original,
            Source::Add => &self.add,
        };
        &source[piece.range()]
    }

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
        (self.pieces.len(), 0)
    }

    fn slice_chars(&self, range: Range<usize>) -> String {
        self.chars()
            .skip(range.start)
            .take(range.end - range.start)
            .collect()
    }
}

impl std::fmt::Display for PieceTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string())
    }
}

impl std::str::FromStr for PieceTable {
    type Err = IdeError;
    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::from_str(s))
    }
}
