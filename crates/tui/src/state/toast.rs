//! Persistent state for the toast widget.
//!
//! A toast is a single transient message, usually the result of a command. It
//! replaces whatever is currently shown and expires on its own; the app polls
//! [`ToastState::poll_expiry`] once per frame.
//!
//! This is the replacement for the status-line messages the bottom bar used to
//! own: command handlers call [`ToastState::error`] and friends, and the app
//! renders the result.

use std::time::{Duration, Instant};

use crate::viewmodel::{level::Level, toast::ToastView};

/// How long a toast stays on screen.
pub const DEFAULT_TTL: Duration = Duration::from_secs(4);

/// Longer TTL for failures, which usually carry an error code worth reading.
pub const ERROR_TTL: Duration = Duration::from_secs(6);

/// One transient message shown by the toast widget.
#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub level: Level,
    pub header: String,
    pub body: String,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl ToastMessage {
    pub fn new(
        level: Level,
        header: impl Into<String>,
        body: impl Into<String>,
        ttl: Duration,
    ) -> Self {
        Self {
            level,
            header: header.into(),
            body: body.into(),
            created_at: Instant::now(),
            ttl,
        }
    }

    /// Whether the toast should be gone at `now`.
    pub fn is_expired(&self, now: Instant) -> bool {
        !self.ttl.is_zero() && now.duration_since(self.created_at) >= self.ttl
    }
}

/// The toast currently on screen; a new one replaces it.
#[derive(Debug, Default)]
pub struct ToastState {
    current: Option<ToastMessage>,
}

impl ToastState {
    /// Show a toast, replacing any visible one.
    pub fn show(&mut self, level: Level, header: impl Into<String>, body: impl Into<String>) {
        let ttl = match level {
            Level::Error | Level::Panic => ERROR_TTL,
            _ => DEFAULT_TTL,
        };
        self.current = Some(ToastMessage::new(level, header, body, ttl));
    }

    pub fn info(&mut self, header: impl Into<String>, body: impl Into<String>) {
        self.show(Level::Info, header, body);
    }

    pub fn success(&mut self, header: impl Into<String>, body: impl Into<String>) {
        self.show(Level::Success, header, body);
    }

    pub fn warning(&mut self, header: impl Into<String>, body: impl Into<String>) {
        self.show(Level::Warning, header, body);
    }

    pub fn error(&mut self, header: impl Into<String>, body: impl Into<String>) {
        self.show(Level::Error, header, body);
    }

    /// The toast to draw, if one is visible.
    pub fn visible(&self) -> Option<&ToastMessage> {
        self.current.as_ref()
    }

    pub fn dismiss(&mut self) {
        self.current = None;
    }

    /// Drop the toast once its TTL has passed. Called once per frame.
    pub fn poll_expiry(&mut self, now: Instant) {
        if let Some(toast) = &self.current {
            if toast.is_expired(now) {
                self.current = None;
            }
        }
    }
}

impl<'a> From<&'a ToastMessage> for ToastView<'a> {
    fn from(toast: &'a ToastMessage) -> Self {
        Self {
            level: toast.level,
            header: &toast.header,
            body: &toast.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn showing_a_toast_replaces_the_previous_one() {
        let mut state = ToastState::default();
        state.info("Opened", "src/main.rs");
        state.error("Command not found", "E492: Not an editor command: nope");

        let toast = state.visible().expect("a toast is visible");
        assert_eq!(toast.level, Level::Error);
        assert_eq!(toast.header, "Command not found");
    }

    #[test]
    fn poll_expiry_drops_the_toast() {
        let mut state = ToastState::default();
        state.success("Saved", "src/main.rs");

        let now = state.visible().unwrap().created_at + DEFAULT_TTL;
        state.poll_expiry(now);

        assert!(state.visible().is_none());
    }

    #[test]
    fn errors_live_longer_than_successes() {
        let mut state = ToastState::default();
        state.error("Command not found", "E492");
        let error_ttl = state.visible().unwrap().ttl;

        state.success("Saved", "ok");
        let success_ttl = state.visible().unwrap().ttl;

        assert!(error_ttl > success_ttl);
    }

    #[test]
    fn converts_to_the_widget_view() {
        let mut state = ToastState::default();
        state.warning("No other buffer", "E85: Only one buffer");

        let view = ToastView::from(state.visible().unwrap());
        assert_eq!(view.level, Level::Warning);
        assert_eq!(view.header, "No other buffer");
        assert_eq!(view.body, "E85: Only one buffer");
    }
}
