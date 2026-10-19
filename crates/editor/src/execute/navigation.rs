//! Workspace commands: buffer switching, closing, and panel focus.

use domain::workspace::WorkspaceState;
use editor_core::command::WorkspaceCommand;
use tui::ui::Ui;

pub fn handle_navigation_command(_ws: &mut WorkspaceState, _ui: &mut Ui, cmd: WorkspaceCommand) {
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
