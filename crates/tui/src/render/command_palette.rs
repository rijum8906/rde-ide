//! Command palette rendering.
//!
//! In command mode, centers a modal command editor over the layout. The
//! palette is drawn after every panel so it always sits on top.

use editor_core::input::mode::InputMode;
use ratatui::{Frame, layout::Rect};

use crate::{
    ui::Ui, viewmodel::command_palette::CommandPaletteView,
    widgets::command_palette::CommandPaletteWidget,
};

/// Widest the palette grows; keeps long lines from spanning the whole screen.
const MAX_WIDTH: u16 = 72;

/// Palette height: top border, one input row, bottom border.
const HEIGHT: u16 = 3;

/// Space kept free on either side of the card.
const HORIZONTAL_MARGIN: u16 = 6;

/// Draw the command palette when the editor is in command mode.
pub fn render_command_palette(ui: &Ui, frame: &mut Frame, mode: InputMode) {
    if mode != InputMode::Command {
        return;
    }

    let Some(area) = palette_area(frame.area()) else {
        return;
    };

    let state = &ui.widgets.bottom_bar.state;
    let view = CommandPaletteView {
        prompt: ":",
        text: &state.command_line,
        cursor: state.command_cursor,
    };

    frame.render_widget(CommandPaletteWidget::new(&view), area);
}

/// Place the card in the middle of the screen.
fn palette_area(screen: Rect) -> Option<Rect> {
    if screen.width < 40 || screen.height < HEIGHT + 4 {
        return None;
    }

    let width = MAX_WIDTH.min(screen.width.saturating_sub(HORIZONTAL_MARGIN));
    let x = screen.x + (screen.width - width) / 2;
    let y = screen.y + (screen.height - HEIGHT) / 2;

    Some(Rect {
        x,
        y,
        width,
        height: HEIGHT,
    })
}

#[cfg(test)]
mod tests {
    use super::palette_area;
    use ratatui::layout::Rect;

    #[test]
    fn centers_the_palette_on_screen() {
        let area = palette_area(Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        })
        .expect("wide screen has a palette area");

        assert_eq!(area.width, 72);
        assert_eq!(area.x, 14);
        assert_eq!(area.height, 3);
        assert_eq!(area.y, 18);
    }

    #[test]
    fn no_palette_on_tiny_screens() {
        assert!(
            palette_area(Rect {
                x: 0,
                y: 0,
                width: 30,
                height: 10,
            })
            .is_none()
        );
    }
}
