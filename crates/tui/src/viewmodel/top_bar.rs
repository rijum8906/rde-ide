//! Top bar view model: which tabs to show and which is active.

/// One tab's display data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabView {
    pub title: String,
    pub is_active: bool,
    pub is_dirty: bool,
}

/// Everything the top bar needs to render.
#[derive(Debug)]
pub struct TopBarView {
    pub tabs: Vec<TabView>,
}
