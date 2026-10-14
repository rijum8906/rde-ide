use domain::error::Result;
use editor_core::command::command::Command;

use crate::app::App;

impl App {
    pub fn execute(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::Quit | Command::ForceQuit => {
                self.is_running = false;
            }

            Command::EnterMode(mode) => {
                self.set_mode(mode);
            }
            _ => {
                // Other commands: implement as you go.
            }
        }
        Ok(())
    }
}
