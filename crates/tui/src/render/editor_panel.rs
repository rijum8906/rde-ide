//! Editor panel rendering: builds line views from the active buffer and
//! draws them.

use domain::workspace::WorkspaceState;
use ratatui::Frame;

use crate::{
    ui::Ui,
    viewmodel::editor_panel::{EditorPanelView, LineView},
    widgets::editor_panel::EditorPanelWidget,
};

pub fn render_editor_panel(ws: &mut WorkspaceState, ui: &mut Ui, frame: &mut Frame) {
    let left_editor_view = build_editor_panel_view(ws, ui);
    let left_editor_widget = EditorPanelWidget::new(&left_editor_view);
    frame.render_stateful_widget(
        left_editor_widget,
        ui.widgets.editor_panel.geometry,
        &mut ui.widgets.editor_panel.state,
    );
}

fn build_editor_panel_view(ws: &mut WorkspaceState, ui: &mut Ui) -> EditorPanelView {
    let Some(buffer) = ws.active_buffer() else {
        return EditorPanelView::empty();
    };

    // Refresh the cached line count before building the line views; cursor
    // clamping and scroll correction depend on it.

    ui.widgets.editor_panel.state.line_count = buffer.line_count();

    let mut line_views = vec![];
    for i in 0..buffer.line_count() {
        line_views.push(LineView {
            number: i + 1,
            text: buffer.line(i).unwrap(),
            is_current: i == ui.widgets.editor_panel.state.cursor_line,
            selection: None,
        });
    }

    EditorPanelView {
        lines: line_views,
        cursor_screen: Some((
            ui.widgets.editor_panel.state.cursor_line as u16,
            ui.widgets.editor_panel.state.cursor_col as u16,
        )),
        gutter_width: 5,
        is_focused: true,
        highlight_current_line: true,
        empty_message: None,
    }
}
