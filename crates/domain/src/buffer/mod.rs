//! Text Buffer
//!
//! A piece-table-backed document with save state and origin.

use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use piece_table::PieceTable;

use crate::error::{IdeError, Result};

pub mod piece_table;

/// Where a buffer's content came from / will go to.
#[derive(Debug, Clone)]
enum Origin {
    /// Backed by a file on disk.
    File { path: PathBuf },
    /// A scratch buffer that has never been associated with a file.
    Scratch { name: String },
}

/// Whether the buffer's in-memory state matches what's on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveState {
    /// Never written to disk.
    NeverSaved,
    /// In-memory content matches disk.
    Clean,
    /// In-memory content differs from disk (or from creation).
    Dirty,
}

/// A single document.
#[derive(Debug, Clone)]
pub struct Buffer {
    piece_table: PieceTable,
    origin: Origin,
    save_state: SaveState,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            piece_table: PieceTable::new(),
            origin: Origin::Scratch {
                name: "Unnamed".to_string(),
            },
            save_state: SaveState::NeverSaved,
        }
    }
}

impl Buffer {
    // --- Constructors -----------------------------------------------------

    /// Create an empty scratch buffer with the given display name.
    pub fn scratch(name: impl Into<String>) -> Self {
        Self {
            piece_table: PieceTable::new(),
            origin: Origin::Scratch { name: name.into() },
            save_state: SaveState::NeverSaved,
        }
    }

    /// Create a buffer from in-memory contents (used by tests and by
    /// `open_file`).
    pub fn with_contents(path: impl Into<PathBuf>, contents: &str) -> Result<Self> {
        Ok(Self {
            piece_table: PieceTable::from_str(contents)?,
            origin: Origin::File { path: path.into() },
            save_state: SaveState::Clean,
        })
    }

    /// Open a file from disk. Reads the whole file eagerly; a real editor
    /// would memory-map or stream, but this is correct and simple.
    pub fn open_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|_| {
            // TODO: show debug message
            IdeError::CannotReadFile(path.to_path_buf())
        })?;

        let contents =
            String::from_utf8(bytes).map_err(|_| IdeError::InvalidUtf8(path.to_path_buf()))?;

        Self::with_contents(path.to_path_buf(), &contents)
    }

    // --- Queries ----------------------------------------------------------

    /// Name to display in the tab bar / status line.
    pub fn display_name(&self) -> &str {
        match &self.origin {
            Origin::File { path } => path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("[No Name]"),
            Origin::Scratch { name } => name,
        }
    }

    /// Path backing this buffer, if any.
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

    /// Total number of logical lines.
    pub fn line_count(&self) -> usize {
        self.piece_table.line_count()
    }

    /// The contents of the given line, if it exists.
    pub fn line(&self, idx: usize) -> Option<&str> {
        self.piece_table.get_line_contents(idx)
    }

    // --- Mutation ---------------------------------------------------------

    pub fn insert(&mut self, offset: usize, text: &str) {
        let _ = self.piece_table.insert(offset, text);
        self.mark_dirty();
    }

    pub fn delete(&mut self, range: Range<usize>) {
        let _ = self.piece_table.delete(range);
        self.mark_dirty();
    }

    /// Associate this buffer with a file path (e.g. on first "Save As").
    pub fn set_path(&mut self, path: PathBuf) {
        self.origin = Origin::File { path };
    }

    /// Mark the buffer as saved at its current path.
    pub fn mark_saved(&mut self) {
        if self.path().is_some() {
            self.save_state = SaveState::Clean;
        } else {
            self.save_state = SaveState::NeverSaved;
        }
    }

    fn mark_dirty(&mut self) {
        // Never downgrade from Dirty; Clean and NeverSaved both become Dirty.
        self.save_state = SaveState::Dirty;
    }
}
