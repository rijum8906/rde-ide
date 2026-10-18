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
    layout::Layout,
    state::{
        bottom_bar::BottomBarState, editor_panel::EditorPanelState, left_panel::LeftPanelState,
        top_bar::TopBarState,
    },
};

/// The width of the left panel
pub const LEFT_PANEL_WIDTH: u16 = 25;
/// The width of the right panel
pub const RIGHT_PANEL_WIDTH: u16 = 25;

/// Returns the calcualted height of the panels
/// e.g.(left_panel, right_panel, editor_panel)
pub fn calc_panel_height(screen_height: u16) -> u16 {
    screen_height - TOP_BAR_HEIGHT - BOTTOM_BAR_HEIGHT
}

/// Returns the calcualted width of the editor_panel
/// i.e. after openning or closing of the side panels
pub fn calc_editor_panel_width(screen_width: u16, layout_states: Layout) -> u16 {
    let mut res = screen_width;

    if layout_states.left_panel_open {
        res -= LEFT_PANEL_WIDTH;
    }

    if layout_states.right_panel_open {
        res -= RIGHT_PANEL_WIDTH;
    }

    res
}

/// Calculates the x coordinate of the editor body
pub fn calc_editor_panel_x(layout_states: Layout) -> u16 {
    if layout_states.left_panel_open {
        LEFT_PANEL_WIDTH
    } else {
        0
    }
}

/// Persistent widget states across frames.
#[derive(Debug, Default)]
pub struct Widgets {
    pub top_bar: TopBar,
    pub bottom_bar: BottomBar,
    pub left_panel: LeftPanel,
    pub editor_panel: EditorPanel,
}

#[derive(Debug, Default)]
pub struct TopBar {
    pub state: TopBarState,
    pub geometry: Rect,
}

#[derive(Debug, Default)]
pub struct BottomBar {
    pub state: BottomBarState,
    pub geometry: Rect,
}

#[derive(Debug, Default)]
pub struct LeftPanel {
    pub state: LeftPanelState,
    pub geometry: Rect,
    pub is_open: bool,
}

#[derive(Debug, Default)]
pub struct EditorPanel {
    pub state: EditorPanelState,
    pub geometry: Rect,
}
