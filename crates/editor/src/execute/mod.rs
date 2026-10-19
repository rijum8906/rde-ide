//! Command execution.
//!
//! [`execute`] is the single entry point: it routes each [`Command`] category
//! to its handler and hands over only the state that category is allowed to
//! touch. Handlers are free functions rather than methods so the app shell can
//! own the lifecycle without the editor crate owning the app.
//!
//! Handlers cannot stop the application themselves, so dispatch returns an
//! [`Effect`] that the caller (the app loop) applies. Quit commands are
//! recognised here and reported as [`Effect::Quit`].

use domain::{error::Result, workspace::WorkspaceState};
use editor_core::{
    command::{Command, SessionCommand},
    input::Input,
};
use tui::ui::Ui;

use crate::execute::{
    command_line::handle_command_line_command, edit::handle_edit_command,
    layout::handle_layout_command, mode::handle_mode_command, motion::handle_motion_command,
    navigation::handle_navigation_command, session::handle_session_command,
};

pub(super) mod command_line;
pub(super) mod edit;
pub(super) mod layout;
pub(super) mod mode;
pub(super) mod motion;
pub(super) mod navigation;
pub(super) mod session;

/// Side effect the caller must apply after a command runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Nothing beyond the state changes the handler already made.
    None,
    /// The application should shut down.
    Quit,
}

/// Execute one command against the editor state.
///
/// Returns the [`Effect`] the caller should apply; the app loop turns
/// `Effect::Quit` into "stop running".
pub fn execute(
    ws: &mut WorkspaceState,
    input: &mut Input,
    ui: &mut Ui,
    cmd: Command,
) -> Result<Effect> {
    let effect = match cmd {
        // Lifecycle is owned by the app, so quit commands never reach a
        // handler: they are reported straight back to the caller.
        Command::Session(SessionCommand::Quit | SessionCommand::ForceQuit) => Effect::Quit,

        Command::Session(session_command) => {
            handle_session_command(ws, session_command);
            Effect::None
        }

        Command::Workspace(workspace_command) => {
            handle_navigation_command(ws, ui, workspace_command);
            Effect::None
        }

        Command::Motion(motion) => {
            handle_motion_command(ws, ui, motion);
            Effect::None
        }

        Command::Mode(mode_command) => {
            handle_mode_command(ws, input, ui, mode_command)?;
            Effect::None
        }

        Command::Layout(layout_command) => {
            handle_layout_command(ws, ui, layout_command);
            Effect::None
        }

        Command::Edit(edit_command) => {
            handle_edit_command(ws, ui, edit_command)?;
            Effect::None
        }

        Command::CommandLine(command_line_command) => {
            handle_command_line_command(ws, input, ui, command_line_command)?
        }
    };

    Ok(effect)
}
