//! Bottom bar: status line, command line, and transient messages.

use ratatui::{
    buffer::Buffer as RatBuffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::StatefulWidget,
};

use crate::state::bottom_bar::BottomBarState;
use crate::viewmodel::bottom_bar::{BottomBarMode, BottomBarView};

/// Bottom bar widget. Renders either the status line or the command-line
/// editor, depending on `view.mode`.
#[derive(Debug)]
pub struct BottomBarWidget<'a> {
    view: &'a BottomBarView,
}

impl<'a> BottomBarWidget<'a> {
    pub fn new(view: &'a BottomBarView) -> Self {
        Self { view }
    }
}

impl StatefulWidget for BottomBarWidget<'_> {
    type State = BottomBarState;

    fn render(self, area: Rect, buf: &mut RatBuffer, state: &mut BottomBarState) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        match self.view.mode {
            BottomBarMode::Status => render_status(self.view, area, buf),
            BottomBarMode::Command => render_command(self.view, area, buf, state),
        }
    }
}

// --- Status mode ----------------------------------------------------------

fn render_status(view: &BottomBarView, area: Rect, buf: &mut RatBuffer) {
    let y = area.y;
    let max_x = area.x.saturating_add(area.width);
    let mut x = area.x;

    // Left block: mode badge + filename + dirty marker.
    let badge_style = match view.mode_label {
        "INSERT" => Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD),
        "VISUAL" => Style::default()
            .fg(Color::Black)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .fg(Color::Black)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    };

    x = write_str(buf, x, y, max_x, " ", badge_style);
    x = write_str(buf, x, y, max_x, view.mode_label, badge_style);
    x = write_str(buf, x, y, max_x, " ", badge_style);

    x = write_str(buf, x, y, max_x, " ", base_style());
    x = write_str(
        buf,
        x,
        y,
        max_x,
        &view.file_name,
        filename_style(view.is_dirty),
    );
    if view.is_dirty {
        x = write_str(buf, x, y, max_x, " ●", dirty_style());
    }
    x = write_str(buf, x, y, max_x, " ", base_style());

    // Right block: cursor position + line count.
    let position_text = format!(
        " {}:{} / {} ",
        view.cursor.0, view.cursor.1, view.line_count
    );
    let position_width = position_text.chars().count() as u16;

    // Draw position right-aligned if it fits.
    if position_width <= area.width {
        let pos_x = max_x.saturating_sub(position_width);
        if pos_x > x {
            write_str(buf, pos_x, y, max_x, &position_text, position_style());
        } else {
            // Fall back to inline if the bar is too narrow.
            let _ = write_str(buf, x, y, max_x, &position_text, position_style());
        }
    }

    // Fill any remaining space with base background.
    fill_background(area, buf);
}

// --- Command-line mode ----------------------------------------------------

fn render_command(_view: &BottomBarView, area: Rect, buf: &mut RatBuffer, state: &BottomBarState) {
    let y = area.y;
    let max_x = area.x.saturating_add(area.width);
    let mut x = area.x;

    // Draw ":" prefix.
    x = write_str(buf, x, y, max_x, ":", command_prefix_style());

    // Draw the command-line text.
    let text = &state.command_line;
    let _ = write_str(buf, x, y, max_x, text, command_text_style());

    // Draw the cursor as a highlighted cell at the current cursor position.
    // Compute the screen x of the cursor from the char-count prefix.
    let cursor_x = area
        .x
        .saturating_add(1) // after ":"
        .saturating_add(char_count_to_u16(text, state.command_cursor));

    if cursor_x < max_x {
        // If the cursor is past the end of the text, draw a block cursor
        // on a space. Otherwise, invert the character at the cursor.
        let cursor_char = text.chars().nth(state.command_cursor).unwrap_or(' ');
        if let Some(cell) = buf.cell_mut((cursor_x, y)) {
            cell.set_char(cursor_char).set_style(command_cursor_style());
        }
    }

    // Fill the rest with the base command background.
    let fill_from = cursor_x.saturating_add(1);
    fill_background_from(area, buf, fill_from);
}

/// Count characters in `s` up to `max` characters, returning a u16.
/// Saturates at u16::MAX.
fn char_count_to_u16(s: &str, max: usize) -> u16 {
    s.chars().take(max).count().min(u16::MAX as usize) as u16
}

// --- Helpers --------------------------------------------------------------

fn write_str(buf: &mut RatBuffer, mut x: u16, y: u16, max_x: u16, s: &str, style: Style) -> u16 {
    for ch in s.chars() {
        if x >= max_x {
            break;
        }
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(ch).set_style(style);
        }
        x = x.saturating_add(1);
    }
    x
}

fn fill_background(area: Rect, buf: &mut RatBuffer) {
    let max_x = area.x.saturating_add(area.width);
    for cx in area.x..max_x {
        if let Some(cell) = buf.cell_mut((cx, area.y)) {
            cell.set_style(base_style());
        }
    }
}

fn fill_background_from(area: Rect, buf: &mut RatBuffer, from_x: u16) {
    let max_x = area.x.saturating_add(area.width);
    let start = from_x.max(area.x);
    for cx in start..max_x {
        if let Some(cell) = buf.cell_mut((cx, area.y)) {
            cell.set_style(base_style());
        }
    }
}

// --- Styles ---------------------------------------------------------------

fn base_style() -> Style {
    Style::default()
        .fg(Color::Rgb(200, 200, 200))
        .bg(Color::Rgb(30, 30, 30))
}

fn filename_style(is_dirty: bool) -> Style {
    let fg = if is_dirty {
        Color::Rgb(230, 180, 80)
    } else {
        Color::Rgb(200, 200, 200)
    };
    Style::default()
        .fg(fg)
        .bg(Color::Rgb(30, 30, 30))
        .add_modifier(Modifier::BOLD)
}

fn dirty_style() -> Style {
    Style::default()
        .fg(Color::Rgb(230, 180, 80))
        .bg(Color::Rgb(30, 30, 30))
        .add_modifier(Modifier::BOLD)
}

fn position_style() -> Style {
    Style::default()
        .fg(Color::Rgb(140, 140, 140))
        .bg(Color::Rgb(30, 30, 30))
}

fn command_prefix_style() -> Style {
    Style::default()
        .fg(Color::Rgb(200, 200, 200))
        .bg(Color::Rgb(30, 30, 30))
        .add_modifier(Modifier::BOLD)
}

fn command_text_style() -> Style {
    Style::default()
        .fg(Color::Rgb(220, 220, 220))
        .bg(Color::Rgb(30, 30, 30))
}

fn command_cursor_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(220, 220, 220))
}
