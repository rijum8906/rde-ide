use editor_core::command::LayoutCommand;

use crate::app::App;

impl App {
    pub fn handle_layout_command(&mut self, cmd: LayoutCommand) {
        match cmd {
            LayoutCommand::ToggleFullscreen => todo!(),
            LayoutCommand::ToggleLeftPanel => {
                self.ui.widgets.left_panel.is_open = !self.ui.widgets.left_panel.is_open
            }
            LayoutCommand::ToggleRightPanel => todo!(),
            LayoutCommand::FocusLeftPanel => todo!(),
            LayoutCommand::FocusRightPanel => todo!(),
        }
    }
}
