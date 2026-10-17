use editor_core::command::command::NavigationCommand;

use crate::app::App;

impl App {
    pub fn handle_navigation_command(&mut self, cmd: NavigationCommand) {
        match cmd {
            NavigationCommand::NextTab => todo!(),
            NavigationCommand::PrevTab => todo!(),
            NavigationCommand::SelectTab(_) => todo!(),
            NavigationCommand::CloseTab => todo!(),
            NavigationCommand::CloseAllTabs => todo!(),
            NavigationCommand::NextBuffer => todo!(),
            NavigationCommand::PrevBuffer => todo!(),
            NavigationCommand::FocusEditor => todo!(),
            NavigationCommand::FocusLeftPanel => todo!(),
            NavigationCommand::FocusRightPanel => todo!(),
            NavigationCommand::FocusNextPane => todo!(),
        }
    }
}
