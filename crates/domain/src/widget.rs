//! Widget geometry calculated from the current screen and layout state.

use ratatui::layout::Rect;

pub const TOP_BAR_HEIGHT: u16 = 1;
pub const BOTTOM_BAR_HEIGHT: u16 = 1;

/// Calculated bounding rectangles for each widget in the IDE interface.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WidgetGeometry {
    pub top_bar: Rect,
    pub bottom_bar: Rect,
    pub editor_body: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
}

/// Screen dimensions used to calculate widget geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WidgetSize {
    pub height: u16,
    pub width: u16,
}

/// Layout inputs relevant to widget geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetLayout {
    pub fullscreen: bool,
    pub left_panel_open: bool,
    pub right_panel_open: bool,
}

/// Updates `widget` with geometry for the current screen and layout state.
pub fn compute_widget_geometry(
    widget: &mut WidgetGeometry,
    screen_size: WidgetSize,
    layout: WidgetLayout,
) {
    *widget = if layout.fullscreen {
        WidgetGeometry {
            editor_body: Rect {
                x: 0,
                y: 0,
                width: screen_size.width,
                height: screen_size.height,
            },
            ..WidgetGeometry::default()
        }
    } else {
        let top_height = TOP_BAR_HEIGHT.min(screen_size.height);
        let bottom_height = BOTTOM_BAR_HEIGHT.min(screen_size.height.saturating_sub(top_height));
        let panel_y = top_height;
        let panel_height = screen_size
            .height
            .saturating_sub(top_height)
            .saturating_sub(bottom_height);
        let side_panel_width = ((screen_size.width as f64 * 0.2) as u16).min(screen_size.width / 2);

        let left_panel = layout.left_panel_open.then_some(Rect {
            x: 0,
            y: panel_y,
            width: side_panel_width,
            height: panel_height,
        });
        let right_panel = layout.right_panel_open.then_some(Rect {
            x: screen_size.width.saturating_sub(side_panel_width),
            y: panel_y,
            width: side_panel_width,
            height: panel_height,
        });
        let left_panel = left_panel.unwrap_or_default();
        let right_panel = right_panel.unwrap_or_default();

        WidgetGeometry {
            top_bar: Rect {
                x: 0,
                y: 0,
                width: screen_size.width,
                height: top_height,
            },
            bottom_bar: Rect {
                x: 0,
                y: screen_size.height.saturating_sub(bottom_height),
                width: screen_size.width,
                height: bottom_height,
            },
            editor_body: Rect {
                x: left_panel.width,
                y: panel_y,
                width: screen_size
                    .width
                    .saturating_sub(left_panel.width)
                    .saturating_sub(right_panel.width),
                height: panel_height,
            },
            left_panel,
            right_panel,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_geometry_for_open_side_panels() {
        let mut geometry = WidgetGeometry::default();
        compute_widget_geometry(
            &mut geometry,
            WidgetSize {
                width: 100,
                height: 50,
            },
            WidgetLayout {
                fullscreen: false,
                left_panel_open: true,
                right_panel_open: true,
            },
        );

        assert_eq!(geometry.left_panel.width, 20);
        assert_eq!(geometry.right_panel.width, 20);
        assert_eq!(geometry.editor_body.x, 20);
        assert_eq!(geometry.editor_body.width, 60);
    }

    #[test]
    fn computes_fullscreen_editor_geometry() {
        let mut geometry = WidgetGeometry::default();
        compute_widget_geometry(
            &mut geometry,
            WidgetSize {
                width: 100,
                height: 50,
            },
            WidgetLayout {
                fullscreen: true,
                left_panel_open: false,
                right_panel_open: false,
            },
        );

        assert_eq!(geometry.top_bar, Rect::default());
        assert_eq!(geometry.editor_body, Rect::new(0, 0, 100, 50));
    }
}
