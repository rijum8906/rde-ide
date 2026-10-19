//! Session lifecycle commands: quit, save, and open files.

use domain::workspace::WorkspaceState;
use editor_core::command::SessionCommand;

pub fn handle_session_command(ws: &mut WorkspaceState, cmd: SessionCommand) {
    match cmd {
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
        // NOTE: only Quit and ForceQuit is being ignored because it will be handled in the app level
        _ => {}
    }
}
