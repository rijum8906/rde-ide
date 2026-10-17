//! Text Buffer
//!
//! A piece-table-backed document with save state and origin.
//!
//! `Buffer` is the *public face* of a document. It owns a `PieceTable`
//! (private), tracks where the content came from (`Origin`), and whether
//! it's been modified since the last save (`SaveState`).
//!
//! The piece table is not exposed. Every mutation goes through `Buffer`'s
//! command-style methods (`insert`, `delete`, `replace_all`, `save`) so
//! that invariants stay intact: a modified buffer is always marked dirty,
//! a saved buffer is always clean, and a buffer's display name always
//! matches its origin.

use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use piece_table::PieceTable;

use crate::error::{DomainError, DomainResult};

pub mod piece_table;

// ============================================================================
// Origin
// ============================================================================

/// Where a buffer's content came from and where it will be saved to.
///
/// Private to this module. Callers see `display_name()` and `path()`, which
/// derive from the origin without exposing it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Origin {
    /// Backed by a file on disk.
    File { path: PathBuf },
    /// A scratch buffer never associated with a file.
    Scratch { name: String },
}

// ============================================================================
// SaveState
// ============================================================================

/// Whether the buffer's in-memory state matches what's on disk.
///
/// - `NeverSaved` — a scratch buffer that has never been written.
/// - `Clean` — the in-memory content matches the file on disk.
/// - `Dirty` — the buffer has been modified since the last save.
///
/// A buffer transitions to `Dirty` on any edit, and to `Clean` after
/// `save`/`save_to_path` succeeds. It never transitions from `Dirty` to
/// `NeverSaved`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveState {
    NeverSaved,
    Clean,
    Dirty,
}

// ============================================================================
// Buffer
// ============================================================================

