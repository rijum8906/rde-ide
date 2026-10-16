#[derive(Debug, Default)]
pub struct LayoutStates {
    pub left_panel_open: bool,
    pub right_panel_open: bool,
    pub mode: LayoutMode,
}

#[derive(Debug, Default)]
pub enum LayoutMode {
    #[default]
    Normal,
    FullScreen,
}
