pub(crate) mod cursor;
pub(crate) mod edit;
pub(crate) mod goto;
pub(crate) mod text;

use domain::error::Result;
use editor_core::command::Command;

use crate::app::App;

impl App {
    pub fn execute(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::Session(session_command) => self.handle_session_command(session_command),
            Command::Workspace(navigation_command) => {
                self.handle_navigation_command(navigation_command)
            }
            Command::Motion(motion) => self.handle_motion_command(motion),
            Command::Mode(mode_command) => self.handle_mode_command(mode_command)?,
            Command::Layout(layout_command) => self.handle_layout_command(layout_command),
            Command::Edit(edit_command) => self.handle_edit_command(edit_command)?,
            Command::CommandLine(command_line_command) => {
                self.handle_command_line_command(command_line_command)
            }
        }
        Ok(())
    }

    pub fn execute_command(&mut self, _input: &str) -> Result<()> {
        // TODO: Parse the input to a command and execute that command
        todo!()
    }
}
