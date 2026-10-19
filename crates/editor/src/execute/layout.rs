use domain::workspace::WorkspaceState;
use editor_core::command::LayoutCommand;
use tui::ui::Ui;

pub fn handle_layout_command(_ws: &mut WorkspaceState, ui: &mut Ui, cmd: LayoutCommand) {
    match cmd {
        LayoutCommand::ToggleFullscreen => todo!(),
        LayoutCommand::ToggleLeftPanel => {
            ui.widgets.left_panel.is_open = !ui.widgets.left_panel.is_open
        }
        LayoutCommand::ToggleRightPanel => todo!(),
        LayoutCommand::FocusLeftPanel => todo!(),
        LayoutCommand::FocusRightPanel => todo!(),
    }
}
