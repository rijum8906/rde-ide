//! Editor panel widget: renders the visible portion of the active buffer.
//!
//! Reads a `EditorPanelView` (a snapshot of visible lines and cursor
//! position) and writes cells to the buffer. Mutates `EditorState` only
//! to clamp scroll against the viewport size.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::StatefulWidget,
};

use crate::{
    state::editor_panel::EditorState,
    viewmodel::editor_panel::{EditorPanelView, LineView},
};

/// Editor panel widget.
#[derive(Debug)]
pub struct EditorPanel<'a> {
    view: &'a EditorPanelView,
}

impl<'a> EditorPanel<'a> {
    pub fn new(view: &'a EditorPanelView) -> Self {
        Self { view }
    }
}

impl StatefulWidget for EditorPanel<'_> {
    type State = EditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut EditorState) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Viewport-driven scroll correction. Must happen before drawing.
        let text_width = area.width.saturating_sub(self.view.gutter_width) as usize;
        state.ensure_cursor_visible(area.height as usize, text_width);

        // Background.
        fill_background(area, buf, self.view.is_focused);

        // Empty state.
        if self.view.is_empty() {
            self.render_empty(area, buf);
            return;
        }

        // Draw visible lines.
        let gutter = self.view.gutter_width.min(area.width);
        let text_x = area.x + gutter;
        let max_x = area.x + area.width;

        for (screen_row, line) in self.view.lines.iter().enumerate() {
            let y = area.y + screen_row as u16;
            if y >= area.y + area.height {
                break;
            }

            self.render_gutter(line, area, y, gutter, buf);
            self.render_line_text(line, state, text_x, y, max_x, buf);
        }

        // Draw the cursor on top.
        self.render_cursor(area, buf);
    }
}

impl EditorPanel<'_> {
    fn render_gutter(
        &self,
        line: &LineView,
        area: Rect,
        y: u16,
        gutter_width: u16,
        buf: &mut Buffer,
    ) {
        if gutter_width == 0 {
            return;
        }

        let num_str = format!(
            "{:>width$} ",
            line.number,
            width = (gutter_width - 1) as usize
        );
        let style = gutter_style(line.is_current, self.view.is_focused);

        let mut x = area.x;
        let max_gutter_x = area.x + gutter_width;
        for ch in num_str.chars() {
            if x >= max_gutter_x {
                break;
            }
            buf.get_mut(x, y).set_char(ch).set_style(style);
            x += 1;
        }
    }

    fn render_line_text(
        &self,
        line: &LineView,
        state: &EditorState,
        text_x: u16,
        y: u16,
        max_x: u16,
        buf: &mut Buffer,
    ) {
        // If current-line highlighting is on and this is the cursor line,
        // fill the entire text area with the highlight background first.
        if self.view.highlight_current_line && line.is_current {
            let mut x = text_x;
            while x < max_x {
                buf.get_mut(x, y)
                    .set_char(' ')
                    .set_style(current_line_style());
                x += 1;
            }
        }

        let base_style = if line.is_current && self.view.is_focused {
            current_line_text_style()
        } else {
            text_style(self.view.is_focused)
        };

        let mut x = text_x;
        let mut char_col = state.scroll_x;
        let mut chars = line.text.chars().skip(state.scroll_x);

        while x < max_x {
            let Some(ch) = chars.next() else { break };

            // Selection overlay takes precedence over base style.
            let style = if let Some((sel_start, sel_end)) = line.selection {
                if char_col >= sel_start && char_col < sel_end {
                    selection_style()
                } else {
                    base_style
                }
            } else {
                base_style
            };

            buf.get_mut(x, y).set_char(ch).set_style(style);
            x += 1;
            char_col += 1;
        }
    }

    fn render_cursor(&self, area: Rect, buf: &mut Buffer) {
        if !self.view.is_focused {
            return;
        }
        let Some((row, col)) = self.view.cursor_screen else {
            return;
        };

        let cursor_x = area.x + self.view.gutter_width + col;
        let cursor_y = area.y + row;

        if cursor_x >= area.x + area.width || cursor_y >= area.y + area.height {
            return;
        }

        let cell = buf.get_mut(cursor_x, cursor_y);
        // Preserve the character under the cursor, override only the style.
        cell.set_style(cursor_style());
    }

    fn render_empty(&self, area: Rect, buf: &mut Buffer) {
        let msg = self.view.empty_message.as_deref().unwrap_or("(empty)");
        if area.height == 0 || area.width == 0 {
            return;
        }

        let y = area.y + area.height / 2;
        let msg_width = msg.chars().count() as u16;
        let start_x = area.x + area.width.saturating_sub(msg_width) / 2;

        let style = empty_message_style();
        let mut x = start_x;
        let max_x = area.x + area.width;
        for ch in msg.chars() {
            if x >= max_x {
                break;
            }
            buf.get_mut(x, y).set_char(ch).set_style(style);
            x += 1;
        }
    }
}

// --- Backgrounds ---------------------------------------------------------

fn fill_background(area: Rect, buf: &mut Buffer, focused: bool) {
    let style = bg_style(focused);
    let max_x = area.x + area.width;
    for y in area.y..area.y + area.height {
        for x in area.x..max_x {
            buf.get_mut(x, y).set_style(style);
        }
    }
}

// --- Styles --------------------------------------------------------------

const BG_FOCUSED: Color = Color::Rgb(20, 20, 22);
const BG_UNFOCUSED: Color = Color::Rgb(16, 16, 18);

fn bg_style(focused: bool) -> Style {
    let bg = if focused { BG_FOCUSED } else { BG_UNFOCUSED };
    Style::default().fg(Color::Rgb(220, 220, 220)).bg(bg)
}

fn text_style(focused: bool) -> Style {
    let fg = if focused {
        Color::Rgb(220, 220, 220)
    } else {
        Color::Rgb(150, 150, 150)
    };
    let bg = if focused { BG_FOCUSED } else { BG_UNFOCUSED };
    Style::default().fg(fg).bg(bg)
}

fn current_line_style() -> Style {
    Style::default().bg(Color::Rgb(32, 32, 38))
}

fn current_line_text_style() -> Style {
    Style::default()
        .fg(Color::Rgb(240, 240, 240))
        .bg(Color::Rgb(32, 32, 38))
}

fn gutter_style(is_current: bool, focused: bool) -> Style {
    let fg = if !focused {
        Color::Rgb(80, 80, 80)
    } else if is_current {
        Color::Rgb(200, 200, 200)
    } else {
        Color::Rgb(100, 100, 100)
    };
    let bg = if focused { BG_FOCUSED } else { BG_UNFOCUSED };
    Style::default().fg(fg).bg(bg)
}

fn cursor_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(220, 220, 220))
        .add_modifier(Modifier::BOLD)
}

fn selection_style() -> Style {
    Style::default()
        .fg(Color::Rgb(240, 240, 240))
        .bg(Color::Rgb(60, 90, 140))
}

fn empty_message_style() -> Style {
    Style::default()
        .fg(Color::Rgb(100, 100, 100))
        .bg(BG_FOCUSED)
        .add_modifier(Modifier::ITALIC)
}
