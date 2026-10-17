pub mod builder;
pub mod execute;
pub mod render;

use std::time::Duration;

use crossterm::event::{self, Event};
use domain::{error::Result, workspace::WorkspaceState};
use editor_core::{
    command::command::Command,
    input::{
        key::Key,
        keymap::Keymap,
        mode::Mode,
        resolver::{ResolveResult, Resolver},
    },
};
use platform::events::to_key;
use ratatui::DefaultTerminal;
use tui::{
    layout::LayoutStates,
    widgets::{Size, WidgetGeometry, WidgetStates},
};

pub struct App {
    // ============================================================
    // Domain — the documents being edited
    // ============================================================
    pub workspace: WorkspaceState,

    // ============================================================
    // Editor policy — how input is interpreted
    // ============================================================
    pub mode: Mode,
    pub key_map: Keymap,
    pub resolver: Resolver,

    // ============================================================
    // Layout policy — what the user wants to see
    // ============================================================
    pub layout_states: LayoutStates,

    // ============================================================
    // Widget state — persistent per-widget data across frames
    // ============================================================
    pub widget_geometry: WidgetGeometry,
    pub widget_states: WidgetStates,

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
                "/home/rijum/Projects/rde-ide/apps/ide/src/main.rs",
            )?,
            mode: Mode::Normal,
            key_map: Keymap::default(),
            resolver: Resolver::new(),
            layout_states: LayoutStates::default(),
            widget_geometry: WidgetGeometry::default(),
            widget_states: WidgetStates::default(),
            is_running: false,
        })
    }

    /// Runs the main application loop, drawing the UI and handling events non-blockingly.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.is_running = true;

        if let Ok(size) = terminal.size() {
            self.widget_geometry = WidgetGeometry::compute_with_layout(
                Size {
                    height: size.height,
                    width: size.width,
                },
                &self.layout_states,
            );
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
                        self.widget_geometry = WidgetGeometry::compute_with_layout(
                            Size { width, height },
                            &self.layout_states,
                        );
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
    pub fn handle_key(&mut self, key: Key) -> Result<()> {
        match self.resolver.feed(self.mode, key, &self.key_map) {
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
        if let Some(cmd) = self.resolver.poll_timeout(std::time::Instant::now()) {
            self.execute(cmd)?;
        }
        Ok(())
    }

    /// Fall-through for keys not bound in the keymap.
    fn handle_unhandled(&mut self, keys: editor_core::input::resolver::Pending) -> Result<()> {
        for key in keys {
            if let editor_core::input::key::KeyCode::Char(c) = key.code {
                if !key
                    .modifiers
                    .intersects(crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT)
                {
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

    // ===========================================================
    // Setter and Getter
    // ===========================================================

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_core::input::key::{KeyCode, KeyExt};

    #[test]
    fn test_app_should_quit() {
        let mut app = App::new().unwrap();
        // Initially, is_running is false, so should_quit is true
        assert!(app.should_quit());

        // When running, should_quit is false
        app.is_running = true;
        assert!(!app.should_quit());

        // Quitting sets is_running to false, so should_quit becomes true
        app.execute(Command::Quit).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_escape_quits() {
        let mut app = App::new().unwrap();
        app.is_running = true;
        app.handle_key(Key::plain(KeyCode::Esc)).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_ctrl_q_force_quits() {
        let mut app = App::new().unwrap();
        app.is_running = true;
        app.handle_key(Key::ctrl('q')).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_ctrl_c_force_quits() {
        let mut app = App::new().unwrap();
        app.is_running = true;
        app.handle_key(Key::ctrl('c')).unwrap();
        assert!(!app.is_running);
        assert!(app.should_quit());
    }
}
