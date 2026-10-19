//! Terminal UI for the IDE.
//!
//! The crate renders domain state; it does not decide what that state is.
//! Data flows in one direction each frame:
//!
//! 1. [`render`] builds a view model ([`viewmodel`]) from the workspace and
//!    [`ui`] state.
//! 2. A [`widgets`] implementation draws the view into a Ratatui buffer.
//! 3. Persistent per-widget state ([`state`]) survives between frames, plus
//!    [`layout`] and [`geometry`] for sizing and [`constant`] for shared
//!    dimensions.

pub mod constant;
pub mod geometry;
pub mod layout;
pub mod panel;
pub mod render;
pub mod state;
pub mod ui;
pub mod viewmodel;
pub mod widgets;
