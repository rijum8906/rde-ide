//! Persistent state for the bottom bar widget.
//!
//! Owns the command-line input buffer, its cursor, and any transient status
//! message with an expiry timestamp. Everything else displayed by the bottom
//! bar is passed in as view-model data each frame.

use std::time::{Duration, Instant};

/// Severity of a status message. Affects color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Info,
    Warning,
    Error,
}

/// A transient message shown on the status line.
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub kind: MessageKind,
    /// When the message was set. Used to expire it.
    pub since: Instant,
    /// How long the message stays visible.
    pub ttl: Duration,
}

impl StatusMessage {
    pub fn new(text: impl Into<String>, kind: MessageKind) -> Self {
        Self {
            text: text.into(),
            kind,
            since: Instant::now(),
            ttl: Duration::from_secs(3),
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        now.duration_since(self.since) >= self.ttl
    }
}

/// Bottom-bar state: command-line buffer, cursor within it, and a
/// transient status message.
#[derive(Debug, Default)]
pub struct BottomBarState {
    /// Text typed into `:` command mode. Empty when not in command mode.
    pub command_line: String,

    /// Cursor position within `command_line`, measured in **characters**
    /// (not bytes), so multi-byte input works correctly.
    pub command_cursor: usize,

    /// Currently visible transient message, if any.
    pub message: Option<StatusMessage>,
}

impl BottomBarState {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Command line -----------------------------------------------------

    /// Reset the command-line buffer and cursor. Called when entering or
    /// leaving command mode.
    pub fn clear_command_line(&mut self) {
        self.command_line.clear();
        self.command_cursor = 0;
    }

    /// Append a character at the cursor, advancing the cursor.
    pub fn push_char(&mut self, c: char) {
        let byte_idx = self.cursor_byte_index();
        self.command_line.insert(byte_idx, c);
        self.command_cursor += 1;
    }

    /// Delete the character before the cursor. No-op at the start.
    pub fn backspace(&mut self) {
        if self.command_cursor == 0 {
            return;
        }
        let prev_byte = self.byte_index_at_char(self.command_cursor - 1);
        let cur_byte = self.cursor_byte_index();
        self.command_line.replace_range(prev_byte..cur_byte, "");
        self.command_cursor -= 1;
    }

    /// Delete the character at the cursor. No-op at the end.
    pub fn delete(&mut self) {
        let len_chars = self.command_line.chars().count();
        if self.command_cursor >= len_chars {
            return;
        }
        let cur_byte = self.cursor_byte_index();
        let next_byte = self.byte_index_at_char(self.command_cursor + 1);
        self.command_line.replace_range(cur_byte..next_byte, "");
    }

    pub fn move_cursor_left(&mut self) {
        self.command_cursor = self.command_cursor.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        let len_chars = self.command_line.chars().count();
        if self.command_cursor < len_chars {
            self.command_cursor += 1;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.command_cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.command_cursor = self.command_line.chars().count();
    }

    /// Take the command-line text, leaving the buffer empty. Used on submit.
    pub fn take_command_line(&mut self) -> String {
        self.command_cursor = 0;
        std::mem::take(&mut self.command_line)
    }

    /// Byte offset in `command_line` for the given character index.
    fn byte_index_at_char(&self, char_idx: usize) -> usize {
        self.command_line
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.command_line.len())
    }

    fn cursor_byte_index(&self) -> usize {
        self.byte_index_at_char(self.command_cursor)
    }

    // --- Status message ---------------------------------------------------

    /// Show a message, replacing any current one.
    pub fn set_message(&mut self, text: impl Into<String>, kind: MessageKind) {
        self.message = Some(StatusMessage::new(text, kind));
    }

    pub fn info(&mut self, text: impl Into<String>) {
        self.set_message(text, MessageKind::Info);
    }

    pub fn warn(&mut self, text: impl Into<String>) {
        self.set_message(text, MessageKind::Warning);
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.set_message(text, MessageKind::Error);
    }

    /// Clear any current message.
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    /// Expire the message if its TTL has passed. Call once per frame.
    pub fn poll_message_expiry(&mut self, now: Instant) {
        if let Some(msg) = &self.message {
            if msg.is_expired(now) {
                self.message = None;
            }
        }
    }
}
