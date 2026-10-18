//! Top bar widget: renders a strip of tabs with the active one highlighted.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::StatefulWidget,
};

use crate::state::top_bar::TopBarState;
use crate::viewmodel::top_bar::TopBarView;

const DIRTY_MARKER: char = '●';
const SEPARATOR: char = '│';

/// Tab strip rendered by the top bar.
#[derive(Debug)]
pub struct TopBarWidget<'a> {
    view: &'a TopBarView<'a>,
}

impl<'a> TopBarWidget<'a> {
    /// Construct the widget from its view model.
    pub fn new(view: &'a TopBarView<'a>) -> Self {
        Self { view }
    }
}

impl StatefulWidget for TopBarWidget<'_> {
    type State = TopBarState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut TopBarState) {
        fill_background(area, buf);

        if area.width == 0 || area.height == 0 || self.view.tabs.is_empty() {
            return;
        }

        let y = area.y;
        let max_x = area.x.saturating_add(area.width);
        let mut x = area.x;

        // Simple scan: walk the tabs from the left until the bar is full.
        // (No cache for now; tab widths are cheap.)
        for (idx, tab) in self.view.tabs.iter().enumerate() {
            if x >= max_x {
                break;
            }

            // Separator before every tab but the first.
            if idx > 0 {
                x = write_char(buf, x, y, max_x, SEPARATOR, separator_style());
            }

            let style = if tab.is_active {
                active_tab_style()
            } else {
                inactive_tab_style()
            };

            // Left padding.
            x = write_char(buf, x, y, max_x, ' ', style);

            // Title.
            x = write_str(buf, x, y, max_x, tab.title, style);

            // Right padding.
            x = write_char(buf, x, y, max_x, ' ', style);

            // Dirty marker.
            if tab.is_dirty {
                x = write_char(buf, x, y, max_x, DIRTY_MARKER, dirty_style());
                x = write_char(buf, x, y, max_x, ' ', style);
            }
        }
    }
}

// --- helpers ---

/// Write a single character at `x` on row `y`, clipped to `max_x`.
/// Returns the next free x (saturating).
fn write_char(buf: &mut Buffer, x: u16, y: u16, max_x: u16, ch: char, style: Style) -> u16 {
    if x >= max_x {
        return x;
    }
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_char(ch).set_style(style);
    }
    x.saturating_add(1)
}

/// Write a string starting at `x`, clipped to `max_x`. Returns next free x.
fn write_str(buf: &mut Buffer, mut x: u16, y: u16, max_x: u16, s: &str, style: Style) -> u16 {
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

/// Paint the bar's background style across the whole bar rect.
fn fill_background(area: Rect, buf: &mut Buffer) {
    let max_x = area.x.saturating_add(area.width);
    for x in area.x..max_x {
        if let Some(cell) = buf.cell_mut((x, area.y)) {
            cell.set_style(bar_style());
        }
    }
}

// --- styles ---

/// Background style shared by every part of the bar.
fn bar_style() -> Style {
    Style::default().fg(Color::Reset).bg(Color::Rgb(30, 30, 30))
}

/// Style for the tab the user is currently working in.
fn active_tab_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(220, 220, 220))
        .add_modifier(Modifier::BOLD)
}

/// Style used by all inactive tabs.
fn inactive_tab_style() -> Style {
    Style::default()
        .fg(Color::Rgb(160, 160, 160))
        .bg(Color::Rgb(30, 30, 30))
}

/// Style used for the separator between two adjacent tabs.
fn separator_style() -> Style {
    Style::default()
        .fg(Color::Rgb(80, 80, 80))
        .bg(Color::Rgb(30, 30, 30))
}

/// Style used to mark tabs with unsaved changes.
fn dirty_style() -> Style {
    Style::default()
        .fg(Color::Rgb(230, 180, 80))
        .bg(Color::Rgb(30, 30, 30))
        .add_modifier(Modifier::BOLD)
}
