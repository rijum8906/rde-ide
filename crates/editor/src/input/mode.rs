//! Editor modes. Model all planned modes now, even if only `Normal` is
//! implemented.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
    VisualLine,
    Command, // ":wq" style
}

impl Mode {
    /// Whether typing a printable character should insert it.
    pub fn is_text_entry(self) -> bool {
        matches!(self, Mode::Insert | Mode::Command)
    }
}
