//! Workspace state: the set of open buffers and the active selection.
//!
//! This is the top-level container of the domain. It owns `Buffer`s and
//! tracks which one is focused. It is UI-agnostic: no cursor, no scroll,
//! no layout.

use std::path::Path;

use crate::{
    buffer::Buffer,
    error::{IdeError, Result},
};

/// The set of open documents plus the active selection.
#[derive(Debug)]
pub struct WorkspaceState {
    /// Open buffers, in the order they appear in the tab bar.
    buffers: Vec<Buffer>,

    /// Index of the buffer the user is currently working in.
    ///
    /// The source of truth for which buffer receives input; always
    /// checked against the length of `buffers` before use.
    active_buffer_idx: usize,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        // create a default buffer
        let buffer = Buffer::default();

        Self {
            buffers: vec![buffer],
            active_buffer_idx: 0,
        }
    }
}

impl WorkspaceState {
    // ===========================================================
    // Core Lifestyle
    // ===========================================================

    /// Creates a new workspace state from the path given on the command line.
    ///
    /// # UseCase:
    ///
    /// ```text
    /// ide some_file.rs
    /// ide some_path
    /// ide .
    /// ide ./
    /// ```
    ///
    /// Opening a file loads it as a buffer; opening a directory starts
    /// with no buffers.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        if !path_buf.exists() {
            return Err(IdeError::PathNotFound(path_buf));
        }
        // Case 1: if it's a file
        if path_buf.is_file() {
            let buffer = Buffer::open_file(path)?;
            Ok(Self {
                buffers: vec![buffer],
                active_buffer_idx: 0,
            })
        }
        // Case 2: if it's a dir
        else if path_buf.is_dir() {
            // TODO: need to display all the files and folders in the left panel
            Ok(Self {
                buffers: vec![],
                active_buffer_idx: 0,
            })
        } else {
            Err(IdeError::CannotReadFile(path_buf))
        }
    }

    // ===========================================================
    // Mutation
    // ===========================================================

    /// Check that a buffer can be closed at `index`, then close it,
    /// adjusting the active buffer index accordingly.
    ///
    /// Returns an error when `index` does not name an open buffer.
    pub fn remove_buffer(&mut self, idx: usize) -> Result<()> {
        if idx >= self.buffers.len() {
            return Err(IdeError::AppError(
                "Couldn't find the buffer to close".to_string(),
            ));
        }

        self.buffers.remove(idx);

        // Fix the active index after the removal:
        // - If the removed buffer came before the active one, shift it left.
        // - If the removed buffer was the active one, clamp to the last buffer.
        // - Otherwise the active index is unchanged.
        if self.buffers.is_empty() {
            self.active_buffer_idx = 0;
        } else if idx < self.active_buffer_idx {
            self.active_buffer_idx -= 1;
        } else if idx == self.active_buffer_idx {
            self.active_buffer_idx = self.active_buffer_idx.min(self.buffers.len() - 1);
        }

        Ok(())
    }

    /// Close the currently active buffer.
    pub fn close_active(&mut self) -> Result<()> {
        self.remove_buffer(self.active_buffer_idx)
    }

    // ===========================================================
    // Getter and Setter
    // ===========================================================

    /// The buffers that are currently open.
    pub fn buffers(&self) -> &[Buffer] {
        &self.buffers
    }

    /// The index of the buffer the user is currently working in.
    pub fn active_buffer_idx(&self) -> usize {
        self.active_buffer_idx
    }
}
