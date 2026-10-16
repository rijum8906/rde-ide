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
            Command::ToggleLeftPanel => {
                self.layout_states.left_panel_open = !self.layout_states.left_panel_open;
            }
            Command::MoveCursor(direction) => match direction {
                editor_core::command::command::Direction::Up => {
                    if self.widget_states.editor_panel.cursor_line != 0 {
                        self.widget_states.editor_panel.move_up(1);
                    }
                }
                editor_core::command::command::Direction::Down => {
                    self.widget_states.editor_panel.move_down(1);
                }

                editor_core::command::command::Direction::Left => {
                    self.widget_states.editor_panel.move_left();
                }
                editor_core::command::command::Direction::Right => {
                    self.widget_states.editor_panel.move_right(1);
                }
            },
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
