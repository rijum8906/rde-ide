use domain::workspace::WorkspaceState;
use editor_core::command::SessionCommand;

pub fn handle_session_command(ws: &mut WorkspaceState, cmd: SessionCommand) {
    match cmd {
        SessionCommand::Quit => {
            // TODO: look for dirty buffers and give option to save them
            // and more
        }
        SessionCommand::ForceQuit => todo!(),
        SessionCommand::SaveBuffer => {
            if let Some(buffer) = ws.active_buffer_mut() {
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
