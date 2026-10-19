//! Layout changes: panel visibility and fullscreen.

/// Layout changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutCommand {
    ToggleFullscreen,
    ToggleLeftPanel,
    ToggleRightPanel,
    FocusLeftPanel, // may live here or in Navigation — pick one
    FocusRightPanel,
}
