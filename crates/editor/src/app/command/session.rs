use editor_core::command::command::SessionCommand;

use crate::app::App;

impl App {
    pub fn handle_session_command(&mut self, cmd: SessionCommand) {
        match cmd {
            SessionCommand::Quit => {
                // TODO: look for dirty buffers and give option to save them
                // and more
                self.is_running = false;
            }
            SessionCommand::ForceQuit => self.is_running = false,
            SessionCommand::SaveBuffer => {
                if let Some(buffer) = self.workspace.active_buffer_mut() {
                    let _ = buffer.save();
                }
            }
            SessionCommand::SaveBufferAs(_) => todo!(),
            SessionCommand::SaveAll => todo!(),
            SessionCommand::OpenFile(_) => todo!(),
            SessionCommand::OpenFileTree => todo!(),
            SessionCommand::NewScratchBuffer => todo!(),
        }
    }
}
