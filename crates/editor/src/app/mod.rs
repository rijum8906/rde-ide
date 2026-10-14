pub mod execute;
pub mod ui;

use std::time::Duration;

use crossterm::event::{self, Event};
use domain::error::Result;
use editor_core::{
    command::command::Command,
    input::{
        key::Key,
        keymap::{Keymap, default_keymap},
        mode::Mode,
        resolver::{ResolveResult, Resolver},
    },
};
use platform::events::to_key;
use ratatui::DefaultTerminal;

use crate::app::ui::Ui;

pub struct App {
    // ========= UI =========
    ui: Ui,

    // ========= States ========
    is_running: bool,
    mode: Mode,

    // ========= Handlers ==========
    key_map: Keymap,
    resolver: Resolver,
}

impl App {
    pub fn new() -> Result<Self> {
        // Initialize the ui
        let ui = Ui::new()?;

        Ok(Self {
            ui,
            is_running: false,
            mode: Mode::Normal,
            key_map: default_keymap(),
            resolver: Resolver::new(),
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
                    Event::Key(key_event) => {
                        if let Some(key) = to_key(key_event) {
                            self.handle_key(key)?;
                        }
                    }
                    Event::Resize(width, height) => {
                        self.ui.handle_resize(width, height);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Feed one key. Called by the binary's event loop.
    pub fn handle_key(&mut self, key: Key) -> Result<()> {
        match self.resolver.feed(self.mode, key, &self.key_map) {
            ResolveResult::Resolved(cmd) => self.execute(cmd)?,
            ResolveResult::Pending => {}
            ResolveResult::Unhandled(keys) => self.handle_unhandled(keys)?,
            ResolveResult::Abandoned(_keys) => {
                // Optional: beep, or set a status message.
            }
        }
        Ok(())
    }

    /// Called once per frame, before feeding keys. Expires stale prefixes.
    pub fn tick(&mut self) -> Result<()> {
        if let Some(cmd) = self.resolver.poll_timeout(std::time::Instant::now()) {
            self.execute(cmd)?;
        }
        Ok(())
    }

    /// Fall-through for keys not bound in the keymap.
    fn handle_unhandled(&mut self, keys: editor_core::input::resolver::Pending) -> Result<()> {
        for key in keys {
            if let editor_core::input::key::KeyCode::Char(c) = key.code {
                if !key.modifiers.ctrl && !key.modifiers.alt {
                    match self.mode {
                        Mode::Insert => self.execute(Command::InsertChar(c))?,
                        Mode::Command => self.execute(Command::CommandLinePush(c))?,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
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
