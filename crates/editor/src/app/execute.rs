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
                    if let Some(buffer) = self.workspace.active_buffer() {
                        if self.widget_states.editor_panel.cursor_line > 0 {
                            self.widget_states
                                .editor_panel
                                .move_up(buffer.line_char_len(
                                    self.widget_states.editor_panel.cursor_line - 1,
                                ));
                        }
                    }
                }
                editor_core::command::command::Direction::Down => {
                    if let Some(buffer) = self.workspace.active_buffer() {
                        if self.widget_states.editor_panel.cursor_line < buffer.line_count() {
                            self.widget_states
                                .editor_panel
                                .move_down(buffer.line_char_len(
                                    self.widget_states.editor_panel.cursor_line + 1,
                                ));
                        }
                    }
                }

                editor_core::command::command::Direction::Left => {
                    self.widget_states.editor_panel.move_left();
                }
                editor_core::command::command::Direction::Right => {
                    let line_len = self
                        .workspace
                        .active_buffer()
                        .unwrap()
                        .line_char_len(self.widget_states.editor_panel.cursor_line);
                    self.widget_states.editor_panel.move_right(line_len);
                }
            },
            Command::MoveToFileStart => {
                self.widget_states.editor_panel.move_to_file_start();
            }
            Command::MoveToFileEnd => {
                if let Some(buffer) = self.workspace.active_buffer() {
                    self.widget_states
                        .editor_panel
                        .move_to_file_end(buffer.line_char_len(buffer.line_count() - 1));
                }
            }
            Command::MoveToLineStart => {
                self.widget_states.editor_panel.move_home();
            }
            Command::MoveToLineEnd => {
                if let Some(buffer) = self.workspace.active_buffer() {
                    self.widget_states.editor_panel.move_end(
                        buffer.line_char_len(self.widget_states.editor_panel.cursor_line),
                    );
                }
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
