//! Panel identifiers.
//!
//! [`PanelType`] names the focusable regions of the layout. It is kept small
//! and `Copy` so it can live directly in [`crate::layout::Layout`].

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
    Left,
    Right,
    Editor,
}
