//! Side-panel widget: renders a flattened file-tree list.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::StatefulWidget,
};

use crate::{
    state::left_panel::LeftPanelState,
    viewmodel::left_panel::{LeftPanelView, RowView},
};

const INDENT_PER_LEVEL: u16 = 2;
const DIR_EXPANDED: &str = "▾ ";
const DIR_COLLAPSED: &str = "▸ ";
const FILE_PREFIX: &str = "  ";

/// Side-panel widget.
#[derive(Debug)]
pub struct LeftPanelWidget<'a> {
    view: &'a LeftPanelView,
}

impl<'a> LeftPanelWidget<'a> {
    pub fn new(view: &'a LeftPanelView) -> Self {
        Self { view }
    }
}

impl StatefulWidget for LeftPanelWidget<'_> {
    type State = LeftPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut LeftPanelState) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Fill background.
        fill_background(area, buf);

        // Reserve the top row for a title.
        if area.height < 2 {
            // Too short for title + content; just render content.
            self.render_rows(area, buf, state, 0);
            return;
        }

        self.render_title(area, buf);
        let body = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height - 1,
        };
        self.render_rows(body, buf, state, 0);
    }
}

impl LeftPanelWidget<'_> {
    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let y = area.y;
        let max_x = area.x + area.width;

        // Title row: bold, dim background.
        let style = title_style();
        let mut x = area.x;
        for ch in self.view.title.chars() {
            if x >= max_x {
                break;
            }
            buf.cell_mut((x, y)).unwrap().set_char(ch).set_style(style);
            x += 1;
        }
        // Pad the rest of the row.
        while x < max_x {
            buf.cell_mut((x, y)).unwrap().set_char(' ').set_style(style);
            x += 1;
        }
    }

    fn render_rows(&self, area: Rect, buf: &mut Buffer, state: &mut LeftPanelState, _unused: u16) {
        // 1. Loading state.
        if self.view.is_loading {
            self.render_centered_message("Loading…", area, buf);
            return;
        }

        // 2. Empty state.
        if self.view.is_empty() {
            let msg = self.view.empty_message.as_deref().unwrap_or("(empty)");
            self.render_centered_message(msg, area, buf);
            return;
        }

        // 3. Clamp scroll so the selected row is visible.
        //
        // This is the *only* place scroll is adjusted. `execute` might set
        // `selected`, but only the widget knows the viewport height.
        let viewport_height = area.height as usize;
        if let Some(sel) = state.selected {
            if sel < state.scroll_y {
                state.scroll_y = sel;
            } else if sel >= state.scroll_y + viewport_height {
                state.scroll_y = sel + 1 - viewport_height;
            }
        }
        // Also clamp to the maximum scroll position.
        let max_scroll = self.view.rows.len().saturating_sub(viewport_height);
        if state.scroll_y > max_scroll {
            state.scroll_y = max_scroll;
        }

        // 4. Render visible rows.
        let rows = &self.view.rows;
        let max_x = area.x + area.width;

        for screen_row in 0..area.height {
            let row_idx = state.scroll_y + screen_row as usize;
            let Some(row) = rows.get(row_idx) else {
                break;
            };
            let y = area.y + screen_row;

            // Compose the row's text and style.
            let (prefix, prefix_style, name_style) = row_style(row);

            let mut x = area.x;

            // Left padding.
            x = write_char(buf, x, y, max_x, ' ', row_bg_style(row));

            // Indentation.
            for _ in 0..(row.depth as u16 * INDENT_PER_LEVEL) {
                x = write_char(buf, x, y, max_x, ' ', row_bg_style(row));
            }

            // Prefix (expand/collapse arrow for dirs).
            for ch in prefix.chars() {
                x = write_char(buf, x, y, max_x, ch, prefix_style);
            }

            // Name.
            for ch in row.name.chars() {
                if x >= max_x {
                    break;
                }
                buf.cell_mut((x, y))
                    .unwrap()
                    .set_char(ch)
                    .set_style(name_style);
                x += 1;
            }

            // Fill the rest of the row with the row's background.
            while x < max_x {
                buf.cell_mut((x, y))
                    .unwrap()
                    .set_char(' ')
                    .set_style(row_bg_style(row));
                x += 1;
            }
        }
    }

    fn render_centered_message(&self, msg: &str, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let y = area.y + area.height / 2;
        let max_x = area.x + area.width;
        let msg_width = msg.chars().count() as u16;
        let start_x = area.x + area.width.saturating_sub(msg_width) / 2;

        let style = empty_message_style();
        for (x, ch) in (start_x..).zip(msg.chars()) {
            if x >= max_x {
                break;
            }
            buf.cell_mut((x, y)).unwrap().set_char(ch).set_style(style);
        }
    }
}

