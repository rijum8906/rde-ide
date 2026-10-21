//! Editor panel rendering: builds line views for the visible viewport and
//! draws them.
//!
//! Only the lines between `scroll_y` and `scroll_y + height` are fetched from
//! the buffer. Building every line each frame made redraw cost grow with the
//! document, which dominated the latency of a single key press.

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
        ui.as_mut(),
    );
}

/// Width reserved for line numbers, including the trailing space.
const GUTTER_WIDTH: u16 = 5;

fn build_editor_panel_view(ws: &mut WorkspaceState, ui: &mut Ui) -> EditorPanelView {
    let Some(buffer) = ws.active_buffer() else {
        return EditorPanelView::empty();
    };

    let area = ui.widgets.editor_panel.geometry;
    let viewport_height = area.height as usize;
    let text_width = area.width.saturating_sub(GUTTER_WIDTH) as usize;

    let state = &mut ui.widgets.editor_panel.state;
    // Refresh the cached line count before scroll correction; clamping and
    // cursor visibility depend on it.
    state.line_count = buffer.line_count();
    state.ensure_cursor_visible(viewport_height, text_width);

    let first_line = state.scroll_y.min(buffer.line_count());
    let end_line = (first_line + viewport_height).min(buffer.line_count());

    let lines = (first_line..end_line)
        .map(|index| LineView {
            number: index + 1,
            text: buffer.line(index).unwrap_or_default(),
            is_current: index == state.cursor_line,
            selection: None,
        })
        .collect();

    EditorPanelView {
        lines,
        cursor_screen: Some((
            state.cursor_line.saturating_sub(state.scroll_y) as u16,
            state.cursor_col.saturating_sub(state.scroll_x) as u16,
        )),
        gutter_width: GUTTER_WIDTH,
        is_focused: true,
        highlight_current_line: true,
        empty_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn view_contains_only_the_visible_viewport() {
        let text = (0..100)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");

        let mut workspace = WorkspaceState::new(".");
        workspace.push_buffer(Buffer::with_contents("test.txt", &text));

        let mut ui = Ui::default();
        ui.widgets.editor_panel.geometry = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 10,
        };
        ui.widgets.editor_panel.state.set_line_count(100);
        ui.widgets.editor_panel.state.cursor_line = 50;
        ui.widgets.editor_panel.state.cursor_col = 2;

        let view = build_editor_panel_view(&mut workspace, &mut ui);

        // The cursor at line 50 scrolls the viewport to lines 41..=50.
        assert_eq!(view.lines.len(), 10);
        assert_eq!(view.lines[0].number, 42);
        assert_eq!(view.lines[0].text, "line 41");
        assert_eq!(view.cursor_screen, Some((9, 2)));
        assert!(view.lines[9].is_current);
    }
}
