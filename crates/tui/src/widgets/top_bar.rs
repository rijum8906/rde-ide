//! Neovim-style tab bar rendered from the workspace's buffer list.

use domain::buffer::Buffer;
use ratatui::{
    buffer::Buffer as RatBuffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

/// Character shown next to a tab with unsaved changes (U+25CF).
const DIRTY_MARKER: char = '●';

/// Vertical separator drawn between adjacent tabs.
const TAB_SEPARATOR: char = '│';

/// Padding inserted on each side of a tab's title.
const TAB_PADDING: char = ' ';

/// Neovim-style tab bar.
///
/// Borrows the workspace's buffers for the duration of a single render call.
/// Constructed fresh each frame — cheap (one slice reference + one index).
#[derive(Debug)]
pub struct TopBar<'a> {
    buffers: &'a [Buffer],
    active_index: usize,
}

impl<'a> TopBar<'a> {
    /// Construct a top bar from the workspace's open buffers.
    ///
    /// `active_index` must be a valid index into `buffers` for the active
    /// tab to be highlighted; out-of-range values simply highlight nothing.
    pub fn new(buffers: &'a [Buffer], active_index: usize) -> Self {
        Self {
            buffers,
            active_index,
        }
    }

    pub fn buffers(&self) -> &[Buffer] {
        self.buffers
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }
}

impl Widget for TopBar<'_> {
    fn render(self, area: Rect, buf: &mut RatBuffer) {
        if area.width == 0 || area.height == 0 || self.buffers.is_empty() {
            // Still fill the background even with no tabs, so the bar owns
            // its rect visually.
            fill_background(area, buf);
            return;
        }

        let y = area.y;
        let max_x = area.x.saturating_add(area.width);
        let mut x = area.x;

        for (idx, buffer) in self.buffers.iter().enumerate() {
            if x >= max_x {
                break;
            }

            // Separator before every tab except the first.
            if idx > 0 {
                x = write_char(buf, x, y, max_x, TAB_SEPARATOR, separator_style());
            }

            let is_active = idx == self.active_index;
            let tab_style = if is_active {
                active_tab_style()
            } else {
                inactive_tab_style()
            };

            // Leading padding.
            x = write_char(buf, x, y, max_x, TAB_PADDING, tab_style);

            // Title (clipped to remaining width).
            x = write_str(buf, x, y, max_x, buffer.display_name(), tab_style);

            // Trailing padding.
            x = write_char(buf, x, y, max_x, TAB_PADDING, tab_style);

            // Dirty marker.
            if buffer.is_dirty() {
                x = write_char(buf, x, y, max_x, DIRTY_MARKER, dirty_style());
                x = write_char(buf, x, y, max_x, TAB_PADDING, tab_style);
            }
        }

        // Fill the trailing region so the top bar paints its full rect.
        fill_background_from(area, buf, x);
    }
}

// --- Rendering helpers ----------------------------------------------------

/// Write a single character at `x` on row `y`, clipped to `max_x`.
/// Returns the next free x (saturating).
fn write_char(buf: &mut RatBuffer, x: u16, y: u16, max_x: u16, ch: char, style: Style) -> u16 {
    if x >= max_x {
        return x;
    }
    buf.get_mut(x, y).set_char(ch).set_style(style);
    x.saturating_add(1)
}

/// Write a string starting at `x`, clipped to `max_x`. Returns next free x.
fn write_str(buf: &mut RatBuffer, mut x: u16, y: u16, max_x: u16, s: &str, style: Style) -> u16 {
    for ch in s.chars() {
        if x >= max_x {
            break;
        }
        buf.get_mut(x, y).set_char(ch).set_style(style);
        x = x.saturating_add(1);
    }
    x
}

/// Paint the background style across the whole bar rect.
fn fill_background(area: Rect, buf: &mut RatBuffer) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let max_x = area.x.saturating_add(area.width);
    for cx in area.x..max_x {
        buf.get_mut(cx, area.y).set_style(bar_style());
    }
}

/// Paint the background from `from_x` to the end of the bar.
fn fill_background_from(area: Rect, buf: &mut RatBuffer, from_x: u16) {
    let max_x = area.x.saturating_add(area.width);
    let start = from_x.max(area.x);
    for cx in start..max_x {
        buf.get_mut(cx, area.y).set_style(bar_style());
    }
}

// --- Styles ---------------------------------------------------------------
//
// Kept as small functions for now so the widget is self-contained. Once you
// add a `theme` module, replace these with `Theme::default().tab_active` etc.

fn bar_style() -> Style {
    Style::default().fg(Color::Reset).bg(Color::Rgb(30, 30, 30))
}

fn active_tab_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(220, 220, 220))
        .add_modifier(Modifier::BOLD)
}

fn inactive_tab_style() -> Style {
    Style::default()
        .fg(Color::Rgb(160, 160, 160))
        .bg(Color::Rgb(30, 30, 30))
}

fn separator_style() -> Style {
    Style::default()
        .fg(Color::Rgb(80, 80, 80))
        .bg(Color::Rgb(30, 30, 30))
}

fn dirty_style() -> Style {
    Style::default()
        .fg(Color::Rgb(230, 180, 80))
        .bg(Color::Rgb(30, 30, 30))
        .add_modifier(Modifier::BOLD)
}
