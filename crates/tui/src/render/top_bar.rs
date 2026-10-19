//! Top bar rendering: builds the tab strip view and draws it.

use domain::workspace::WorkspaceState;
use ratatui::Frame;

use crate::{
    ui::Ui,
    viewmodel::top_bar::{TabView, TopBarView},
    widgets::top_bar::TopBarWidget,
};

pub fn render_top_bar(ws: &mut WorkspaceState, ui: &mut Ui, frame: &mut Frame) {
    let top_bar_view = build_top_bar_view(ws);
    let top_bar = TopBarWidget::new(&top_bar_view);
    frame.render_stateful_widget(
        top_bar,
        ui.widgets.top_bar.geometry,
        &mut ui.widgets.top_bar.state,
    );
}

fn build_top_bar_view<'a>(workspace: &'a mut WorkspaceState) -> TopBarView<'a> {
    let active = workspace.active_buffer_idx();
    TopBarView {
        tabs: workspace
            .buffers()
            .iter()
            .enumerate()
            .map(|(i, b)| TabView {
                title: b.display_name(),
                is_active: i == active,
                is_dirty: b.is_dirty(),
            })
            .collect(),
    }
}
