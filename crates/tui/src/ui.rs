//! Composite UI state.
//!
//! [`Ui`] groups the layout policy ([`Layout`]) with the persistent widget
//! state ([`Widgets`]) so the two can be passed around as one value.

use crate::{layout::Layout, widgets::Widgets};

#[derive(Debug, Default)]
pub struct Ui {
    pub layout: Layout,
    pub widgets: Widgets,
}
