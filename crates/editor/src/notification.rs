//! Transient notifications shown over the UI.
//!
//! The app pushes a [`Notification`] when something worth telling the user
//! about happens (a save, a failed command, ...). [`Notifications`] keeps the
//! most recent few, expires them after their TTL, and is polled once per frame
//! by [`tick`](Notifications::tick).
//!
//! Rendering lives in the TUI crate; this module only owns the data and the
//! conversion into the widget's view model.

use std::time::{Duration, Instant};

use tui::viewmodel::notification::{NotificationLevel, NotificationView};

/// How long a notification stays on screen unless it is given its own TTL.
pub const DEFAULT_TTL: Duration = Duration::from_secs(5);

/// How many notifications may be visible at once.
pub const DEFAULT_CAPACITY: usize = 4;

/// Severity of a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Warning,
    Success,
    Error,
    Panic,
}

/// Title and body of a notification.
#[derive(Debug, Clone)]
pub struct NotificationBody {
    pub header: String,
    pub body: String,
}

impl NotificationBody {
    /// Build a body with both a header and body text.
    pub fn new(header: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            body: body.into(),
        }
    }

    /// Build a body with only text; the header falls back to the level name.
    pub fn body(text: impl Into<String>) -> Self {
        Self {
            header: String::new(),
            body: text.into(),
        }
    }
}

/// One notification: what to show and how long to show it.
#[derive(Debug, Clone)]
pub struct Notification {
    pub level: Level,
    pub body: NotificationBody,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl Notification {
    pub fn new(level: Level, body: NotificationBody, ttl: Duration) -> Self {
        Self {
            level,
            body,
            created_at: Instant::now(),
            ttl,
        }
    }

    pub fn info(body: NotificationBody) -> Self {
        Self::new(Level::Info, body, DEFAULT_TTL)
    }

    pub fn warning(body: NotificationBody) -> Self {
        Self::new(Level::Warning, body, DEFAULT_TTL)
    }

    pub fn success(body: NotificationBody) -> Self {
        Self::new(Level::Success, body, DEFAULT_TTL)
    }

    pub fn error(body: NotificationBody) -> Self {
        Self::new(Level::Error, body, DEFAULT_TTL)
    }

    pub fn panic(body: NotificationBody) -> Self {
        // Panics are important; leave them up longer.
        Self::new(Level::Panic, body, Duration::from_secs(10))
    }

    /// Whether this notification should still be on screen at `now`.
    pub fn is_expired(&self, now: Instant) -> bool {
        !self.ttl.is_zero() && now.duration_since(self.created_at) >= self.ttl
    }
}

/// The notifications currently on screen, newest last.
#[derive(Debug)]
pub struct Notifications {
    items: Vec<Notification>,
    /// Max simultaneous visible; older ones evicted.
    capacity: usize,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            capacity: DEFAULT_CAPACITY,
        }
    }
}

impl Notifications {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity,
        }
    }

    /// Show a notification, evicting the oldest when full.
    pub fn push(&mut self, n: Notification) {
        if self.capacity == 0 {
            return;
        }
        if self.items.len() >= self.capacity {
            self.items.remove(0);
        }
        self.items.push(n);
    }

    /// Drop expired ones. Called once per frame by the app.
    pub fn tick(&mut self, now: Instant) {
        self.items.retain(|n| !n.is_expired(now));
    }

    /// The notifications that should be drawn, oldest first.
    pub fn visible(&self) -> &[Notification] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn dismiss_all(&mut self) {
        self.items.clear();
    }
}

impl From<Level> for NotificationLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::Info => Self::Info,
            Level::Warning => Self::Warning,
            Level::Success => Self::Success,
            Level::Error => Self::Error,
            Level::Panic => Self::Panic,
        }
    }
}

impl<'a> From<&'a Notification> for NotificationView<'a> {
    fn from(notification: &'a Notification) -> Self {
        Self {
            level: notification.level.into(),
            header: &notification.body.header,
            body: &notification.body.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification(header: &str) -> Notification {
        Notification::info(NotificationBody::new(header, "body"))
    }

    #[test]
    fn default_capacity_keeps_notifications_visible() {
        let mut notifications = Notifications::default();
        notifications.push(notification("one"));

        assert!(!notifications.is_empty());
        assert_eq!(notifications.visible().len(), 1);
    }

    #[test]
    fn push_evicts_the_oldest_when_full() {
        let mut notifications = Notifications::with_capacity(2);
        notifications.push(notification("one"));
        notifications.push(notification("two"));
        notifications.push(notification("three"));

        let headers: Vec<&str> = notifications
            .visible()
            .iter()
            .map(|n| n.body.header.as_str())
            .collect();
        assert_eq!(headers, ["two", "three"]);
    }

    #[test]
    fn tick_drops_expired_notifications() {
        let mut notifications = Notifications::default();
        notifications.push(notification("old"));
        notifications.push(notification("new"));

        let now = Instant::now();
        notifications.items[0].created_at = now - Duration::from_secs(60);

        notifications.tick(now);

        let headers: Vec<&str> = notifications
            .visible()
            .iter()
            .map(|n| n.body.header.as_str())
            .collect();
        assert_eq!(headers, ["new"]);
    }

    #[test]
    fn zero_ttl_notifications_never_expire() {
        let n = Notification::new(
            Level::Info,
            NotificationBody::new("sticky", "stays"),
            Duration::ZERO,
        );

        assert!(!n.is_expired(Instant::now() + Duration::from_secs(3600)));
    }

    #[test]
    fn converts_to_the_widget_view() {
        let n = Notification::error(NotificationBody::new("Oops", "disk full"));
        let view = NotificationView::from(&n);

        assert_eq!(view.level, NotificationLevel::Error);
        assert_eq!(view.header, "Oops");
        assert_eq!(view.body, "disk full");
    }
}
