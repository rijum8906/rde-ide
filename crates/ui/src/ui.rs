//! UI Module for IDE project.
//!
//! This module provides the user interface components and functionality for the IDE project.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ide_core::error::{IdeError, Result};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Wrap},
};
use std::time::Duration;

use crate::{
    constant::{MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH},
    state::ui::UiState,
};

pub struct Ui {
    state: UiState,
    is_running: bool,
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

        Ok(Ui {
            state,
            is_running: true,
        })
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

    /// Runs the main application loop, drawing the UI and handling events non-blockingly.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while self.is_running {
            terminal.draw(|frame| self.render(frame))?;

            // Poll for events with a timeout to allow frequent UI refreshes/animations
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_event(key),
                    Event::Resize(width, height) => {
                        self.handle_resize(width, height);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, width: u16, height: u16) {
        // Update layout state when dimensions meet minimum requirements
        if Self::validate_screen_size(height, width).is_ok() {
            self.state.resize(height, width);
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        // Quit on Esc or Ctrl+C
        if key.code == KeyCode::Esc
            || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
        {
            self.is_running = false;
        }

        // TODO: apply proper logic
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        if let Err(IdeError::ScreenTooSmall {
            min_w,
            min_h,
            cur_w,
            cur_h,
        }) = Self::validate_screen_size(area.height, area.width)
        {
            self.render_screen_too_small(frame, area, min_w, min_h, cur_w, cur_h);
        } else {
            self.render_normal(frame, area);
        }
    }

    fn render_normal(&self, frame: &mut Frame, area: Rect) {
        let layout_state = self.state.layout_state();

        frame.render_widget(
            format!(
                "RDE IDE | Mode: {:?} | Press Esc to exit",
                layout_state.layout_mode()
            ),
            area,
        );
    }

    fn render_screen_too_small(
        &self,
        frame: &mut Frame,
        area: Rect,
        min_w: u16,
        min_h: u16,
        cur_w: u16,
        cur_h: u16,
    ) {
        // For extremely small viewports, render compact text without borders
        if area.width < 32 || area.height < 6 {
            let text = vec![
                Line::from(Span::styled(
                    "Screen too small!",
                    Style::default().fg(Color::Red).bold(),
                )),
                Line::from(Span::styled(
                    format!("{cur_w}x{cur_h} < {min_w}x{min_h}"),
                    Style::default().fg(Color::Yellow),
                )),
            ];
            let paragraph = Paragraph::new(text).alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        }

        // Determine warning box bounds (centered within frame)
        let box_width = 54.min(area.width);
        let box_height = 9.min(area.height);

        let vertical_margin = area.height.saturating_sub(box_height) / 2;
        let horizontal_margin = area.width.saturating_sub(box_width) / 2;

        let popup_area = Rect {
            x: area.x + horizontal_margin,
            y: area.y + vertical_margin,
            width: box_width,
            height: box_height,
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Line::from(vec![
                Span::styled(" ⚠ ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    "Screen Size Too Small",
                    Style::default().fg(Color::Red).bold(),
                ),
                Span::raw(" "),
            ]))
            .title_alignment(Alignment::Center);

        let mut lines = Vec::new();
        if box_height >= 7 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![
            Span::raw("Minimum required: "),
            Span::styled(
                format!("{min_w} x {min_h}"),
                Style::default().fg(Color::Green).bold(),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Current size:     "),
            Span::styled(
                format!("{cur_w} x {cur_h}"),
                Style::default().fg(Color::Red).bold(),
            ),
        ]));
        if box_height >= 8 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Please enlarge the terminal window to continue.",
                Style::default().fg(Color::Yellow),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, popup_area);
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
