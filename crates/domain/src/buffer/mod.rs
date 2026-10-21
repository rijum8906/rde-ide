//! Text Buffer
//!
//! A piece-table-backed document with save state and origin. The text itself
//! lives in [`crate::piece_table::PieceTable`]; this type owns the document's
//! identity (file path or scratch name) and whether it differs from disk.

use std::fmt::Display;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::error::{DomainError, DomainResult};
use crate::piece_table::PieceTable;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Origin {
    File { path: PathBuf },
    Scratch { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveState {
    NeverSaved,
    Clean,
    Dirty,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    piece_table: PieceTable,
    origin: Origin,
    save_state: SaveState,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::scratch("Unnamed")
    }
}

impl Buffer {
    pub fn scratch(name: impl Into<String>) -> Self {
        Self {
            piece_table: PieceTable::from(""),
            origin: Origin::Scratch { name: name.into() },
            save_state: SaveState::NeverSaved,
        }
    }

    pub fn with_contents(path: impl Into<PathBuf>, contents: &str) -> Self {
        Self {
            piece_table: PieceTable::from(contents),
            origin: Origin::File { path: path.into() },
            save_state: SaveState::Clean,
        }
    }

    pub fn open_file(path: impl AsRef<Path>) -> DomainResult<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| DomainError::CannotReadFile {
            path: path.to_path_buf(),
            source,
        })?;

        let contents = String::from_utf8(bytes).map_err(|_| DomainError::InvalidUtf8 {
            path: path.to_path_buf(),
        })?;

        Ok(Self::with_contents(path.to_path_buf(), &contents))
    }

    pub fn display_name(&self) -> &str {
        match &self.origin {
            Origin::File { path } => path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("[No Name]"),
            Origin::Scratch { name } => name,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match &self.origin {
            Origin::File { path } => Some(path),
            Origin::Scratch { .. } => None,
        }
    }

    pub fn save_state(&self) -> SaveState {
        self.save_state
    }

    pub fn is_dirty(&self) -> bool {
        self.save_state == SaveState::Dirty
    }

    pub fn is_scratch(&self) -> bool {
        matches!(self.origin, Origin::Scratch { .. })
    }

    /// Number of lines in the document. An empty document counts as one
    /// (empty) line.
    pub fn line_count(&self) -> usize {
        self.piece_table.line_count()
    }

    /// Character range of line `idx`, excluding the trailing newline.
    ///
    /// The editor's cursor model is character-based, so this is converted from
    /// the piece table's byte range. `None` if the line does not exist.
    pub fn line_range(&self, idx: usize) -> Option<Range<usize>> {
        let range = self.piece_table.line_range(idx)?;
        Some(
            self.piece_table.byte_to_char_offset(range.start)
                ..self.piece_table.byte_to_char_offset(range.end),
        )
    }

    /// Text of line `idx` without its trailing newline, or `None` if the line
    /// does not exist.
    pub fn line(&self, idx: usize) -> Option<String> {
        self.piece_table.line(idx)
    }

    /// Number of characters in line `idx` (0 if the line does not exist).
    pub fn line_char_len(&self, idx: usize) -> usize {
        self.line(idx).map(|line| line.chars().count()).unwrap_or(0)
    }

    /// Number of bytes in line `idx` (0 if the line does not exist).
    pub fn line_byte_len(&self, idx: usize) -> usize {
        self.line(idx).map(|line| line.len()).unwrap_or(0)
    }

    /// Document length in bytes.
    pub fn byte_len(&self) -> usize {
        self.piece_table.len()
    }

    /// Number of characters (not bytes) in the document.
    pub fn char_count(&self) -> usize {
        self.piece_table.char_count()
    }

    /// Convert a character offset in the document into a byte offset.
    pub fn char_to_byte_offset(&self, char_offset: usize) -> usize {
        self.piece_table.char_to_byte_offset(char_offset)
    }

    /// True if `offset` is a boundary between UTF-8 characters.
    pub fn is_char_boundary(&self, offset: usize) -> bool {
        self.piece_table.is_char_boundary(offset)
    }

    pub fn is_empty(&self) -> bool {
        self.piece_table.is_empty()
    }

    /// Insert `text` at byte `offset`. Empty text is a successful no-op.
    pub fn insert(&mut self, offset: usize, text: &str) -> DomainResult<()> {
        if text.is_empty() {
            return Ok(());
        }
        self.piece_table.insert(text, offset)?;
        self.mark_dirty();
        Ok(())
    }

    /// Delete the byte range `range`. The range must lie on char boundaries.
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

    pub fn set_path(&mut self, path: PathBuf) {
        let was_scratch = self.is_scratch();
        self.origin = Origin::File { path };

        if was_scratch {
            match self.save_state {
                SaveState::NeverSaved if self.piece_table.is_empty() => {
                    self.save_state = SaveState::Clean;
                }
                SaveState::NeverSaved => {
                    self.save_state = SaveState::Dirty;
                }
                other => {
                    self.save_state = other;
                }
            }
        }
    }

    pub fn save(&mut self) -> DomainResult<()> {
        let Some(path) = self.path().map(|p| p.to_path_buf()) else {
            return Err(DomainError::NoPath);
        };
        self.save_to_path(&path)
    }

    pub fn save_to_path(&mut self, path: &Path) -> DomainResult<()> {
        let contents = self.to_string();
        fs::write(path, contents.as_bytes()).map_err(|source| DomainError::CannotWriteFile {
            path: path.to_path_buf(),
            source,
        })?;

        if self.path() != Some(path) {
            self.origin = Origin::File {
                path: path.to_path_buf(),
            };
        }
        self.save_state = SaveState::Clean;
        Ok(())
    }

    pub fn rename_to(&mut self, new_path: PathBuf) -> DomainResult<()> {
        let Some(old_path) = self.path().map(|p| p.to_path_buf()) else {
            return Err(DomainError::NoPath);
        };
        fs::rename(&old_path, &new_path).map_err(|source| DomainError::CannotRename {
            from: old_path,
            to: new_path.clone(),
            source,
        })?;
        self.origin = Origin::File { path: new_path };
        Ok(())
    }

    fn mark_dirty(&mut self) {
        self.save_state = SaveState::Dirty;
    }
}

