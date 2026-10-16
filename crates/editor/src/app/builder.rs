use domain::workspace::{file_tree::scan_dir, state::WorkspaceState};
use tui::viewmodel::{
    bottom_bar::{BottomBarMode, BottomBarView},
    left_panel::{LeftPanelView, RowView},
    top_bar::{TabView, TopBarView},
};

use crate::app::App;

impl App {
    /// Build the top-bar view model from workspace state.
    pub fn build_top_bar_view(workspace: &WorkspaceState) -> TopBarView<'_> {
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

    /// Build the bottom-bar view model derived from app and workspace state.
    pub fn build_bottom_bar_view(&self) -> BottomBarView {
        let active_idx = self.workspace.active_buffer_idx();
        let (file_name, is_dirty, line_count) = match self.workspace.buffers().get(active_idx) {
            Some(buffer) => (
                buffer.display_name().to_string(),
                buffer.is_dirty(),
                buffer.line_count().max(1),
            ),
            None => ("[No Name]".to_string(), false, 1),
        };

        BottomBarView {
            mode: BottomBarMode::Status,
            mode_label: self.mode.as_str(),
            file_name,
            is_dirty,
            cursor: (1, 1),
            line_count,
            message: None,
            message_kind: None,
            hint: None,
        }
    }

    pub fn build_side_panel(&self) -> LeftPanelView {
        let file_tree_nodes = scan_dir(self.workspace.root_dir()).unwrap();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_bottom_bar_default_workspace() {
        let app = App::new().unwrap();
        let view = app.build_bottom_bar_view();
        assert_eq!(view.file_name, "Unnamed");
        assert_eq!(view.mode_label, "NORMAL");
        assert_eq!(view.line_count, 1);
        assert!(!view.is_dirty);
    }

    #[test]
    fn test_build_bottom_bar_empty_workspace() {
        let mut app = App::new().unwrap();
        app.workspace.remove_buffer(0).unwrap();
        // Should safely fallback without panicking
        let view = app.build_bottom_bar_view();
        assert_eq!(view.file_name, "[No Name]");
        assert_eq!(view.mode_label, "NORMAL");
        assert_eq!(view.line_count, 1);
        assert!(!view.is_dirty);
    }
}
