use tui::{layout::Layout, widgets::Widgets};

#[derive(Debug, Default)]
pub struct Ui {
    pub layout: Layout,
    pub widgets: Widgets,
}