impl Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.piece_table.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_go_through_the_piece_table() {
        let mut buffer = Buffer::scratch("scratch");
        assert!(buffer.is_empty());

        buffer.insert(0, "abc").unwrap();
        buffer.insert(3, "d").unwrap();
        assert_eq!(buffer.to_string(), "abcd");

        buffer.delete(1..3).unwrap();
        assert_eq!(buffer.to_string(), "ad");
        assert_eq!(buffer.byte_len(), 2);
        assert_eq!(buffer.char_count(), 2);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn empty_edits_are_noops_and_keep_save_state() {
        let mut buffer = Buffer::with_contents("test.txt", "abc");
        buffer.insert(1, "").unwrap();
        buffer.delete(1..1).unwrap();

        assert_eq!(buffer.to_string(), "abc");
        assert_eq!(buffer.save_state(), SaveState::Clean);
    }

    #[test]
    fn empty_document_has_one_empty_line() {
        let buffer = Buffer::scratch("scratch");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line(0).as_deref(), Some(""));
        assert_eq!(buffer.line_range(0), Some(0..0));
        assert_eq!(buffer.line(1), None);
    }

    #[test]
    fn line_queries_are_character_based_for_multibyte_text() {
        let buffer = Buffer::with_contents("test.txt", "héllo\nwörld");

        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line(0).as_deref(), Some("héllo"));
        assert_eq!(buffer.line(1).as_deref(), Some("wörld"));

        // "héllo" is 5 chars / 6 bytes, so line 1 starts at char 6 (byte 7).
        assert_eq!(buffer.line_range(0), Some(0..5));
        assert_eq!(buffer.line_range(1), Some(6..11));
        assert_eq!(buffer.line_char_len(0), 5);
        assert_eq!(buffer.line_byte_len(0), 6);

        // Char 2 is the "é": it starts at byte 1 and ends on byte 3.
        assert_eq!(buffer.char_to_byte_offset(2), 3);
        assert!(!buffer.is_char_boundary(2));
        assert!(buffer.is_char_boundary(3));
    }

    #[test]
    fn inserting_newlines_updates_lines_and_ranges() {
        let mut buffer = Buffer::with_contents("test.txt", "one\ntwo");
        assert_eq!(buffer.line_count(), 2);

        buffer.insert(3, "\n\n").unwrap(); // "one\n\n\ntwo"
        assert_eq!(buffer.line_count(), 4);
        assert_eq!(buffer.line(1).as_deref(), Some(""));
        assert_eq!(buffer.line(2).as_deref(), Some(""));
        assert_eq!(buffer.line(3).as_deref(), Some("two"));
        assert_eq!(buffer.line_range(3), Some(6..9));
        assert!(buffer.is_dirty());

        buffer.delete(3..5).unwrap(); // back to "one\ntwo"
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line_range(1), Some(4..7));
    }

    #[test]
    fn delete_rejects_splitting_a_multibyte_char() {
        let mut buffer = Buffer::with_contents("test.txt", "é");
        assert!(buffer.delete(1..2).is_err());
        assert_eq!(buffer.to_string(), "é");
        assert_eq!(buffer.save_state(), SaveState::Clean);
    }

    #[test]
    fn insert_past_end_is_reported() {
        let mut buffer = Buffer::with_contents("test.txt", "abc");
        assert!(buffer.insert(4, "x").is_err());
        assert_eq!(buffer.to_string(), "abc");
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
