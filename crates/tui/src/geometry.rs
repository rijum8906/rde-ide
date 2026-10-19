//! Widget geometry calculation.
//!
//! Given the terminal size and the current [`Layout`](crate::layout::Layout)
//! policy, computes a [`Rect`] for every widget. Widgets never size
//! themselves; they draw into the rect assigned here.

use ratatui::layout::Rect;

use crate::{
    constant::{BOTTOM_BAR_HEIGHT, TOP_BAR_HEIGHT},
    layout::LayoutMode,
    ui::Ui,
};

pub fn calc_widgets_geometry(ui: &mut Ui) {
    let screen_size = ui.layout.screen_size;
    if ui.layout.mode == LayoutMode::FullScreen {
        ui.widgets.editor_panel.geometry = Rect {
            x: 0,
            y: 0,
            height: screen_size.0,
            width: screen_size.1,
        }
    } else if ui.layout.mode == LayoutMode::Normal {
        let top_height = TOP_BAR_HEIGHT.min(screen_size.0);
        let bottom_height = BOTTOM_BAR_HEIGHT.min(screen_size.1.saturating_sub(top_height));
        let panel_y = top_height;
        let panel_height = screen_size
            .0
            .saturating_sub(top_height)
            .saturating_sub(bottom_height);
        let side_panel_width = ((screen_size.1 as f64 * 0.2) as u16).min(screen_size.1 / 2);

        // ---- Top Bar ----
        ui.widgets.top_bar.geometry = Rect {
            x: 0,
            y: 0,
            width: screen_size.1,
            height: top_height,
        };

        // ---- Bottom Bar ----
        ui.widgets.bottom_bar.geometry = Rect {
            x: 0,
            y: screen_size.0.saturating_sub(bottom_height),
            width: screen_size.1,
            height: bottom_height,
        };

        // ---- Left PaneL ----
        let left_panel = ui.widgets.left_panel.is_open.then_some(Rect {
            x: 0,
            y: panel_y,
            width: side_panel_width,
            height: panel_height,
        });
        let left_panel = left_panel.unwrap_or_default();
        ui.widgets.left_panel.geometry = left_panel;

        // TODO: Do the same for right
        let right_panel = Rect::default();

        // ---- Editor Panel ----
        ui.widgets.editor_panel.geometry = Rect {
            x: left_panel.width,
            y: panel_y,
            width: screen_size
                .1
                .saturating_sub(left_panel.width)
                .saturating_sub(right_panel.width),
            height: panel_height,
        };
    }
}
