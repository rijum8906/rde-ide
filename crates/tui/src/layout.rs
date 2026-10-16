//! Layout policy states: panel visibility and full-screen toggles.

/// User layout state controlling which panels are visible and fullscreen mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutStates {
    pub left_panel_open: bool,
    pub right_panel_open: bool,
    pub mode: LayoutMode,
}

impl Default for LayoutStates {
    fn default() -> Self {
        Self {
            left_panel_open: true,
            right_panel_open: false,
            mode: LayoutMode::Normal,
        }
    }
}

/// Active display mode for the overall application layout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    #[default]
    Normal,
    FullScreen,
}
