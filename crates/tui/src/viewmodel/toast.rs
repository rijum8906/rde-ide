//! View model for the toast widget.

use crate::viewmodel::level::Level;

/// Everything the toast widget needs to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToastView<'a> {
    pub level: Level,
    /// Bold title line. When empty the level name is used instead.
    pub header: &'a str,
    /// Body text. Wrapped and clamped to two lines.
    pub body: &'a str,
}
