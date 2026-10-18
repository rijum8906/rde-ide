//! Editor modes. Model all planned modes now, even if only `Normal` is
//! implemented.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    Visual,
    VisualLine,
    Command, // ":wq" style
}

impl InputMode {
    /// Returns uppercase string representation used in status line badges.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::VisualLine => "VISUAL LINE",
            Self::Command => "COMMAND",
        }
    }

    /// Backwards-compatible alias for `as_str`.
    pub fn to_str(&self) -> &'static str {
        self.as_str()
    }

    /// Whether typing a printable character should insert it.
    pub fn is_text_entry(self) -> bool {
        matches!(self, Self::Insert | Self::Command)
    }
}

impl fmt::Display for InputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display_and_as_str() {
        assert_eq!(InputMode::Normal.as_str(), "NORMAL");
        assert_eq!(InputMode::Insert.as_str(), "INSERT");
        assert_eq!(InputMode::Visual.as_str(), "VISUAL");
        assert_eq!(InputMode::VisualLine.as_str(), "VISUAL LINE");
        assert_eq!(InputMode::Command.as_str(), "COMMAND");
        assert_eq!(format!("{}", InputMode::Insert), "INSERT");
    }
}
