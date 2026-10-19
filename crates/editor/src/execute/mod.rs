use domain::{error::Result, workspace::WorkspaceState};
use editor_core::{command::Command, input::Input};
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

pub fn execute(
    ws: &mut WorkspaceState,
    input: &mut Input,
    ui: &mut Ui,
    cmd: Command,
) -> Result<()> {
    match cmd {
        Command::Session(session_command) => handle_session_command(ws, session_command),
        Command::Workspace(navigation_command) => {
            handle_navigation_command(ws, ui, navigation_command)
        }
        Command::Motion(motion) => handle_motion_command(ws, ui, motion),
        Command::Mode(mode_command) => handle_mode_command(ws, input, ui, mode_command)?,
        Command::Layout(layout_command) => handle_layout_command(ws, ui, layout_command),
        Command::Edit(edit_command) => handle_edit_command(ws, ui, edit_command)?,
        Command::CommandLine(command_line_command) => {
            handle_command_line_command(command_line_command)
        }
    }
    Ok(())
}
