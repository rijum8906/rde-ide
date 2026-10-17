use editor_core::command::command::LayoutCommand;

use crate::app::App;

impl App {
    pub fn handle_layout_command(&mut self, cmd: LayoutCommand) {
        match cmd {
            LayoutCommand::ToggleFullscreen => todo!(),
            LayoutCommand::ToggleLeftPanel => {
                self.layout_states.left_panel_open = !self.layout_states.left_panel_open
            }
            LayoutCommand::ToggleRightPanel => todo!(),
            LayoutCommand::FocusLeftPanel => todo!(),
            LayoutCommand::FocusRightPanel => todo!(),
        }
    }
}
