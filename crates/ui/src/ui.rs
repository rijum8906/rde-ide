//! UI Module for IDE project.
//!
//! This module provides the user interface components and functionality for the IDE project.

use crossterm::event::{self, Event, KeyEvent};
use ide_core::error::Result;
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

use crate::state::ui::UiState;

pub struct Ui {
    state: UiState,
    is_running: bool,
}

impl Ui {
    pub fn new() -> Result<Self> {
        // Only initialize color_eyre if not already initialized by the main binary
        let _ = color_eyre::install();

        // Query initial terminal size for layout and validation
        let (width, height) = crossterm::terminal::size()?;

        // Initialize UiState with dimensions
        let state = UiState::new(height, width)?;

        Ok(Ui {
            state,
            is_running: false,
        })
    }

    /// Runs the main application loop, drawing the UI and handling events non-blockingly.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.is_running {
            terminal.draw(|frame| self.render(frame))?;

            // Poll for events with a timeout to allow frequent UI refreshes/animations
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_event(key),
                    Event::Resize(_width, _height) => {
                        // TODO: Dynamically update layout/widget states on terminal resize
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        // Example: quit on Esc
        if key.code == crossterm::event::KeyCode::Esc {
            self.is_running = false;
        }
    }

    fn render(&self, frame: &mut Frame) {
        // Render components using layout state
        let area = frame.area();
        let layout_state = self.state.layout_state();

        frame.render_widget(
            format!(
                "RDE IDE | Mode: {:?} | Press Esc to exit",
                layout_state.layout_mode()
            ),
            area,
        );
    }
}