// --- Row styling ---------------------------------------------------------

fn row_style(row: &RowView) -> (String, Style, Style) {
    let prefix = if row.is_dir {
        if row.is_expanded {
            DIR_EXPANDED.to_string()
        } else {
            DIR_COLLAPSED.to_string()
        }
    } else {
        FILE_PREFIX.to_string()
    };

    let prefix_style = if row.is_selected {
        selected_prefix_style()
    } else if row.is_dir {
        dir_prefix_style()
    } else {
        file_prefix_style()
    };

    let name_style = if row.is_selected {
        selected_name_style()
    } else if row.is_dir {
        dir_name_style()
    } else {
        file_name_style()
    };

    (prefix, prefix_style, name_style)
}

fn row_bg_style(row: &RowView) -> Style {
    if row.is_selected {
        selected_bg_style()
    } else {
        base_style()
    }
}

// --- Helpers -------------------------------------------------------------

fn write_char(buf: &mut Buffer, x: u16, y: u16, max_x: u16, ch: char, style: Style) -> u16 {
    if x >= max_x {
        return x;
    }
    buf.cell_mut((x, y)).unwrap().set_char(ch).set_style(style);
    x.saturating_add(1)
}

fn fill_background(area: Rect, buf: &mut Buffer) {
    let max_x = area.x.saturating_add(area.width);
    for x in area.x..max_x {
        for y in area.y..area.y + area.height {
            buf.cell_mut((x, y)).unwrap().set_style(base_style());
        }
    }
}

// --- Styles --------------------------------------------------------------

const PANEL_BG: Color = Color::Rgb(24, 24, 24);

fn base_style() -> Style {
    Style::default().fg(Color::Rgb(200, 200, 200)).bg(PANEL_BG)
}

fn title_style() -> Style {
    Style::default()
        .fg(Color::Rgb(180, 180, 180))
        .bg(Color::Rgb(35, 35, 35))
        .add_modifier(Modifier::BOLD)
}

fn empty_message_style() -> Style {
    Style::default()
        .fg(Color::Rgb(120, 120, 120))
        .bg(PANEL_BG)
        .add_modifier(Modifier::ITALIC)
}

fn dir_prefix_style() -> Style {
    Style::default().fg(Color::Rgb(120, 170, 220)).bg(PANEL_BG)
}

fn dir_name_style() -> Style {
    Style::default()
        .fg(Color::Rgb(180, 210, 240))
        .bg(PANEL_BG)
        .add_modifier(Modifier::BOLD)
}

fn file_prefix_style() -> Style {
    Style::default().fg(Color::Rgb(120, 120, 120)).bg(PANEL_BG)
}

fn file_name_style() -> Style {
    Style::default().fg(Color::Rgb(200, 200, 200)).bg(PANEL_BG)
}

fn selected_prefix_style() -> Style {
    Style::default()
        .fg(Color::Rgb(40, 40, 40))
        .bg(Color::Rgb(90, 130, 190))
}

fn selected_name_style() -> Style {
    Style::default()
        .fg(Color::Rgb(255, 255, 255))
        .bg(Color::Rgb(90, 130, 190))
        .add_modifier(Modifier::BOLD)
}

fn selected_bg_style() -> Style {
    Style::default().bg(Color::Rgb(90, 130, 190))
}
