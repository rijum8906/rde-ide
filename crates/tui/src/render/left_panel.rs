use domain::workspace::{WorkspaceState, file_tree::scan_dir};
use ratatui::Frame;

use crate::{
    ui::Ui,
    viewmodel::left_panel::{LeftPanelView, RowView},
    widgets::left_panel::LeftPanelWidget,
};

pub fn render_left_panel(ws: &mut WorkspaceState, ui: &mut Ui, frame: &mut Frame) {
    let left_panel_view = build_left_panel_view(ws);
    let left_panel_widget = LeftPanelWidget::new(&left_panel_view);
    frame.render_stateful_widget(
        left_panel_widget,
        ui.widgets.left_panel.geometry,
        &mut ui.widgets.left_panel.state,
    );
}

fn build_left_panel_view(ws: &mut WorkspaceState) -> LeftPanelView {
    let file_tree_nodes = scan_dir(ws.root_dir()).unwrap_or_default();
    let rows: Vec<RowView> = file_tree_nodes
        .iter()
        .map(|file_tree_node| RowView {
            name: file_tree_node.name.clone(),
            depth: 1,
            is_dir: file_tree_node.is_dir,
            is_expanded: false,
            is_selected: false,
        })
        .collect();

    LeftPanelView {
        rows,
        title: "Left Panel".to_string(),
        is_loading: false,
        empty_message: Some("No file or folder to show".to_string()),
    }
}
