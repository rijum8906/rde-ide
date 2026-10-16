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
    pub fn to_str(&self) -> &'static str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Visual => "Visual",
            Mode::VisualLine => "VisualLine",
            Mode::Command => "Command",
        }
    }
}

impl Mode {
    /// Whether typing a printable character should insert it.
    pub fn is_text_entry(self) -> bool {
        matches!(self, Mode::Insert | Mode::Command)
    }
}
