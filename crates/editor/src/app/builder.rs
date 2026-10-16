use tui::{
    state::bottom_bar::MessageKind,
    viewmodel::{
        bottom_bar::{BottomBarMode, BottomBarView},
        top_bar::{TabView, TopBarView},
    },
};

use crate::app::App;

impl App {
    /// Build the top-bar view model from current workspace state.
    ///
    /// `tabs` must outlive the returned view; the caller owns the Vec.
    pub fn build_top_bar_view(&self) -> TopBarView {
        let active = self.workspace.active_buffer_idx();
        TopBarView {
            tabs: self
                .workspace
                .buffers()
                .iter()
                .enumerate()
                .map(|(i, b)| TabView {
                    title: b.display_name().to_string(),
                    is_active: i == active,
                    is_dirty: b.is_dirty(),
                })
                .collect(),
        }
    }

    pub fn build_bottom_bar_view(&self) -> BottomBarView {
        BottomBarView {
            mode: BottomBarMode::Status,
            mode_label: self.mode.to_str(),
            file_name: self.workspace.buffers()[self.workspace.active_buffer_idx()]
                .display_name()
                .to_string(),
            is_dirty: false,
            cursor: (0, 1),
            line_count: 30,
            message: Some("Hi".to_string()),
            message_kind: Some(MessageKind::Info),
            hint: None,
        }
    }
}
