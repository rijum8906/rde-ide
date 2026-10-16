use domain::error::{IdeError, Result};
use ratatui::Frame;
use tui::{
    constant::{MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH},
    widgets::{
        Size, WidgetGeometry, bottom_bar::BottomBar, left_panel::LeftPanel,
        screen_too_small::ScreenTooSmall, top_bar::TopBar,
    },
};

use crate::app::App;

impl App {
    pub fn render_into(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Step 1: check the screen size
        if let Err(e) = Self::validate_screen_size(area.height, area.width) {
            let screen_too_small = ScreenTooSmall::from_err(e);
            frame.render_widget(screen_too_small, area);
            return;
        }

        // Step 2: Render UI
        self.render_ui(frame);
    }

    fn render_ui(&mut self, frame: &mut Frame) {
        // Step 1: calculate and resolve the geometry of the layout elements
        self.widget_geometry =
            WidgetGeometry::compute_with_layout(Size::from(frame.area()), &self.layout_states);

        // Step 2: render the top bar
        let top_bar_view = Self::build_top_bar_view(&self.workspace);
        let top_bar = TopBar::new(&top_bar_view);
        frame.render_stateful_widget(
            top_bar,
            self.widget_geometry.top_bar,
            &mut self.widget_states.top_bar,
        );

        // Step 3: render the bottom bar
        let bottom_bar_view = self.build_bottom_bar_view();
        let bottom_bar = BottomBar::new(&bottom_bar_view);
        frame.render_stateful_widget(
            bottom_bar,
            self.widget_geometry.bottom_bar,
            &mut self.widget_states.bottom_bar,
        );

        // Step 4: render left panel
        let side_panel_view = self.build_side_panel();
        let side_panel = LeftPanel::new(&side_panel_view);
        frame.render_stateful_widget(
            side_panel,
            self.widget_geometry.left_panel,
            &mut self.widget_states.left_panel,
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
