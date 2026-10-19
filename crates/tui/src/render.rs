//! Frame rendering entry point.
//!
//! Validates the terminal size, recalculates widget geometry, then draws each
//! panel from the current domain and UI state. Widgets stay dumb: everything
//! they need is assembled here and handed over as a view model.
//!
//! Call [`render_into`] once per frame with the frame's mutable workspace, UI
//! state, and input state.

pub(super) mod bottom_bar;
pub(super) mod command_palette;
pub(super) mod editor_panel;
pub(super) mod left_panel;
pub(super) mod top_bar;

use domain::{
    error::{IdeError, Result},
    workspace::WorkspaceState,
};
use editor_core::input::Input;
use ratatui::Frame;

use crate::{
    constant::{MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH},
    geometry::calc_widgets_geometry,
    render::{
        bottom_bar::render_bottom_bar, command_palette::render_command_palette,
        editor_panel::render_editor_panel, left_panel::render_left_panel, top_bar::render_top_bar,
    },
    ui::Ui,
    widgets::screen_too_small::ScreenTooSmall,
};

pub fn render_into(ws: &mut WorkspaceState, ui: &mut Ui, input: &Input, frame: &mut Frame) {
    let area = frame.area();

    // Step 1: check the screen size
    if let Err(_e) = validate_screen_size(area.height, area.width) {
        let screen_too_small = ScreenTooSmall::new(area.width, area.height);
        frame.render_widget(screen_too_small, area);
        return;
    }

    // Step 2: Render UI
    render_ui(ws, ui, input, frame);
}

fn render_ui(ws: &mut WorkspaceState, ui: &mut Ui, input: &Input, frame: &mut Frame) {
    let screen_rect = frame.area();
    // Step 1: calculate and resolve the geometry of the layout elements
    ui.layout.screen_size = (screen_rect.height, screen_rect.width);
    calc_widgets_geometry(ui);

    // Step 2: render the top bar
    render_top_bar(ws, ui, frame);

    // Step 3: render the bottom bar
    render_bottom_bar(ws, ui, frame, input.mode);

    // Step 4: render panels
    if ui.widgets.left_panel.is_open {
        render_left_panel(ws, ui, frame);
    }
    // TODO: create the right panel and implement here

    render_editor_panel(ws, ui, frame);

    // Command mode opens a modal editor over the layout, so it is drawn last.
    render_command_palette(ui, frame, input.mode);
}

/// Validates that the terminal meets minimum size requirements.
fn validate_screen_size(screen_height: u16, screen_width: u16) -> Result<()> {
    if screen_height < MIN_SCREEN_HEIGHT || screen_width < MIN_SCREEN_WIDTH {
        return Err(IdeError::ScreenTooSmall {
            min_w: MIN_SCREEN_WIDTH,
            min_h: MIN_SCREEN_HEIGHT,
            cur_w: screen_width,
            cur_h: screen_height,
        });
    }
    Ok(())
}
