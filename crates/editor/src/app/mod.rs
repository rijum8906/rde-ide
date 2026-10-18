pub mod builder;
pub mod command;
pub mod execute;
pub mod geometry;
pub mod render;

use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};
use domain::{error::Result, workspace::WorkspaceState};
use editor_core::{
    command::{Command, CommandLineCommand::Push, EditCommand::InsertChar},
    input::{Input, mode::InputMode, resolver::ResolveResult},
    ui::Ui,
};
use platform::events::to_key;
use ratatui::DefaultTerminal;

pub struct App {
    // ============================================================
    // Domain — the documents being edited
    // ============================================================
    pub workspace: WorkspaceState,

    // ============================================================
    // Editor policy — how input is interpreted
    // ============================================================
    input: Input,

    // ============================================================
    // Layout policy — what the user wants to see
    // ============================================================
    ui: Ui,

    // ============================================================
    // Lifecycle
    // ============================================================
    pub is_running: bool,
}

impl App {
    // ===========================================================
    // Core Lifestyle
    // ===========================================================
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: WorkspaceState::from_path(
                "/home/rijum/Projects/rde-ide/apps/ide/Cargo.toml",
            )?,
            input: Input::default(),
            ui: Ui::default(),
            is_running: false,
        })
    }

    /// Runs the main application loop, drawing the UI and handling events non-blockingly.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.is_running = true;

        if let Ok(size) = terminal.size() {
            self.ui.layout.screen_size = (size.height, size.width);
            self.calc_widgets_geometry();
        }

        while !self.should_quit() {
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(ke) => {
                        if let Some(key) = to_key(ke) {
                            self.handle_key(key)?;
                        }
                    }
                    Event::Resize(width, height) => {
                        self.ui.layout.screen_size = (height, width);
                        self.calc_widgets_geometry();
                    }
                    _ => {}
                }
            }

            if self.should_quit() {
                break;
            }

            self.tick()?;

            terminal.draw(|frame| self.render_into(frame))?;
        }

        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        !self.is_running
    }

    // ===========================================================
    // Input Handling
    // ===========================================================

    /// Feed one key. Called by the binary's event loop.
    pub fn handle_key(&mut self, key_event: KeyEvent) -> Result<()> {
        match self
            .input
            .resolver
            .feed(self.input.mode, key_event, &self.input.key_map)
        {
            ResolveResult::Resolved(cmd) => self.execute(cmd)?,
            ResolveResult::Pending => {}
            ResolveResult::Unhandled(keys) => self.handle_unhandled(keys)?,
            ResolveResult::Abandoned(_keys) => {
                // TODO: beep, or set a status message.
            }
        }
        Ok(())
    }

    /// Called once per frame, before feeding keys. Expires stale prefixes.
    pub fn tick(&mut self) -> Result<()> {
        if let Some(cmd) = self.input.resolver.poll_timeout(std::time::Instant::now()) {
            self.execute(cmd)?;
        }
        Ok(())
    }

    /// Fall-through for keys not bound in the keymap.
    fn handle_unhandled(&mut self, keys: editor_core::input::resolver::Pending) -> Result<()> {
        for key in keys {
            if let editor_core::input::key::KeyCode::Char(c) = key.code {
                if !key.modifiers.intersects(
                    crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT,
                ) {
                    match self.input.mode {
                        InputMode::Insert => self.execute(Command::Edit(InsertChar(c)))?,
                        InputMode::Command => self.execute(Command::CommandLine(Push(c)))?,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    // ===========================================================
    // Setter and Getter
    // ===========================================================

    pub fn set_mode(&mut self, mode: InputMode) {
        self.input.mode = mode;
    }

    pub fn mode(&self) -> InputMode {
        self.input.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_core::{
        command::SessionCommand::Quit,
        input::key::{KeyCode, KeyExt},
    };

    #[test]
    fn test_app_should_quit() {
        let mut app = App::new().unwrap();
        // Initially, is_running is false, so should_quit is true
        assert!(app.should_quit());

        // When running, should_quit is false
        app.is_running = true;
        assert!(!app.should_quit());

        // Quitting sets is_running to false, so should_quit becomes true
        app.execute(Command::Session(Quit)).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_escape_quits() {
        let mut app = App::new().unwrap();
        app.is_running = true;
        app.handle_key(KeyEvent::plain(KeyCode::Esc)).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_ctrl_q_force_quits() {
        let mut app = App::new().unwrap();
        app.is_running = true;
        app.handle_key(KeyEvent::ctrl('q')).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_ctrl_c_force_quits() {
        let mut app = App::new().unwrap();
        app.is_running = true;
        app.handle_key(KeyEvent::ctrl('c')).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }

    #[test]
    fn test_insert_ctrl_j_enters_normal_mode() {
        let mut app = App::new().unwrap();
        app.is_running = true;
        app.input.mode = InputMode::Insert;
        app.handle_key(KeyEvent::ctrl('j')).unwrap();
        assert_eq!(app.input.mode, InputMode::Normal);
    }
}
