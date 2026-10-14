//! UI State Module for IDE project.
//!
//! This module provides the state management for the user interface components of the IDE project.
//! It defines the structures and logic needed to maintain and update the UI state, such as current buffer, cursor position, and mode.

use domain::error::Result;
use ratatui::layout::Rect;

use crate::{
    state::bottom_bar::BottomBarState,
    styles::layout::{LayoutPlan, LayoutState},
};

pub struct UiState {
    layout_state: LayoutState,
    bottom_bar_state: BottomBarState,
    screen_rect: Rect,
}

impl UiState {
    pub fn new(screen_height: u16, screen_width: u16) -> Result<Self> {
        let screen_rect = Rect {
            x: 0,
            y: 0,
            height: screen_height,
            width: screen_width,
        };
        Ok(Self {
            layout_state: LayoutState::compute(LayoutPlan::default(), screen_rect)?,
            bottom_bar_state: BottomBarState::new(),
            screen_rect,
        })
    }

    /// Resizes layout state when screen size changes.
    pub fn resize(&mut self, new_screen_height: u16, new_screen_width: u16) {
        let screen_rect = ratatui::layout::Rect {
            x: 0,
            y: 0,
            height: new_screen_height,
            width: new_screen_width,
        };
        if let Ok(layout_state) = crate::styles::layout::LayoutState::compute(
            self.layout_state.plan,
            screen_rect,
        ) {
            self.layout_state = layout_state;
            self.screen_rect = screen_rect;
        }
    }

    pub fn bottom_bar_state(&self) -> &BottomBarState {
        &self.bottom_bar_state
    }

    pub fn bottom_bar_state_mut(&mut self) -> &mut BottomBarState {
        &mut self.bottom_bar_state
    }

    pub fn screen_rect(&self) -> Rect {
        self.screen_rect
    }

    pub fn layout_state(&self) -> LayoutState {
        self.layout_state
    }
}
