//! UI Module for IDE project.
//!
//! This module provides the user interface components and functionality for the IDE project.

use crossterm::event::{self, Event, KeyEvent};
use ide_core::error::Result;
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

pub struct UiState {
    // TODO: Add editor state here (e.g., current buffer, cursor position, mode)
    should_quit: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self { should_quit: false }
    }
}

pub struct Ui {
    state: UiState,
}

impl Ui {
    pub fn new() -> Result<Self> {
        // Only initialize color_eyre if not already initialized by the main binary
        let _ = color_eyre::install();
        Ok(Ui {
            state: UiState::new(),
        })
    }

    /// Runs the main application loop, drawing the UI and handling events non-blockingly.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.state.should_quit {
            terminal.draw(|frame| self.render(frame))?;

            // Poll for events with a timeout to allow frequent UI refreshes/animations
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_event(key),
                    Event::Resize(_, _) => {
                        // Ratatui handles resizing automatically via terminal.draw,
                        // but you can trigger state recomputations here if needed.
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        // Example: quit on Esc or Ctrl+C
        if key.code == crossterm::event::KeyCode::Esc {
            self.state.should_quit = true;
        }
    }

    fn render(&self, frame: &mut Frame) {
        // Render your editor components here (status bar, editor grid, sidebar, etc.)
        frame.render_widget("Neovim-like Editor - Press Esc to exit", frame.area());
    }
}
