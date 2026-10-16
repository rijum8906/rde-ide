//! View model for the editor panel.
//!
//! The view is a *snapshot* of the visible window: which lines to draw,
//! where the cursor is, and how the widget should style things. It's built
//! fresh each frame by the app from the active buffer and `EditorState`.

/// A single line to render.
#[derive(Debug, Clone)]
pub struct LineView {
    /// Line number to display in the gutter, 1-indexed.
    pub number: usize,
    /// The text of the line, without a trailing newline.
    pub text: String,
    /// Whether this line is currently under the cursor.
    pub is_current: bool,
    /// Optional selection range within this line (start_col..end_col),
    /// measured in characters. `None` if no selection intersects.
    pub selection: Option<(usize, usize)>,
}

/// Everything the editor panel needs to render.
#[derive(Debug, Clone)]
pub struct EditorPanelView {
    /// Visible lines, in display order (top to bottom).
    pub lines: Vec<LineView>,
    /// Cursor position within the visible viewport, in (row, col) cells.
    /// `None` if the cursor is off-screen (which shouldn't happen after
    /// `ensure_cursor_visible`, but defensive).
    pub cursor_screen: Option<(u16, u16)>,
    /// Gutter width in columns, including padding.
    pub gutter_width: u16,
    /// Whether this panel has focus (drawn with a highlighted border or
    /// brighter text).
    pub is_focused: bool,
    /// Whether the current line should be highlighted.
    pub highlight_current_line: bool,
    /// Optional placeholder text shown when the buffer is empty.
    pub empty_message: Option<String>,
}

impl EditorPanelView {
    /// Construct an empty view for when there's no active buffer.
    pub fn empty() -> Self {
        Self {
            lines: Vec::new(),
            cursor_screen: None,
            gutter_width: 0,
            is_focused: true,
            highlight_current_line: false,
            empty_message: Some("(no buffer)".to_string()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
