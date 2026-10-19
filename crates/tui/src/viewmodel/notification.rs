//! View model for floating notifications.
//!
//! The app owns the notification data (`editor::notification::Notifications`);
//! this is the per-frame snapshot the widget draws. It borrows the strings so
//! showing a notification copies nothing.

pub use crate::viewmodel::level::Level as NotificationLevel;

/// One notification card: a header line and a body paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationView<'a> {
    pub level: NotificationLevel,
    /// Bold title line. When empty the level name is used instead.
    pub header: &'a str,
    /// Body text. Wrapped and clamped to the card; may be empty.
    pub body: &'a str,
}
