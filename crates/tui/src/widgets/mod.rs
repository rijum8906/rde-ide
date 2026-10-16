//! This module contains all the Wigets needed to build the IDE
//!
//! - wigets may also include states to manage them

pub mod bottom_bar;
pub mod screen_too_small;
pub mod top_bar;

use ratatui::layout::Rect;

use crate::{
    constant::{BOTTOM_BAR_HEIGHT, TOP_BAR_HEIGHT},
    state::{bottom_bar::BottomBarState, top_bar::TopBarState},
};

#[derive(Debug, Default)]
pub struct WidgetGeometry {
    pub top_bar: Rect,
    pub bottom_bar: Rect,
    pub editor_body: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
}

#[derive(Debug, Default)]
pub struct WidgetStates {
    pub top_bar: TopBarState,
    pub bottom_bar: BottomBarState,
}

#[derive(Debug)]
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
    pub fn compute(screen_size: Size) -> WidgetGeometry {
        // 20% for side panels
        let side_panel_width = (screen_size.width as f64 * 0.2) as u16;
        WidgetGeometry {
            top_bar: Rect {
                x: 0,
                y: 0,
                width: screen_size.width,
                height: TOP_BAR_HEIGHT,
            },
            bottom_bar: Rect {
                x: 0,
                y: screen_size.height - BOTTOM_BAR_HEIGHT,
                width: screen_size.width,
                height: BOTTOM_BAR_HEIGHT,
            },

            editor_body: Rect {
                x: 0,
                y: screen_size.height - BOTTOM_BAR_HEIGHT,
                width: screen_size.width,
                height: BOTTOM_BAR_HEIGHT,
            },
            left_panel: Rect {
                x: 0,
                y: 0,
                width: side_panel_width,
                height: screen_size.height,
            },
            right_panel: Rect {
                x: screen_size.width - side_panel_width,
                y: 0,
                width: side_panel_width,
                height: screen_size.height,
            },
        }
    }
}
