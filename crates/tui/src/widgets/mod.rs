//! This module contains all the Wigets needed to build the IDE
//!
//! - wigets may also include states to manage them

pub mod bottom_bar;
pub mod editor_panel;
pub mod left_panel;
pub mod screen_too_small;
pub mod top_bar;

use ratatui::layout::Rect;

use crate::{
    constant::{BOTTOM_BAR_HEIGHT, TOP_BAR_HEIGHT},
    layout::{LayoutMode, LayoutStates},
    state::{
        bottom_bar::BottomBarState, editor_panel::EditorState, left_panel::LeftPanelState,
        top_bar::TopBarState,
    },
};

/// Calculated bounding rectangles for each widget in the IDE interface.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WidgetGeometry {
    pub top_bar: Rect,
    pub bottom_bar: Rect,
    pub editor_body: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
}

/// Persistent widget states across frames.
#[derive(Debug, Default)]
pub struct WidgetStates {
    pub top_bar: TopBarState,
    pub bottom_bar: BottomBarState,
    pub left_panel: LeftPanelState,
    pub editor_panel: EditorState,
}

/// Screen dimension abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub height: u16,
    pub width: u16,
}

impl From<Rect> for Size {
    fn from(value: Rect) -> Self {
        Self {
            height: value.height,
            width: value.width,
        }
    }
}

impl WidgetGeometry {
    /// Compute widget geometry taking layout policy (panel visibility, fullscreen) into account.
    pub fn compute_with_layout(screen_size: Size, layout: &LayoutStates) -> WidgetGeometry {
        match layout.mode {
            LayoutMode::FullScreen => WidgetGeometry {
                top_bar: Rect::default(),
                bottom_bar: Rect::default(),
                left_panel: Rect::default(),
                right_panel: Rect::default(),
                editor_body: Rect {
                    x: 0,
                    y: 0,
                    width: screen_size.width,
                    height: screen_size.height,
                },
            },
            LayoutMode::Normal => {
                let top_height = TOP_BAR_HEIGHT.min(screen_size.height);
                let bottom_height =
                    BOTTOM_BAR_HEIGHT.min(screen_size.height.saturating_sub(top_height));
                let body_y = top_height;
                let body_height = screen_size
                    .height
                    .saturating_sub(top_height)
                    .saturating_sub(bottom_height);

                let side_panel_width =
                    ((screen_size.width as f64 * 0.2) as u16).min(screen_size.width / 2);

                let left_panel_rect = if layout.left_panel_open {
                    Rect {
                        x: 0,
                        y: body_y,
                        width: side_panel_width,
                        height: body_height,
                    }
                } else {
                    Rect::default()
                };

                let right_panel_rect = if layout.right_panel_open {
                    Rect {
                        x: screen_size.width.saturating_sub(side_panel_width),
                        y: body_y,
                        width: side_panel_width,
                        height: body_height,
                    }
                } else {
                    Rect::default()
                };

                let editor_x = left_panel_rect.width;
                let editor_width = screen_size
                    .width
                    .saturating_sub(left_panel_rect.width)
                    .saturating_sub(right_panel_rect.width);

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
                        x: editor_x,
                        y: body_y,
                        width: editor_width,
                        height: body_height,
                    },
                    left_panel: left_panel_rect,
                    right_panel: right_panel_rect,
                }
            }
        }
    }

    /// Compute widget geometry using default layout settings (normal mode, panels closed).
    pub fn compute(screen_size: Size) -> WidgetGeometry {
        Self::compute_with_layout(screen_size, &LayoutStates::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::panel::PanelType;

    use super::*;

    #[test]
    fn test_widget_geometry_normal() {
        let size = Size {
            width: 100,
            height: 50,
        };
        let geom = WidgetGeometry::compute(size);

        assert_eq!(geom.top_bar.y, 0);
        assert_eq!(geom.top_bar.height, TOP_BAR_HEIGHT);
        assert_eq!(
            geom.bottom_bar.y,
            size.height.saturating_sub(BOTTOM_BAR_HEIGHT)
        );
        assert_eq!(geom.bottom_bar.height, BOTTOM_BAR_HEIGHT);

        // Editor body sits between top and bottom bars without overlap
        assert_eq!(geom.editor_body.y, TOP_BAR_HEIGHT);
        assert_eq!(
            geom.editor_body.height,
            size.height - TOP_BAR_HEIGHT - BOTTOM_BAR_HEIGHT
        );
        assert_eq!(geom.editor_body.width, size.width);
    }

    #[test]
    fn test_widget_geometry_fullscreen() {
        let size = Size {
            width: 100,
            height: 50,
        };
        let layout = LayoutStates {
            mode: LayoutMode::FullScreen,
            left_panel_open: false,
            focused_panel: PanelType::Editor,
            right_panel_open: false,
        };
        let geom = WidgetGeometry::compute_with_layout(size, &layout);

        assert_eq!(geom.top_bar, Rect::default());
        assert_eq!(geom.bottom_bar, Rect::default());
        assert_eq!(geom.editor_body.x, 0);
        assert_eq!(geom.editor_body.y, 0);
        assert_eq!(geom.editor_body.width, 100);
        assert_eq!(geom.editor_body.height, 50);
    }

    #[test]
    fn test_widget_geometry_side_panels() {
        let size = Size {
            width: 100,
            height: 50,
        };
        let layout = LayoutStates {
            mode: LayoutMode::Normal,
            left_panel_open: true,
            right_panel_open: true,
            focused_panel: PanelType::Editor,
        };
        let geom = WidgetGeometry::compute_with_layout(size, &layout);

        assert_eq!(geom.left_panel.width, 20);
        assert_eq!(geom.right_panel.width, 20);
        assert_eq!(geom.editor_body.x, 20);
        assert_eq!(geom.editor_body.width, 60);
    }
}
