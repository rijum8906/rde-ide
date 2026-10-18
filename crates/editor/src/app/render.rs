use domain::error::{IdeError, Result};
use ratatui::Frame;
use tui::{
    constant::{MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH},
    widgets::{
        bottom_bar::BottomBarWidget, editor_panel::EditorPanelWidget, left_panel::LeftPanelWidget,
        screen_too_small::ScreenTooSmall, top_bar::TopBarWidget,
    },
};

use crate::app::App;

impl App {
    pub fn render_into(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Step 1: check the screen size
        if let Err(e) = Self::validate_screen_size(area.height, area.width) {
            let screen_too_small = ScreenTooSmall::new(area.width, area.height);
            frame.render_widget(screen_too_small, area);
            return;
        }

        // Step 2: Render UI
        self.render_ui(frame);
    }

    fn render_ui(&mut self, frame: &mut Frame) {
        let screen_rect = frame.area();
        // Step 1: calculate and resolve the geometry of the layout elements
        self.ui.layout.screen_size = (screen_rect.height, screen_rect.width);
        self.calc_widgets_geometry();

        // Step 2: render the top bar
        let top_bar_view = Self::build_top_bar_view(&self.workspace);
        let top_bar = TopBarWidget::new(&top_bar_view);
        frame.render_stateful_widget(
            top_bar,
            self.ui.widgets.top_bar.geometry,
            &mut self.ui.widgets.top_bar.state,
        );

        // Step 3: render the bottom bar
        let bottom_bar_view = self.build_bottom_bar_view();
        let bottom_bar = BottomBarWidget::new(&bottom_bar_view);
        frame.render_stateful_widget(
            bottom_bar,
            self.ui.widgets.bottom_bar.geometry,
            &mut self.ui.widgets.bottom_bar.state,
        );

        // Step 4: render panels
        if self.ui.widgets.left_panel.is_open {
            let left_panel_view = self.build_side_panel();
            let left_panel = LeftPanelWidget::new(&left_panel_view);
            frame.render_stateful_widget(
                left_panel,
                self.ui.widgets.left_panel.geometry,
                &mut self.ui.widgets.left_panel.state,
            );
        }
        // TODO: create the right panel and implement here

        let editor_panel_view = self.build_editor_panel();
        let editor_panel = EditorPanelWidget::new(&editor_panel_view);
        frame.render_stateful_widget(
            editor_panel,
            self.ui.widgets.editor_panel.geometry,
            &mut self.ui.widgets.editor_panel.state,
        );
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
}
