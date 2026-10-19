//! View-model data passed to the bottom bar each frame.
//!
//! All of this is derived from the app state; none of it is persistent.
//! Transient messages are shown by the toast widget instead.

/// Which layout the bottom bar is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomBarMode {
    /// Normal status line: mode badge, filename, cursor position.
    Status,
    /// Command-line editing: `:wq_` with a visible cursor.
    Command,
}

/// Everything the bottom bar needs to render, except its own state.
#[derive(Debug, Clone)]
pub struct BottomBarView {
    pub mode: BottomBarMode,
    /// Mode badge text, e.g. "NORMAL", "INSERT".
    pub mode_label: &'static str,
    /// Display name of the active buffer, or "[No Name]".
    pub file_name: String,
    /// Whether the buffer has unsaved changes.
    pub is_dirty: bool,
    /// 1-indexed cursor position for display: (line, col).
    pub cursor: (usize, usize),
    /// Total line count for display: "line / total".
    pub line_count: usize,
    /// Left-side hint text (e.g. "-- NORMAL --" or a key hint).
    pub hint: Option<String>,
}

impl Default for BottomBarView {
    fn default() -> Self {
        Self {
            mode: BottomBarMode::Status,
            mode_label: "NORMAL",
            file_name: "[No Name]".to_string(),
            is_dirty: false,
            cursor: (1, 1),
            line_count: 1,
            hint: None,
        }
    }
}
