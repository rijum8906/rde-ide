use domain::error::Result;
use editor_core::{command::command::Command, input::mode::Mode};

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
            Command::ExitToNormal => {
                self.set_mode(Mode::Normal);
            }
            _ => {
                // Other commands: implement as you go.
            }
        }
        Ok(())
    }
}
