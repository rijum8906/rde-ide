//! Text Buffer
//!
//! A piece-table-backed document with save state and origin.

use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use piece_table::PieceTable;

use crate::error::{DomainError, DomainResult};

pub mod piece_table;

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
            piece_table: PieceTable::new(),
            origin: Origin::Scratch { name: name.into() },
            save_state: SaveState::NeverSaved,
        }
    }

    pub fn with_contents(path: impl Into<PathBuf>, contents: &str) -> Self {
        Self {
            piece_table: PieceTable::from_str(contents),
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

    pub fn line_count(&self) -> usize {
        self.piece_table.line_count()
    }

    pub fn line(&self, idx: usize) -> Option<String> {
        self.piece_table.get_line_contents(idx)
    }

    pub fn line_range(&self, idx: usize) -> Option<Range<usize>> {
        self.piece_table.line_range(idx)
    }

    pub fn char_to_byte_offset(&self, char_offset: usize) -> usize {
        self.piece_table.char_to_byte_offset(char_offset)
    }

    pub fn line_char_len(&self, idx: usize) -> usize {
        self.line(idx).map(|s| s.chars().count()).unwrap_or(0)
    }

    pub fn line_byte_len(&self, idx: usize) -> usize {
        self.line(idx).map(|s| s.len()).unwrap_or(0)
    }

    pub fn byte_len(&self) -> usize {
        self.piece_table.len()
    }

    pub fn char_count(&self) -> usize {
        self.piece_table.char_count()
    }

    pub fn is_empty(&self) -> bool {
        self.piece_table.is_empty()
    }

    pub fn to_string(&self) -> String {
        self.piece_table.to_string()
    }

    pub fn is_char_boundary(&self, offset: usize) -> bool {
        self.piece_table.is_char_boundary(offset)
    }

    pub fn insert(&mut self, offset: usize, text: &str) -> DomainResult<()> {
        self.piece_table.insert(offset, text)?;
        if !text.is_empty() {
            self.mark_dirty();
        }
        Ok(())
    }

    pub fn delete(&mut self, range: Range<usize>) -> DomainResult<()> {
        if range.start == range.end {
            return Ok(());
        }
        self.piece_table.delete(range)?;
        self.mark_dirty();
        Ok(())
    }

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
