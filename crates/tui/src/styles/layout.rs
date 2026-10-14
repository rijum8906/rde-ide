//! Layout state: mode, per-widget geometry, and visibility.
//!
//! This module computes widget rectangles from a `LayoutPlan` and the
//! current terminal area. It is deliberately domain-free: `tui` renders
//! whatever the caller passes in.

use domain::error::{IdeError, Result};
use ratatui::layout::Rect;

use crate::constant::{
    BOTTOM_BAR_HEIGHT, MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH, SIDE_PANEL_WIDTH, TOP_BAR_HEIGHT,
};

/// User-selected layout policy. This is what the user *wants*; the resolved
/// geometry may differ (clamped, hidden in fullscreen, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutPlan {
    pub mode: LayoutMode,
    pub show_left_panel: bool,
    pub show_right_panel: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutMode {
    #[default]
    Normal,
    Fullscreen,
}

/// Resolved geometry for the current frame. Recomputed whenever the terminal
/// is resized or the plan changes; the *plan* is what persists user intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutState {
    pub plan: LayoutPlan,
    pub top_bar: Rect,
    pub bottom_bar: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
    pub editor: Rect,
    /// The full terminal area this layout was computed for. Useful for
    /// widgets that need to know total bounds (e.g. centering).
    pub screen: Rect,
}

impl LayoutState {
    /// Compute layout geometry for the given plan and terminal area.
    ///
    /// Returns `Err(TuiError::TooSmall)` if the area is below minimum size.
    pub fn compute(plan: LayoutPlan, area: Rect) -> Result<Self> {
        if area.width < MIN_SCREEN_WIDTH || area.height < MIN_SCREEN_HEIGHT {
            return Err(IdeError::ScreenTooSmall {
                min_w: MIN_SCREEN_WIDTH,
                min_h: MIN_SCREEN_HEIGHT,
                cur_w: area.width,
                cur_h: area.height,
            });
        }

        let mut state = LayoutState {
            plan,
            screen: area,
            ..Default::default()
        };

        match plan.mode {
            LayoutMode::Fullscreen => {
                // Everything goes to the editor; chrome collapses to zero
                // rects. Callers checking `is_hidden()` or just `.width == 0`
                // will skip rendering them.
                state.top_bar = Rect::default();
                state.bottom_bar = Rect::default();
                state.left_panel = Rect::default();
                state.right_panel = Rect::default();
                state.editor = area;
            }
            LayoutMode::Normal => {
                state.top_bar = Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: TOP_BAR_HEIGHT,
                };

                state.bottom_bar = Rect {
                    x: area.x,
                    y: area.y + area.height - BOTTOM_BAR_HEIGHT,
                    width: area.width,
                    height: BOTTOM_BAR_HEIGHT,
                };

                // Body sits between top and bottom bars.
                let body_y = area.y + TOP_BAR_HEIGHT;
                let body_height = area
                    .height
                    .saturating_sub(TOP_BAR_HEIGHT)
                    .saturating_sub(BOTTOM_BAR_HEIGHT);
                let mut body_x = area.x;
                let mut body_width = area.width;

                // Left panel.
                if plan.show_left_panel {
                    let w = SIDE_PANEL_WIDTH.min(body_width);
                    state.left_panel = Rect {
                        x: body_x,
                        y: body_y,
                        width: w,
                        height: body_height,
                    };
                    body_x += w;
                    body_width = body_width.saturating_sub(w);
                }

                // Right panel.
                if plan.show_right_panel {
                    let w = SIDE_PANEL_WIDTH.min(body_width);
                    let x = area.x + area.width - w;
                    state.right_panel = Rect {
                        x,
                        y: body_y,
                        width: w,
                        height: body_height,
                    };
                    body_width = body_width.saturating_sub(w);
                }

                // Editor fills whatever remains of the body.
                state.editor = Rect {
                    x: body_x,
                    y: body_y,
                    width: body_width,
                    height: body_height,
                };
            }
        }

        Ok(state)
    }

    /// Convenience: hide the top bar in fullscreen mode.
    pub fn is_top_bar_visible(&self) -> bool {
        self.top_bar.width > 0 && self.top_bar.height > 0
    }

    pub fn is_bottom_bar_visible(&self) -> bool {
        self.bottom_bar.width > 0 && self.bottom_bar.height > 0
    }

    pub fn is_left_panel_visible(&self) -> bool {
        self.left_panel.width > 0 && self.left_panel.height > 0
    }

    pub fn is_right_panel_visible(&self) -> bool {
        self.right_panel.width > 0 && self.right_panel.height > 0
    }
}
