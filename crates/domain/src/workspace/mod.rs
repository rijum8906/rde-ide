//! Workspace state: the set of open buffers and the active selection.
//!
//! This is the top-level container of the domain. It owns `Buffer`s and
//! tracks which one is focused. It is UI-agnostic: no cursor, no scroll,
//! no layout.
//!
//! A workspace is always associated with a **root directory**. Opening a
//! file seeds the workspace with that file; opening a directory starts
//! with no buffers.

pub mod file_tree;

use std::env;
use std::path::{Path, PathBuf};

use crate::{
    buffer::Buffer,
    error::{DomainError, DomainResult},
};

/// The set of open documents plus the active selection.
#[derive(Debug)]
pub struct WorkspaceState {
    /// Open buffers, in the order they appear in the tab bar.
    buffers: Vec<Buffer>,

    /// Index of the buffer the user is currently working in.
    ///
    /// Invariant: `buffers.is_empty() || active_buffer_idx < buffers.len()`.
    /// Every mutator maintains this.
    active_buffer_idx: usize,

    /// Root directory of the project / workspace.
    root_dir: PathBuf,
}

// ============================================================================
// Construction
// ============================================================================

impl WorkspaceState {
    /// Create an empty workspace rooted at `root`.
    ///
    /// Does no I/O. The caller decides whether to seed a scratch buffer.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            buffers: Vec::new(),
            active_buffer_idx: 0,
            root_dir: root.into(),
        }
    }

    /// Create an empty workspace rooted at the current working directory.
    ///
    /// If `current_dir` fails, falls back to `.`.
    pub fn with_current_dir() -> Self {
        let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(root)
    }

    /// Create a workspace with a single empty scratch buffer.
    pub fn with_scratch(name: impl Into<String>) -> Self {
        let mut ws = Self::with_current_dir();
        ws.buffers.push(Buffer::scratch(name));
        ws.active_buffer_idx = 0;
        ws
    }

    /// Create a workspace from a command-line path.
    ///
    /// - If `path` is a file, the workspace contains that file as its only
    ///   buffer, and its root is the file's parent directory.
    /// - If `path` is a directory, the workspace is empty and its root is
    ///   that directory.
    /// - If `path` doesn't exist, returns `PathNotFound`.
    /// - If `path` exists but is neither a file nor a directory, returns
    ///   `UnsupportedPath`.
    pub fn from_path(path: impl AsRef<Path>) -> DomainResult<Self> {
        let path_buf = path.as_ref().to_path_buf();

        let meta = std::fs::metadata(&path_buf).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                DomainError::PathNotFound {
                    path: path_buf.clone(),
                }
            } else {
                DomainError::CannotReadFile {
                    path: path_buf.clone(),
                    source,
                }
            }
        })?;

        if !meta.is_file() && !meta.is_dir() {
            return Err(DomainError::UnsupportedPath { path: path_buf });
        }

        let path_buf =
            std::fs::canonicalize(&path_buf).map_err(|source| DomainError::CannotReadFile {
                path: path_buf.clone(),
                source,
            })?;

        if meta.is_file() {
            let root_dir = path_buf
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            let buffer = Buffer::open_file(&path_buf)?;
            Ok(Self {
                buffers: vec![buffer],
                active_buffer_idx: 0,
                root_dir,
            })
        } else {
            Ok(Self {
                buffers: Vec::new(),
                active_buffer_idx: 0,
                root_dir: path_buf,
            })
        }
    }
}

impl Default for WorkspaceState {
    fn default() -> Self {
        // `Default` must not do I/O. Root at `.` and start with no buffers.
        Self::new(PathBuf::from("."))
    }
}

// ============================================================================
// Queries
// ============================================================================

impl WorkspaceState {
    /// All open buffers, in tab order.
    pub fn buffers(&self) -> &[Buffer] {
        &self.buffers
    }

    /// Number of open buffers.
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    /// True if no buffers are open.
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// The active buffer, or `None` if the workspace is empty.
    pub fn active_buffer(&self) -> Option<&Buffer> {
        self.buffers.get(self.active_buffer_idx)
    }

