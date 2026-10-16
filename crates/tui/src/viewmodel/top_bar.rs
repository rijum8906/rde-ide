//! Top bar view model: which tabs to show and which is active.

/// One tab's display data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabView<'a> {
    pub title: &'a str,
    pub is_active: bool,
    pub is_dirty: bool,
}

/// Everything the top bar needs to render.
#[derive(Debug, Clone)]
pub struct TopBarView<'a> {
    pub tabs: Vec<TabView<'a>>,
}
