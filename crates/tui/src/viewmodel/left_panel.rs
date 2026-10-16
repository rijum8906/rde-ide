//! Side-panel view model: a flat list of visible rows.

/// One visible row in the panel.
#[derive(Debug, Clone)]
pub struct RowView {
    /// Display name (file or directory basename).
    pub name: String,
    /// Indentation depth; 0 for top-level, 1 for children, etc.
    pub depth: usize,
    /// True if this row is a directory.
    pub is_dir: bool,
    /// True if this row is an expanded directory.
    /// Only meaningful when `is_dir` is true.
    pub is_expanded: bool,
    /// True if this row is the currently selected row.
    pub is_selected: bool,
}

/// Everything the side panel needs to render.
#[derive(Debug, Clone)]
pub struct LeftPanelView {
    pub rows: Vec<RowView>,
    /// Title to display at the top of the panel (e.g. root directory name).
    pub title: String,
    /// True while a scan is in progress and no tree is available yet.
    pub is_loading: bool,
    /// Optional message shown when the panel is empty.
    pub empty_message: Option<String>,
}

impl LeftPanelView {
    /// Construct an empty view (used while loading, or on scan failure).
    pub fn empty(title: impl Into<String>) -> Self {
        Self {
            rows: Vec::new(),
            title: title.into(),
            is_loading: false,
            empty_message: None,
        }
    }

    pub fn loading(title: impl Into<String>) -> Self {
        Self {
            rows: Vec::new(),
            title: title.into(),
            is_loading: true,
            empty_message: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
