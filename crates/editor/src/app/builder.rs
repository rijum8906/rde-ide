use tui::viewmodel::top_bar::{TabView, TopBarView};

use crate::app::App;

impl App {
    /// Build the top-bar view model from current workspace state.
    ///
    /// `tabs` must outlive the returned view; the caller owns the Vec.
    pub fn build_top_bar_view<'a>(&'a self, tabs: &'a mut Vec<TabView<'a>>) -> TopBarView<'a> {
        tabs.clear();
        // if it's not set then use default `0`
        let active = self.workspace.active_buffer_idx();

        tabs.extend(
            self.workspace
                .buffers()
                .iter()
                .enumerate()
                .map(|(i, b)| TabView {
                    title: b.display_name(),
                    is_active: i == active,
                    is_dirty: b.is_dirty(),
                }),
        );

        TopBarView { tabs }
    }
}