    /// Mutable access to the active buffer, or `None`.
    pub fn active_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(self.active_buffer_idx)
    }

    /// Index of the active buffer.
    ///
    /// Meaningful only if `!is_empty()`; callers should check `is_empty()`
    /// first or use `active_buffer()` instead.
    pub fn active_buffer_idx(&self) -> usize {
        self.active_buffer_idx
    }

    /// Root directory of this workspace.
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// True if any open buffer has unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        self.buffers.iter().any(Buffer::is_dirty)
    }

    /// Number of dirty buffers.
    pub fn dirty_count(&self) -> usize {
        self.buffers.iter().filter(|b| b.is_dirty()).count()
    }

    /// Look up a buffer by its path.
    pub fn buffer_by_path(&self, path: &Path) -> Option<&Buffer> {
        self.buffers.iter().find(|b| b.path() == Some(path))
    }

    /// Index of a buffer by its path, if open.
    pub fn index_of_path(&self, path: &Path) -> Option<usize> {
        self.buffers.iter().position(|b| b.path() == Some(path))
    }
}

// ============================================================================
// Mutation
// ============================================================================

impl WorkspaceState {
    /// Push a buffer and make it active.
    pub fn push_buffer(&mut self, buffer: Buffer) {
        self.buffers.push(buffer);
        self.active_buffer_idx = self.buffers.len() - 1;
    }

    /// Open a file, or focus it if already open.
    ///
    /// - If a buffer with the given path exists, it becomes active.
    /// - Otherwise, reads the file and appends it as the new active buffer.
    pub fn open_file(&mut self, path: impl AsRef<Path>) -> DomainResult<()> {
        let path = path.as_ref();

        if let Some(idx) = self.index_of_path(path) {
            self.active_buffer_idx = idx;
            return Ok(());
        }

        let buffer = Buffer::open_file(path)?;
        self.push_buffer(buffer);
        Ok(())
    }

    /// Close the buffer at `index`. Fails if `index` is out of bounds.
    pub fn remove_buffer(&mut self, index: usize) -> DomainResult<Buffer> {
        if index >= self.buffers.len() {
            return Err(DomainError::NoSuchBuffer {
                index,
                count: self.buffers.len(),
            });
        }

        let removed = self.buffers.remove(index);

        if self.buffers.is_empty() {
            self.active_buffer_idx = 0;
        } else if index < self.active_buffer_idx {
            self.active_buffer_idx -= 1;
        } else if index == self.active_buffer_idx {
            // Active buffer removed: clamp to last valid index. The buffer
            // that shifted into this slot becomes active (or the previous
            // one if we removed the last buffer).
            self.active_buffer_idx = self.active_buffer_idx.min(self.buffers.len() - 1);
        }

        Ok(removed)
    }

    /// Close the currently active buffer.
    pub fn close_active(&mut self) -> DomainResult<Buffer> {
        if self.buffers.is_empty() {
            return Err(DomainError::NoActiveBuffer);
        }
        self.remove_buffer(self.active_buffer_idx)
    }

    /// Set the active buffer by index.
    ///
    /// No-op if `index` is out of bounds. Silent rather than erroring,
    /// because the common case (a keybinding firing after the buffer was
    /// closed) shouldn't crash.
    pub fn set_active(&mut self, index: usize) {
        if index < self.buffers.len() {
            self.active_buffer_idx = index;
        }
    }

    /// Move to the next buffer, wrapping around.
    pub fn next_buffer(&mut self) {
        if !self.buffers.is_empty() {
            self.active_buffer_idx = (self.active_buffer_idx + 1) % self.buffers.len();
        }
    }

    /// Move to the previous buffer, wrapping around.
    pub fn prev_buffer(&mut self) {
        if !self.buffers.is_empty() {
            let n = self.buffers.len();
            self.active_buffer_idx = (self.active_buffer_idx + n - 1) % n;
        }
    }

