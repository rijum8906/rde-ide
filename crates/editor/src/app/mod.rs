pub mod ui;

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use domain::error::Result;
use ratatui::DefaultTerminal;

use crate::{app::ui::Ui, input::mode::Mode};

pub struct App {
    // ========= UI =========
    ui: Ui,

    // ========= States ========
    is_running: bool,
    mode: Mode,
}

impl App {
    pub fn new() -> Result<Self> {
        // Initialize the ui
        let ui = Ui::new()?;

        Ok(Self {
            ui,
            is_running: false,
            mode: Mode::Normal,
        })
    }

    /// Runs the main application loop, drawing the UI and handling events non-blockingly.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.is_running = true;

        while self.is_running {
            self.ui.draw(terminal)?;

            // Poll for events with a timeout to allow frequent UI refreshes/animations
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_event(key),
                    Event::Resize(width, height) => {
                        self.ui.handle_resize(width, height);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
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
}

impl App {
    pub fn ui(&self) -> &Ui {
        &self.ui
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}
