//! Persistent state for the editor panel.
//!
//! Owns the cursor position, viewport scroll offsets, and a cached line
//! count (used to clamp scroll). All of these survive across frames and
//! are read/written by the widget during render.

/// Cursor position and viewport state for the editor panel.
///
/// All coordinates are **0-indexed** and measured in **characters**, not
/// bytes. Multi-byte input works correctly because every offset counts
/// `char`s, not `u8`s.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EditorState {
    /// Cursor line (row), 0-indexed from the top of the buffer.
    pub cursor_line: usize,

    /// Cursor column, 0-indexed from the start of the line.
    pub cursor_col: usize,

    /// Vertical scroll: index of the topmost visible line.
    pub scroll_y: usize,

    /// Horizontal scroll: index of the leftmost visible character.
    pub scroll_x: usize,

    /// Cached line count of the active buffer. Used to clamp scroll and
    /// cursor movement without needing to read the buffer during render.
    pub line_count: usize,

    /// Whether to draw a cursor. Set to `false` in read-only previews.
    pub show_cursor: bool,

    /// Whether to highlight the entire cursor line.
    pub highlight_current_line: bool,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            cursor_line: 0,
            cursor_col: 0,
            scroll_y: 0,
            scroll_x: 0,
            line_count: 0,
            show_cursor: true,
            highlight_current_line: true,
        }
    }

    // ------------------------------------------------------------------
    // Cursor movement
    // ------------------------------------------------------------------

    /// Move the cursor to `(line, col)`, clamping to valid bounds.
    ///
    /// `col` is clamped against the length of the target line, which the
    /// caller passes because the state doesn't know the buffer contents.
    pub fn set_cursor(&mut self, line: usize, col: usize, line_len: usize) {
        self.cursor_line = line.min(self.line_count.saturating_sub(1));
        self.cursor_col = col.min(line_len);
    }

    pub fn move_left(&mut self) {
        self.cursor_col = self.cursor_col.saturating_sub(1);
    }

    pub fn move_right(&mut self, line_len: usize) {
        self.cursor_col = (self.cursor_col + 1).min(line_len);
    }

    pub fn move_up(&mut self, line_len: usize) {
        self.cursor_line = self.cursor_line.saturating_sub(1);
        // Clamp column to the new line's length. Without this, moving up
        // from a long line to a short one leaves the cursor past the end.
        self.cursor_col = self.cursor_col.min(line_len);
    }

    pub fn move_down(&mut self, line_len: usize) {
        if self.cursor_line + 1 < self.line_count {
            self.cursor_line += 1;
        }

        self.cursor_col = self.cursor_col.min(line_len);
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self, line_len: usize) {
        self.cursor_col = line_len;
    }

    pub fn move_to_file_start(&mut self) {
        self.cursor_line = 0;
        self.cursor_col = 0;
    }

    pub fn move_to_file_end(&mut self, last_line_len: usize) {
        self.cursor_line = self.line_count.saturating_sub(1);
        self.cursor_col = last_line_len;
    }

    // ------------------------------------------------------------------
    // Viewport correction
    // ------------------------------------------------------------------

    /// Called from the widget's `render`. Adjusts scroll so the cursor is
    /// within the viewport of the given size. Clamps scroll against
    /// `line_count`.
    ///
    /// This is the one mutation a widget makes: only the widget knows the
    /// viewport dimensions at render time.
    pub fn ensure_cursor_visible(&mut self, viewport_height: usize, viewport_width: usize) {
        if viewport_height == 0 || viewport_width == 0 {
            return;
        }

        // Vertical.
        if self.cursor_line < self.scroll_y {
            self.scroll_y = self.cursor_line;
        } else if self.cursor_line >= self.scroll_y + viewport_height {
            self.scroll_y = self.cursor_line + 1 - viewport_height;
        }
        // Clamp to max scroll.
        let max_scroll_y = self.line_count.saturating_sub(viewport_height);
        if self.scroll_y > max_scroll_y {
            self.scroll_y = max_scroll_y;
        }

        // Horizontal.
        if self.cursor_col < self.scroll_x {
            self.scroll_x = self.cursor_col;
        } else if self.cursor_col >= self.scroll_x + viewport_width {
            self.scroll_x = self.cursor_col + 1 - viewport_width;
        }
    }

    /// Update the cached line count and clamp scroll. Called by the app
    /// whenever the active buffer changes or its line count changes.
    pub fn set_line_count(&mut self, n: usize) {
        self.line_count = n;
        if self.cursor_line >= n {
            self.cursor_line = n.saturating_sub(1);
        }
    }

    /// Reset everything to the start of the file.
    pub fn reset(&mut self) {
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_y = 0;
        self.scroll_x = 0;
    }
}
