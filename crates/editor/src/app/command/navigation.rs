use editor_core::command::WorkspaceCommand;

use crate::app::App;

impl App {
    pub fn handle_navigation_command(&mut self, cmd: WorkspaceCommand) {
        match cmd {
            WorkspaceCommand::NextBuffer => todo!(),
            WorkspaceCommand::PrevBuffer => todo!(),
            WorkspaceCommand::CloseBuffer => todo!(),
            WorkspaceCommand::CloseBufferForce => todo!(),
            WorkspaceCommand::SwitchBuffer(_) => todo!(),
            WorkspaceCommand::FocusEditorPanel => todo!(),
            WorkspaceCommand::FocusLeftPanel => todo!(),
            WorkspaceCommand::FocusRightPanel => todo!(),
            WorkspaceCommand::ToggleFileTree => todo!(),
        }
    }
}
