//! Composite UI state.
//!
//! [`Ui`] groups the layout policy ([`Layout`]) with the persistent widget
//! state ([`Widgets`]) so the two can be passed around as one value.

use crate::{
    layout::Layout,
    state::{
        bottom_bar::BottomBarState, editor_panel::EditorPanelState, left_panel::LeftPanelState,
        top_bar::TopBarState,
    },
    widgets::Widgets,
};

#[derive(Debug, Default)]
pub struct Ui {
    pub layout: Layout,
    pub widgets: Widgets,
}

impl AsRef<Layout> for Ui {
    fn as_ref(&self) -> &Layout {
        &self.layout
    }
}

impl AsMut<BottomBarState> for Ui {
    fn as_mut(&mut self) -> &mut BottomBarState {
        &mut self.widgets.bottom_bar.state
    }
}

impl AsMut<TopBarState> for Ui {
    fn as_mut(&mut self) -> &mut TopBarState {
        &mut self.widgets.top_bar.state
    }
}

impl AsMut<LeftPanelState> for Ui {
    fn as_mut(&mut self) -> &mut LeftPanelState {
        &mut self.widgets.left_panel.state
    }
}

// TODO:
// impl AsMut<RightPanelState> for Ui {
//     fn as_mut(&mut self) -> &mut RightPanelState {
//         &mut self.widgets.right_panel.state
//     }
// }

impl AsMut<EditorPanelState> for Ui {
    fn as_mut(&mut self) -> &mut EditorPanelState {
        &mut self.widgets.editor_panel.state
    }
}
