//! Persistent state for the file-tree side panel.
//!
//! Owns expansion state (which directories are open), selection (which row
//! is highlighted), and scroll offset. Everything displayed is derived
//! from the tree via a view builder.

use std::collections::HashSet;
use std::path::PathBuf;

/// Side-panel state.
#[derive(Debug, Default)]
pub struct LeftPanelState {
    /// Directories whose children are visible, identified by absolute path.
    /// Paths, not indices, so the set survives tree changes.
    pub expanded: HashSet<PathBuf>,

    /// Index into the *flattened* row list of the selected row.
    /// `None` when the panel is empty or nothing is selected.
    pub selected: Option<usize>,

    /// Vertical scroll offset in rows.
    pub scroll_y: usize,
}

impl LeftPanelState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Is the given directory path expanded?
    pub fn is_expanded(&self, path: &PathBuf) -> bool {
        self.expanded.contains(path)
    }

    /// Toggle the expansion state of a directory.
    pub fn toggle_expanded(&mut self, path: PathBuf) {
        if !self.expanded.insert(path.clone()) {
            self.expanded.remove(&path);
        }
    }

    /// Collapse all directories.
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    /// Select a row by index in the flattened list.
    pub fn set_selected(&mut self, index: usize) {
        self.selected = Some(index);
    }

    /// Move selection up by one. Clamps at the top.
    pub fn move_selection_up(&mut self) {
        match self.selected {
            Some(0) | None => self.selected = Some(0),
            Some(i) => self.selected = Some(i - 1),
        }
    }

    /// Move selection down by one, given the number of visible rows.
    pub fn move_selection_down(&mut self, row_count: usize) {
        if row_count == 0 {
            self.selected = None;
            return;
        }
        let current = self.selected.unwrap_or(0);
        self.selected = Some((current + 1).min(row_count - 1));
    }

    /// Move selection to the top.
    pub fn move_selection_home(&mut self) {
        if self.selected.is_some() {
            self.selected = Some(0);
        }
    }

    /// Move selection to the bottom, given the number of visible rows.
    pub fn move_selection_end(&mut self, row_count: usize) {
        if row_count > 0 {
            self.selected = Some(row_count - 1);
        }
    }

    /// Reset selection and scroll. Called when the file tree is (re)loaded.
    pub fn reset(&mut self) {
        self.selected = None;
        self.scroll_y = 0;
        // Note: `expanded` is preserved across resets, so re-scanning the
        // same tree keeps the user's expansion choices.
    }
}
