//! View models: per-frame snapshots handed to widgets.
//!
//! A view model is built by the [`crate::render`] layer from domain and UI
//! state and is immutable for the duration of a frame. Widgets read views;
//! they never reach into the workspace directly.

pub mod bottom_bar;
pub mod editor_panel;
pub mod left_panel;
pub mod top_bar;
