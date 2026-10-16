//! Layout policy states: panel visibility and full-screen toggles.

/// User layout state controlling which panels are visible and fullscreen mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LayoutStates {
    pub left_panel_open: bool,
    pub right_panel_open: bool,
    pub mode: LayoutMode,
}

/// Active display mode for the overall application layout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    #[default]
    Normal,
    FullScreen,
}
