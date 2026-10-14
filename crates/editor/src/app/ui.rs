//! UI Module for IDE project.
//!
//! This module provides the user interface components and functionality for the IDE project.

use domain::{
    buffer::Buffer,
    error::{IdeError, Result},
};
use ratatui::{DefaultTerminal, Frame};
use tui::{
    constant::{MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH},
    ui::UiState,
    viewmodel::bottom_bar::BottomBarView,
    widgets::{bottom_bar::BottomBar, screen_too_small::ScreenTooSmall, top_bar::TopBar},
};

pub struct Ui {
    state: UiState,
}

impl Ui {
    pub fn new() -> Result<Self> {
        // Query initial terminal size for layout and validation
        let (width, height) = crossterm::terminal::size()?;

        Self::validate_screen_size(height, width)?;

        // Only initialize color_eyre if not already initialized by the main binary
        let _ = color_eyre::install();

        // Initialize UiState with dimensions
        let state = UiState::new(height, width)?;

        Ok(Ui { state })
    }

    /// Validates that the terminal meets minimum size requirements.
    pub fn validate_screen_size(screen_height: u16, screen_width: u16) -> Result<()> {
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

    pub fn draw(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| self.render(frame))?;
        Ok(())
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        // Update layout state when dimensions meet minimum requirements
        if Self::validate_screen_size(height, width).is_ok() {
            self.state.resize(height, width);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if let Err(IdeError::ScreenTooSmall {
            min_w: _,
            min_h: _,
            cur_w,
            cur_h,
        }) = Self::validate_screen_size(area.height, area.width)
        {
            frame.render_widget(ScreenTooSmall::new(cur_w, cur_h), frame.area());
        } else {
            self.render_ui(frame);
        }
    }

    fn render_ui(&mut self, frame: &mut Frame) {
        let buffer = Buffer::scratch("unnamed");

        let buffers: Vec<Buffer> = vec![buffer.clone(), buffer.clone(), buffer.clone()];
        let top_bar = TopBar::new(&buffers, 0);
        let top_bar_rect = self.state.layout_state().top_bar;
        frame.render_widget(top_bar, top_bar_rect);

        let bottom_bar_view = BottomBarView::default();
        let bottom_bar_area = self.state.layout_state().bottom_bar;
        let bottom_bar = BottomBar::new(&bottom_bar_view);
        frame.render_stateful_widget(
            bottom_bar,
            bottom_bar_area,
            self.state.bottom_bar_state_mut(),
        );
    }

    pub fn state_mut(&mut self) -> &mut UiState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_screen_size_valid() {
        assert!(Ui::validate_screen_size(MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH).is_ok());
        assert!(Ui::validate_screen_size(MIN_SCREEN_HEIGHT + 10, MIN_SCREEN_WIDTH + 20).is_ok());
    }

    #[test]
    fn test_validate_screen_size_too_small() {
        assert!(Ui::validate_screen_size(MIN_SCREEN_HEIGHT - 1, MIN_SCREEN_WIDTH).is_err());
        assert!(Ui::validate_screen_size(MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH - 1).is_err());
        assert!(Ui::validate_screen_size(10, 20).is_err());
    }
}
