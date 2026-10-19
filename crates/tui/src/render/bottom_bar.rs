//! Bottom bar rendering: builds the status-line view and draws it.

use domain::workspace::WorkspaceState;
use editor_core::input::mode::InputMode;
use ratatui::Frame;

use crate::{
    ui::Ui,
    viewmodel::bottom_bar::{BottomBarMode, BottomBarView},
    widgets::bottom_bar::BottomBarWidget,
};

pub fn render_bottom_bar(
    ws: &mut WorkspaceState,
    ui: &mut Ui,
    frame: &mut Frame,
    input_mode: InputMode,
) {
    let bottom_bar_view = build_bottom_bar_view(ws, ui, input_mode);
    let bottom_bar_widget = BottomBarWidget::new(&bottom_bar_view);
    frame.render_stateful_widget(
        bottom_bar_widget,
        ui.widgets.bottom_bar.geometry,
        &mut ui.widgets.bottom_bar.state,
    );
}

fn build_bottom_bar_view(
    ws: &mut WorkspaceState,
    ui: &mut Ui,
    input_mode: InputMode,
) -> BottomBarView {
    let active_idx = ws.active_buffer_idx();
    let (file_name, is_dirty, line_count) = match ws.buffers().get(active_idx) {
        Some(buffer) => (
            buffer.display_name().to_string(),
            buffer.is_dirty(),
            buffer.line_count().max(1),
        ),
        None => ("[No Name]".to_string(), false, 1),
    };

    // Messages are owned by the widget state and expire on their own.
    ui.widgets
        .bottom_bar
        .state
        .poll_message_expiry(std::time::Instant::now());
    let (message, message_kind) = match &ui.widgets.bottom_bar.state.message {
        Some(msg) => (Some(msg.text.clone()), Some(msg.kind)),
        None => (None, None),
    };

    BottomBarView {
        mode: BottomBarMode::Status,
        mode_label: input_mode.as_str(),
        file_name,
        is_dirty,
        // The view model documents (line, col) as 1-indexed for display.
        cursor: (
            ui.widgets.editor_panel.state.cursor_line + 1,
            ui.widgets.editor_panel.state.cursor_col + 1,
        ),
        line_count,
        message,
        message_kind,
        hint: None,
    }
}