    /// Update the workspace root directory.
    pub fn set_root_dir(&mut self, root_dir: impl Into<PathBuf>) {
        self.root_dir = root_dir.into();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn ws_with_scratch(name: &str) -> WorkspaceState {
        let mut ws = WorkspaceState::with_current_dir();
        ws.push_buffer(Buffer::scratch(name));
        ws
    }

    #[test]
    fn new_is_empty() {
        let ws = WorkspaceState::new(".");
        assert!(ws.is_empty());
        assert_eq!(ws.buffer_count(), 0);
        assert!(ws.active_buffer().is_none());
    }

    #[test]
    fn push_activates_new_buffer() {
        let mut ws = WorkspaceState::new(".");
        ws.push_buffer(Buffer::scratch("a"));
        ws.push_buffer(Buffer::scratch("b"));
        assert_eq!(ws.buffer_count(), 2);
        assert_eq!(ws.active_buffer_idx(), 1);
        assert_eq!(ws.active_buffer().unwrap().display_name(), "b");
    }

    #[test]
    fn next_buffer_wraps() {
        let mut ws = ws_with_scratch("a");
        ws.push_buffer(Buffer::scratch("b"));
        ws.set_active(0);
        ws.next_buffer();
        assert_eq!(ws.active_buffer_idx(), 1);
        ws.next_buffer();
        assert_eq!(ws.active_buffer_idx(), 0);
    }

    #[test]
    fn prev_buffer_wraps_without_underflow() {
        let mut ws = ws_with_scratch("a");
        ws.push_buffer(Buffer::scratch("b"));
        ws.set_active(0);
        ws.prev_buffer();
        assert_eq!(ws.active_buffer_idx(), 1);
    }

    #[test]
    fn remove_before_active_shifts() {
        let mut ws = WorkspaceState::new(".");
        ws.push_buffer(Buffer::scratch("a"));
        ws.push_buffer(Buffer::scratch("b"));
        ws.push_buffer(Buffer::scratch("c"));
        ws.set_active(2);

        ws.remove_buffer(0).unwrap();

        assert_eq!(ws.active_buffer_idx(), 1);
        assert_eq!(ws.active_buffer().unwrap().display_name(), "c");
    }

    #[test]
    fn remove_active_clamps() {
        let mut ws = WorkspaceState::new(".");
        ws.push_buffer(Buffer::scratch("a"));
        ws.push_buffer(Buffer::scratch("b"));
        ws.set_active(1);

        ws.remove_buffer(1).unwrap();

        assert_eq!(ws.active_buffer_idx(), 0);
        assert_eq!(ws.active_buffer().unwrap().display_name(), "a");
    }

    #[test]
    fn remove_last_buffer_is_valid() {
        let mut ws = ws_with_scratch("a");
        ws.remove_buffer(0).unwrap();
        assert!(ws.is_empty());
        assert!(ws.active_buffer().is_none());
        assert_eq!(ws.active_buffer_idx(), 0);
    }

    #[test]
    fn remove_out_of_bounds_errors() {
        let mut ws = ws_with_scratch("a");
        let err = ws.remove_buffer(5).unwrap_err();
        match err {
            DomainError::NoSuchBuffer { index, count } => {
                assert_eq!(index, 5);
                assert_eq!(count, 1);
            }
            other => panic!("expected NoSuchBuffer, got {other:?}"),
        }
    }

    #[test]
    fn close_active_on_empty_errors() {
        let mut ws = WorkspaceState::new(".");
        let err = ws.close_active().unwrap_err();
        assert!(matches!(err, DomainError::NoActiveBuffer));
    }

    #[test]
    fn set_active_out_of_bounds_is_noop() {
        let mut ws = ws_with_scratch("a");
        ws.set_active(99);
        assert_eq!(ws.active_buffer_idx(), 0);
    }

    #[test]
    fn from_path_nonexistent_errors() {
        let err = WorkspaceState::from_path("/this/does/not/exist").unwrap_err();
        assert!(matches!(err, DomainError::PathNotFound { .. }));
    }

    #[test]
    fn from_path_directory_is_empty_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let ws = WorkspaceState::from_path(dir.path()).unwrap();
        assert!(ws.is_empty());
        assert_eq!(ws.root_dir(), dir.path());
    }

    #[test]
    fn from_path_file_opens_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("hello.txt");
        std::fs::write(&file, "hello\nworld\n").unwrap();

        let ws = WorkspaceState::from_path(&file).unwrap();
        assert_eq!(ws.buffer_count(), 1);
        assert_eq!(ws.root_dir(), dir.path());

        let buf = ws.active_buffer().unwrap();
        assert_eq!(buf.display_name(), "hello.txt");
        assert_eq!(buf.path(), Some(file.as_path()));
    }

    #[test]
    fn from_path_normalizes_file_and_workspace_root() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        let file = nested.join("hello.txt");
        std::fs::write(&file, "hello").unwrap();

        let path_with_parent = nested.join("..").join("nested").join("hello.txt");
        let ws = WorkspaceState::from_path(path_with_parent).unwrap();

        assert_eq!(ws.root_dir(), nested.canonicalize().unwrap());
        assert_eq!(ws.active_buffer().unwrap().path(), Some(file.as_path()));
    }

    #[test]
    fn open_file_twice_focuses_existing() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "content").unwrap();

        let mut ws = WorkspaceState::new(dir.path());
        ws.push_buffer(Buffer::scratch("scratch"));
        ws.open_file(&file).unwrap();
        assert_eq!(ws.buffer_count(), 2);
        assert_eq!(ws.active_buffer_idx(), 1);

        // Move away, then open the same file again.
        ws.set_active(0);
        ws.open_file(&file).unwrap();
        assert_eq!(ws.buffer_count(), 2, "must not duplicate");
        assert_eq!(ws.active_buffer_idx(), 1, "must focus existing buffer");
    }
}
