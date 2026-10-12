//! UI State Module for IDE project.
//!
//! This module provides the state management for the user interface components of the IDE project.
//! It defines the structures and logic needed to maintain and update the UI state, such as current buffer, cursor position, and mode.

use ide_core::error::Result;

use crate::state::layout::{LayoutMode, LayoutState};

pub struct UiState {
    layout_state: LayoutState,
}

impl UiState {
    pub fn new(screen_height: u16, screen_width: u16) -> Result<Self> {
        let layout_state = LayoutState::new(LayoutMode::Normal, screen_height, screen_width)?;

        Ok(Self { layout_state })
    }

    pub fn layout_state(&self) -> &LayoutState {
        &self.layout_state
    }

    /// Resizes layout state when screen size changes.
    pub fn resize(&mut self, screen_height: u16, screen_width: u16) {
        self.layout_state.resize(screen_height, screen_width);
    }
}