/// A single document.
///
/// Constructors:
/// - `Buffer::scratch(name)` — an unnamed, empty buffer.
/// - `Buffer::open_file(path)` — reads from disk.
/// - `Buffer::with_contents(path, text)` — in-memory, marked clean.
///
/// Mutators:
/// - `insert(offset, text)` — inserts at a byte offset and marks dirty.
/// - `delete(range)` — deletes a byte range and marks dirty.
/// - `replace_all(text)` — replaces the whole buffer and marks dirty.
/// - `save()` / `save_to_path(path)` — write to disk and mark clean.
///
/// Queries:
/// - `line_count()`, `line(idx)`, `line_char_len(idx)`, `line_byte_len(idx)`
/// - `byte_len()`, `char_count()`, `is_empty()`
/// - `to_string()` — the whole document.
/// - `display_name()`, `path()`, `save_state()`, `is_dirty()`, `is_scratch()`
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
    // ------------------------------------------------------------------
    // Constructors
    // ------------------------------------------------------------------

    /// Create an empty scratch buffer with the given display name.
    pub fn scratch(name: impl Into<String>) -> Self {
        Self {
            piece_table: PieceTable::new(),
            origin: Origin::Scratch { name: name.into() },
            save_state: SaveState::NeverSaved,
        }
    }

    /// Create a buffer from in-memory contents.
    ///
    /// The buffer is considered `Clean` — this constructor is used by
    /// `open_file` after reading from disk, and by tests that want a
    /// pre-populated buffer without disk I/O.
    pub fn with_contents(path: impl Into<PathBuf>, contents: &str) -> Self {
        Self {
            piece_table: PieceTable::from_str(contents),
            origin: Origin::File { path: path.into() },
            save_state: SaveState::Clean,
        }
    }

    /// Open a file from disk. Reads the whole file eagerly.
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

    // ------------------------------------------------------------------
    // Metadata
    // ------------------------------------------------------------------

    /// Name to display in the tab bar / status line.
    ///
    /// For file-backed buffers, this is the file's basename. For scratch
    /// buffers, it's the name given at construction.
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

    /// Current save state.
    pub fn save_state(&self) -> SaveState {
        self.save_state
    }

    /// Convenience: is the buffer modified since the last save?
    pub fn is_dirty(&self) -> bool {
        self.save_state == SaveState::Dirty
    }

    /// Convenience: is this buffer unbacked by a file?
    pub fn is_scratch(&self) -> bool {
        matches!(self.origin, Origin::Scratch { .. })
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    /// Total number of logical lines.
    ///
    /// An empty buffer has 0 lines. A buffer ending in `\n` has one more
    /// line than it has newlines (the trailing empty line).
    pub fn line_count(&self) -> usize {
        self.piece_table.line_count()
    }

    /// The contents of the given line, without a trailing newline.
    ///
    /// Allocates a `String` because a line may span multiple pieces. If
    /// you call this in a hot loop, consider adding a line-index cache to
    /// `PieceTable`.
    pub fn line(&self, idx: usize) -> Option<String> {
        self.piece_table.get_line_contents(idx)
    }

    /// Length of the given line in **characters** (not bytes). 0 if the
    /// line doesn't exist.
    pub fn line_char_len(&self, idx: usize) -> usize {
        self.line(idx).map(|s| s.chars().count()).unwrap_or(0)
    }

    /// Length of the given line in **bytes**. 0 if the line doesn't exist.
    pub fn line_byte_len(&self, idx: usize) -> usize {
        self.line(idx).map(|s| s.len()).unwrap_or(0)
    }

    /// Total byte length of the buffer.
    pub fn byte_len(&self) -> usize {
        self.piece_table.len()
    }

    /// Total character count.
    pub fn char_count(&self) -> usize {
        self.piece_table.char_count()
    }

    /// True if the buffer has no content.
    pub fn is_empty(&self) -> bool {
        self.piece_table.is_empty()
    }

    /// The entire buffer contents as a `String`.
    ///
    /// Allocates the full document. Use for saving, whole-buffer copy, and
    /// tests — not in hot paths.
    pub fn to_string(&self) -> String {
        self.piece_table.to_string()
    }

    /// True if the byte at `offset` is on a UTF-8 char boundary.
    pub fn is_char_boundary(&self, offset: usize) -> bool {
        self.piece_table.is_char_boundary(offset)
    }

    // ------------------------------------------------------------------
    // Mutation
    // ------------------------------------------------------------------

    /// Insert `text` at the given **byte** offset.
    ///
    /// Returns an error if the offset is out of bounds or not on a char
    /// boundary. Marks the buffer dirty on success.
    pub fn insert(&mut self, offset: usize, text: &str) -> DomainResult<()> {
        self.piece_table.insert(offset, text)?;
        if !text.is_empty() {
            self.mark_dirty();
        }
        Ok(())
    }

    /// Delete the given **byte** range.
    ///
    /// Returns an error if the range is out of bounds or not on char
    /// boundaries. Marks the buffer dirty on success.
    pub fn delete(&mut self, range: Range<usize>) -> DomainResult<()> {
        if range.start == range.end {
            return Ok(());
        }
        self.piece_table.delete(range)?;
        self.mark_dirty();
        Ok(())
    }

    /// Replace the entire buffer contents. Marks the buffer dirty.
    pub fn replace_all(&mut self, text: &str) {
        self.piece_table.replace_all(text);
        self.mark_dirty();
    }

    // ------------------------------------------------------------------
    // Origin
    // ------------------------------------------------------------------

    /// Associate this buffer with a file path.
    ///
    /// Does not write to disk. If the buffer was `Clean` and is now
    /// associated with a path, it stays `Clean`. If it was `NeverSaved`
    /// with content, it becomes `Dirty` — the caller must save it to make
    /// the association real on disk.
    pub fn set_path(&mut self, path: PathBuf) {
        let was_scratch = self.is_scratch();
        self.origin = Origin::File { path };

        if was_scratch {
            match self.save_state {
                SaveState::NeverSaved if self.piece_table.is_empty() => {
                    // Empty scratch buffer associated with a path. It's
                    // as if we just created an empty file: nothing to save.
                    self.save_state = SaveState::Clean;
                }
                SaveState::NeverSaved => {
                    // Has content but was never saved. Needs a save.
                    self.save_state = SaveState::Dirty;
                }
                other => {
                    self.save_state = other;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Persistence
    // ------------------------------------------------------------------

    /// Write the buffer to its associated path.
    ///
    /// Errors if the buffer has no path. On success, marks the buffer
    /// clean.
    pub fn save(&mut self) -> DomainResult<()> {
        let Some(path) = self.path().map(|p| p.to_path_buf()) else {
            return Err(DomainError::NoPath);
        };
        self.save_to_path(&path)
    }

    /// Write the buffer to the given path and update its origin.
    ///
    /// On success:
    /// - The file on disk is overwritten.
    /// - The buffer's origin is set to `path`.
    /// - The save state becomes `Clean`.
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

    /// Rename the underlying file on disk and update the buffer's path.
    ///
    /// Errors if the buffer has no path, or if the filesystem rename fails.
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

    // ------------------------------------------------------------------
    // Internal
    // ------------------------------------------------------------------

    /// Transition to `Dirty`. Idempotent.
    fn mark_dirty(&mut self) {
        self.save_state = SaveState::Dirty;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Constructors --------------------------------------------------

    #[test]
    fn scratch_is_empty_and_unnamed() {
        let buf = Buffer::scratch("hello");
        assert!(buf.is_empty());
        assert_eq!(buf.display_name(), "hello");
        assert!(buf.is_scratch());
        assert_eq!(buf.path(), None);
        assert_eq!(buf.save_state(), SaveState::NeverSaved);
        assert!(!buf.is_dirty());
    }

    #[test]
    fn with_contents_is_clean() {
        let buf = Buffer::with_contents("/tmp/test.rs", "hello\nworld");
        assert_eq!(buf.display_name(), "test.rs");
        assert_eq!(buf.path(), Some(Path::new("/tmp/test.rs")));
        assert_eq!(buf.save_state(), SaveState::Clean);
        assert!(!buf.is_dirty());
        assert!(!buf.is_scratch());
    }

    #[test]
    fn default_is_unnamed_scratch() {
        let buf = Buffer::default();
        assert!(buf.is_scratch());
        assert_eq!(buf.display_name(), "Unnamed");
    }

    // --- Queries -------------------------------------------------------

    #[test]
    fn line_count_matches_piece_table() {
        let buf = Buffer::with_contents("/tmp/a", "one\ntwo\nthree");
        assert_eq!(buf.line_count(), 3);
    }

    #[test]
    fn line_returns_contents() {
        let buf = Buffer::with_contents("/tmp/a", "one\ntwo\nthree");
        assert_eq!(buf.line(0), Some("one".to_string()));
        assert_eq!(buf.line(1), Some("two".to_string()));
        assert_eq!(buf.line(2), Some("three".to_string()));
        assert_eq!(buf.line(3), None);
    }

    #[test]
    fn line_char_len_counts_chars_not_bytes() {
        let buf = Buffer::with_contents("/tmp/a", "héllo\nx");
        // "héllo" is 5 chars, 6 bytes.
        assert_eq!(buf.line_char_len(0), 5);
        assert_eq!(buf.line_byte_len(0), 6);
    }

    #[test]
    fn byte_and_char_count() {
        let buf = Buffer::with_contents("/tmp/a", "héllo");
        assert_eq!(buf.byte_len(), 6);
        assert_eq!(buf.char_count(), 5);
    }

    // --- Mutation ------------------------------------------------------

    #[test]
    fn insert_marks_dirty() {
        let mut buf = Buffer::with_contents("/tmp/a", "hello");
        assert!(!buf.is_dirty());
        buf.insert(5, " world").unwrap();
        assert!(buf.is_dirty());
        assert_eq!(buf.to_string(), "hello world");
    }

    #[test]
    fn insert_empty_text_does_not_mark_dirty() {
        let mut buf = Buffer::with_contents("/tmp/a", "hello");
        buf.insert(3, "").unwrap();
        assert!(!buf.is_dirty());
    }

    #[test]
    fn delete_marks_dirty() {
        let mut buf = Buffer::with_contents("/tmp/a", "hello world");
        buf.delete(5..6).unwrap();
        assert!(buf.is_dirty());
        assert_eq!(buf.to_string(), "helloworld");
    }

    #[test]
    fn delete_empty_range_does_not_mark_dirty() {
        let mut buf = Buffer::with_contents("/tmp/a", "hello");
        buf.delete(2..2).unwrap();
        assert!(!buf.is_dirty());
    }

    #[test]
    fn replace_all_marks_dirty() {
        let mut buf = Buffer::with_contents("/tmp/a", "hello");
        buf.replace_all("goodbye");
        assert!(buf.is_dirty());
        assert_eq!(buf.to_string(), "goodbye");
    }

    #[test]
    fn insert_at_non_boundary_errors() {
        let mut buf = Buffer::with_contents("/tmp/a", "é");
        assert!(buf.insert(1, "x").is_err());
        // Buffer must not have been modified on error.
        assert!(!buf.is_dirty());
        assert_eq!(buf.to_string(), "é");
    }

    // --- Save state ----------------------------------------------------

    #[test]
    fn mark_dirty_is_idempotent() {
        let mut buf = Buffer::with_contents("/tmp/a", "hello");
        buf.insert(0, "x").unwrap();
        buf.insert(0, "y").unwrap();
        assert!(buf.is_dirty());
        assert_eq!(buf.save_state(), SaveState::Dirty);
    }

    #[test]
    fn set_path_on_empty_scratch_becomes_clean() {
        let mut buf = Buffer::scratch("x");
        assert_eq!(buf.save_state(), SaveState::NeverSaved);
        buf.set_path(PathBuf::from("/tmp/new.rs"));
        // Empty scratch buffer associated with a path = empty file.
        // Nothing to save; buffer is clean.
        assert_eq!(buf.save_state(), SaveState::Clean);
        assert!(!buf.is_scratch());
    }

    #[test]
    fn set_path_on_nonempty_scratch_becomes_dirty() {
        let mut buf = Buffer::scratch("x");
        buf.insert(0, "hello").unwrap();
        assert!(buf.is_dirty());

        // Simulate "Save As": the caller sets a path, then saves.
        buf.set_path(PathBuf::from("/tmp/new.rs"));
        assert_eq!(buf.save_state(), SaveState::Dirty); // still dirty
    }

    // --- Persistence ---------------------------------------------------

    #[test]
    fn save_without_path_errors() {
        let mut buf = Buffer::scratch("x");
        buf.insert(0, "hello").unwrap();
        let err = buf.save().unwrap_err();
        assert!(matches!(err, DomainError::NoPath));
    }

    #[test]
    fn save_to_path_writes_and_cleans() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.rs");

        let mut buf = Buffer::scratch("test");
        buf.insert(0, "hello world").unwrap();
        assert!(buf.is_dirty());

        buf.save_to_path(&path).unwrap();
        assert_eq!(buf.save_state(), SaveState::Clean);
        assert!(!buf.is_dirty());
        assert_eq!(buf.path(), Some(path.as_path()));

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, "hello world");
    }

    #[test]
    fn open_after_save_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("roundtrip.txt");

        let mut buf = Buffer::scratch("roundtrip");
        buf.insert(0, "line one\nline two\n").unwrap();
        buf.save_to_path(&path).unwrap();

        let reopened = Buffer::open_file(&path).unwrap();
        assert_eq!(reopened.to_string(), "line one\nline two\n");
        assert_eq!(reopened.line_count(), 3); // trailing empty line
        assert_eq!(reopened.display_name(), "roundtrip.txt");
        assert!(!reopened.is_dirty());
    }

    #[test]
    fn open_nonexistent_file_errors() {
        let err = Buffer::open_file("/this/does/not/exist.txt").unwrap_err();
        assert!(matches!(err, DomainError::CannotReadFile { .. }));
    }

    #[test]
    fn rename_updates_origin() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("old.txt");
        let new = dir.path().join("new.txt");

        let mut buf = Buffer::scratch("old");
        buf.insert(0, "hi").unwrap();
        buf.save_to_path(&old).unwrap();

        buf.rename_to(new.clone()).unwrap();
        assert_eq!(buf.path(), Some(new.as_path()));
        assert_eq!(buf.display_name(), "new.txt");
        assert!(!old.exists());
        assert!(new.exists());
    }
}
