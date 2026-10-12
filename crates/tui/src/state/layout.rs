//! UI Layout State Management for IDE project.
//!
//! This module provides the state management for the layout of the user interface
//! components of the IDE project.

use domain::error::Result;

/// Manages the overall layout mode and individual widget states of the UI.
pub struct LayoutState {
    widget_states: WidgetStates,
    layout_mode: LayoutMode,
}

impl LayoutState {
    /// Creates a new `LayoutState`, validating screen dimensions against minimum requirements.
    pub fn new(layout_mode: LayoutMode, screen_height: u16, screen_width: u16) -> Result<Self> {
        let widget_states = WidgetStates::default(screen_height, screen_width);

        Ok(Self {
            layout_mode,
            widget_states,
        })
    }

    pub fn set_layout_mode(&mut self, layout_mode: LayoutMode) {
        self.layout_mode = layout_mode;
    }

    pub fn layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }

    pub fn widget_states(&self) -> &WidgetStates {
        &self.widget_states
    }

    pub fn widget_states_mut(&mut self) -> &mut WidgetStates {
        &mut self.widget_states
    }

    /// Updates widget layout geometry when the terminal is resized.
    pub fn resize(&mut self, screen_height: u16, screen_width: u16) {
        // NOTE: adjust widgets geometry according to the screen size
        self.widget_states = WidgetStates::default(screen_height, screen_width);
    }
}

/// LayoutMode enum defines the different layout modes available in the IDE project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Represents a layout mode where the UI has top and bottom panels,
    /// and also left and right panels (if the user has opened any side panels).
    Normal,
    /// Fullscreen mode, where the UI occupies the entire terminal window,
    /// hiding any side panels or status bars.
    Fullscreen,
}

/// Represents the edge offsets of a panel relative to the screen borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectGeometry {
    /// Distance from the top edge
    pub top: u16,
    /// Distance from the bottom edge
    pub bottom: u16,
    /// Distance from the left edge
    pub left: u16,
    /// Distance from the right edge
    pub right: u16,
}

impl RectGeometry {
    /// Returns the calculated height of the panel given the screen height.
    pub fn height(&self, screen_height: u16) -> u16 {
        screen_height
            .saturating_sub(self.top)
            .saturating_sub(self.bottom)
    }

    /// Returns the calculated width of the panel given the screen width.
    pub fn width(&self, screen_width: u16) -> u16 {
        screen_width
            .saturating_sub(self.left)
            .saturating_sub(self.right)
    }
}

/// Holds state properties for an individual UI widget or panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiWidgetState {
    pub geometry: RectGeometry,
    pub is_hidden: bool,
    pub z_index: u8,
}

/// Collects state objects for all primary chrome components of the IDE.
pub struct WidgetStates {
    pub top_bar: UiWidgetState,
    pub bottom_bar: UiWidgetState,
    pub left_panel: UiWidgetState,
    pub right_panel: UiWidgetState,
}

impl WidgetStates {
    /// Initializes default panel geometries and visibility.
    pub fn default(screen_height: u16, screen_width: u16) -> Self {
        Self {
            top_bar: UiWidgetState {
                geometry: RectGeometry {
                    top: 0,
                    bottom: screen_height.saturating_sub(1),
                    left: 0,
                    right: 0,
                },
                is_hidden: false,
                z_index: 10,
            },
            bottom_bar: UiWidgetState {
                geometry: RectGeometry {
                    top: screen_height.saturating_sub(2),
                    bottom: 0,
                    left: 0,
                    right: 0,
                },
                is_hidden: false,
                z_index: 10,
            },
            left_panel: UiWidgetState {
                geometry: RectGeometry {
                    top: 1,
                    bottom: 2,
                    left: 0,
                    right: screen_width.saturating_sub(30),
                },
                is_hidden: false,
                z_index: 5,
            },
            right_panel: UiWidgetState {
                geometry: RectGeometry {
                    top: 1,
                    bottom: 2,
                    left: screen_width.saturating_sub(30),
                    right: 0,
                },
                is_hidden: true,
                z_index: 5,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_states_saturating_bounds() {
        // Verify no underflow panics occur on very small or zero dimensions
        let states = WidgetStates::default(0, 0);
        assert_eq!(states.top_bar.geometry.bottom, 0);
        assert_eq!(states.left_panel.geometry.right, 0);

        let states_small = WidgetStates::default(5, 10);
        assert_eq!(states_small.top_bar.geometry.bottom, 4);
        assert_eq!(states_small.bottom_bar.geometry.top, 3);
        assert_eq!(states_small.left_panel.geometry.right, 0);
    }

    #[test]
    fn test_layout_state_resize() {
        let mut state = LayoutState::new(LayoutMode::Normal, 50, 100).unwrap();
        state.resize(60, 120);
        assert_eq!(state.widget_states().top_bar.geometry.bottom, 59);
        assert_eq!(state.widget_states().left_panel.geometry.right, 90);
    }
}
