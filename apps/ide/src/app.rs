//! Application shell: owns runtime state and drives the event loop.
//!
//! [`App`] ties the three layers together:
//!
//! - the domain ([`WorkspaceState`]) holding the open buffers,
//! - the input policy ([`Input`]) turning keys into commands,
//! - the UI state ([`Ui`]) holding layout and widget state.
//!
//! Each frame it polls terminal events, resolves keys into [`Command`]s,
//! dispatches them through [`editor::execute::execute`], and applies the
//! resulting [`Effect`].

use std::time::{Duration, Instant};

use domain::{error::Result, workspace::WorkspaceState};
use editor::{
    execute::{Effect, execute},
    notification::Notifications,
};
use editor_core::{
    command::{Command, CommandLineCommand, EditCommand::InsertChar},
    input::{
        Input,
        key::{Key, Mods},
        mode::InputMode,
        resolver::ResolveResult,
    },
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
};
use tui::{
    geometry, render,
    ui::Ui,
    viewmodel::{notification::NotificationView, toast::ToastView},
};

pub struct App {
    // ============================================================
    // Domain — the documents being edited
    // ============================================================
    pub workspace: WorkspaceState,

    pub notifications: Notifications,

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
            notifications: Notifications::default(),
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
            geometry::calc_widgets_geometry(&mut self.ui);
        }

        while !self.should_quit() {
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        self.handle_key(Key::from(key_event))?;
                    }
                    Event::Resize(width, height) => {
                        self.ui.layout.screen_size = (height, width);
                        geometry::calc_widgets_geometry(&mut self.ui);
                    }
                    _ => {}
                }
            }

            if self.should_quit() {
                break;
            }

            self.tick()?;

            terminal.draw(|frame| self.render(frame))?;
        }

        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        !self.is_running
    }

    // ===========================================================
    // Rendering
    // ===========================================================

    /// Draw one frame: the layout first, then any visible notifications, and
    /// finally the toast.
    pub fn render(&mut self, frame: &mut Frame) {
        render::render_into(&mut self.workspace, &mut self.ui, &self.input, frame);

        // Notifications sit top-centre; their bottom row lets the toast start
        // below them instead of overlapping.
        let mut notification_bottom = 0;
        let visible = self.notifications.visible();
        if !visible.is_empty() {
            let views: Vec<NotificationView<'_>> =
                visible.iter().map(NotificationView::from).collect();
            notification_bottom = render::render_notifications(&views, frame).unwrap_or(0);
        }

        // The toast is the newest, most local message, so it is drawn last.
        if let Some(toast) = self.ui.widgets.toast.state.visible() {
            let view = ToastView::from(toast);
            render::render_toast(&view, frame, notification_bottom);
        }
    }

    // ===========================================================
    // Input Handling
    // ===========================================================

    /// Feed one key. Called by the binary's event loop.
    pub fn handle_key(&mut self, key: Key) -> Result<()> {
        match self
            .input
            .resolver
            .feed(self.input.mode, key, &self.input.key_map)
        {
            ResolveResult::Resolved(cmd) => self.dispatch(cmd)?,
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
        let now = Instant::now();

        // Command
        if let Some(cmd) = self.input.resolver.poll_timeout(now) {
            self.dispatch(cmd)?;
        }

        // Notifications
        self.notifications.tick(now);

        // Toasts
        self.ui.widgets.toast.state.poll_expiry(now);

        Ok(())
    }

    /// Fall-through for keys not bound in the keymap.
    fn handle_unhandled(&mut self, keys: editor_core::input::resolver::Pending) -> Result<()> {
        for key in keys {
            if let editor_core::input::key::KeyCode::Char(c) = key.code {
                if key.mods != Mods::CTRL && key.mods != Mods::ALT {
                    match self.input.mode {
                        InputMode::Insert => self.dispatch(Command::Edit(InsertChar(c)))?,
                        InputMode::Command => {
                            self.dispatch(Command::CommandLine(CommandLineCommand::Push(c)))?
                        }

                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// Dispatch a command and apply the side effect it reports.
    ///
    /// Lifecycle is owned here rather than in the editor crate, so quit
    /// commands come back as [`Effect::Quit`].
    fn dispatch(&mut self, cmd: Command) -> Result<()> {
        if execute(&mut self.workspace, &mut self.input, &mut self.ui, cmd)? == Effect::Quit {
            self.is_running = false;
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
