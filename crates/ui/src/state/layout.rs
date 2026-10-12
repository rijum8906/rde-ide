//! UI Layout State Management for IDE project.
//!
//! This module provides the state management for the layout of the user interface
//! components of the IDE project.

use crate::constant::{MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH};
use crate::error::{Result, UiError};

/// Manages the overall layout mode and individual widget states of the UI.
pub struct LayoutState {
    layout_mode: LayoutMode,
    widget_states: WidgetStates,
}

impl LayoutState {
    /// Creates a new `LayoutState`, validating screen dimensions against minimum requirements.
    pub fn new(layout_mode: LayoutMode, screen_height: u16, screen_width: u16) -> Result<Self> {
        Self::validate_screen_size(screen_height, screen_width)?;

        let widget_states = WidgetStates::default(screen_height, screen_width);

        Ok(Self {
            layout_mode,
            widget_states,
        })
    }

    /// Validates that the terminal meets minimum size requirements.
    pub fn validate_screen_size(screen_height: u16, screen_width: u16) -> Result<()> {
        if screen_height < MIN_SCREEN_HEIGHT || screen_width < MIN_SCREEN_WIDTH {
            return Err(UiError::ScreenTooSmall(
                MIN_SCREEN_HEIGHT,
                MIN_SCREEN_WIDTH,
                screen_height,
                screen_width,
            ));
        }
        Ok(())
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
                    bottom: screen_height - 1,
                    left: 0,
                    right: 0,
                },
                is_hidden: false,
                z_index: 10,
            },
            bottom_bar: UiWidgetState {
                geometry: RectGeometry {
                    top: screen_height - 2,
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
                    right: screen_width - 30,
                },
                is_hidden: false,
                z_index: 5,
            },
            right_panel: UiWidgetState {
                geometry: RectGeometry {
                    top: 1,
                    bottom: 2,
                    left: screen_width - 30,
                    right: 0,
                },
                is_hidden: true,
                z_index: 5,
            },
        }
    }
}
