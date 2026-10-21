//! Text Buffer
//!
//! A piece-table-backed document with save state and origin. The text itself
//! lives in [`crate::piece_table::PieceTable`]; this type owns the document's
//! identity (file path or scratch name) and whether it differs from disk.
//!
//! Operations are split by area, mirroring the piece table: [`insert`],
//! [`delete`], [`line`](mod@line), and [`word`]. This module keeps
//! construction, metadata, text-level queries, and persistence.

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{DomainError, DomainResult};
use crate::piece_table::PieceTable;

pub mod delete;
pub mod insert;
pub mod line;
pub mod word;

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

    /// Convert a byte offset in the document into a character offset.
    pub fn byte_to_char_offset(&self, byte_offset: usize) -> usize {
        self.piece_table.byte_to_char_offset(byte_offset)
    }

    /// True if `offset` is a boundary between UTF-8 characters.
    pub fn is_char_boundary(&self, offset: usize) -> bool {
        self.piece_table.is_char_boundary(offset)
    }

    pub fn is_empty(&self) -> bool {
        self.piece_table.is_empty()
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
}
