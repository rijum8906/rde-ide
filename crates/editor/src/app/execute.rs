use domain::error::Result;
use editor_core::{command::command::Command, input::mode::Mode};

use crate::app::App;

impl App {
    pub fn execute(&mut self, cmd: Command) -> Result<()> {
        if cmd.should_quit() {
            self.is_running = false;
            return Ok(());
        }

        match cmd {
            Command::EnterMode(mode) => {
                self.set_mode(mode);
            }
            Command::CancelCommandLine | Command::ExitToNormal => {
                self.set_mode(Mode::Normal);
            }
            _ => {
                // Other commands: implement as you go.
            }
        }
        Ok(())
    }

    pub fn execute_command(&mut self, _input: &str) -> Result<()> {
        // TODO: Parse the input to a command and execute that command
        todo!()
    }
}
