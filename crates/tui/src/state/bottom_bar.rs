//! Persistent state for the bottom bar and command palette.
//!
//! Owns the command-line input buffer, its cursor, the command history, and
//! any transient status message with an expiry timestamp. Everything else
//! displayed by the bottom bar or the palette is passed in as view-model data
//! each frame.
//!
//! The command line is character-indexed (`command_cursor` counts chars, not
//! bytes) so multi-byte input edits correctly.

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

/// Bottom-bar state: command-line buffer, cursor within it, command history,
/// and a transient status message.
#[derive(Debug, Default)]
pub struct BottomBarState {
    /// Text typed into `:` command mode. Empty when not in command mode.
    pub command_line: String,

    /// Cursor position within `command_line`, measured in **characters**
    /// (not bytes), so multi-byte input works correctly.
    pub command_cursor: usize,

    /// Submitted command lines, oldest first. Used by `Up`/`Down` in command
    /// mode. Capped at [`HISTORY_LIMIT`].
    pub history: Vec<String>,

    /// Position while browsing history; `None` when editing a fresh line.
    pub history_index: Option<usize>,

    /// The in-progress line saved before browsing history, restored when the
    /// user moves past the newest entry.
    pub history_draft: String,

    /// Currently visible transient message, if any.
    pub message: Option<StatusMessage>,
}

/// Maximum number of remembered command lines.
pub const HISTORY_LIMIT: usize = 100;

impl BottomBarState {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Command line -----------------------------------------------------

    /// Reset the command-line buffer, cursor, and history browsing position.
    /// Called when entering or leaving command mode.
    pub fn clear_command_line(&mut self) {
        self.command_line.clear();
        self.command_cursor = 0;
        self.reset_history_nav();
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

    // --- History ----------------------------------------------------------

    /// Record a submitted command line, ignoring empty input and consecutive
    /// duplicates. Resets history browsing.
    pub fn push_history(&mut self, line: impl Into<String>) {
        let line = line.into();
        if line.trim().is_empty() {
            return;
        }
        if self.history.last() != Some(&line) {
            self.history.push(line);
        }
        if self.history.len() > HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.reset_history_nav();
    }

    /// Stop browsing history and forget the saved draft.
    pub fn reset_history_nav(&mut self) {
        self.history_index = None;
        self.history_draft.clear();
    }

    /// Recall the previous (older) history entry, saving the current line as a
    /// draft on the first step.
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let index = match self.history_index {
            None => {
                self.history_draft = self.command_line.clone();
                self.history.len() - 1
            }
            Some(0) => 0,
            Some(index) => index - 1,
        };

        self.history_index = Some(index);
        self.command_line = self.history[index].clone();
        self.command_cursor = self.command_line.chars().count();
    }

    /// Recall the next (newer) history entry, restoring the draft once past
    /// the newest entry.
    pub fn history_next(&mut self) {
        let Some(index) = self.history_index else {
            return;
        };

        if index + 1 < self.history.len() {
            self.history_index = Some(index + 1);
            self.command_line = self.history[index + 1].clone();
        } else {
            self.history_index = None;
            self.command_line = std::mem::take(&mut self.history_draft);
        }
        self.command_cursor = self.command_line.chars().count();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_char_inserts_at_cursor() {
        let mut state = BottomBarState::new();
        state.push_char('w');
        state.push_char('q');
        state.move_cursor_left();
        state.push_char('!');

        assert_eq!(state.command_line, "w!q");
        assert_eq!(state.command_cursor, 2);
    }

    #[test]
    fn backspace_handles_multibyte_characters() {
        let mut state = BottomBarState::new();
        state.push_char('é');
        state.push_char('x');
        state.backspace();
        state.backspace();

        assert_eq!(state.command_line, "");
        assert_eq!(state.command_cursor, 0);
    }

    #[test]
    fn history_prev_recalls_and_next_restores_draft() {
        let mut state = BottomBarState::new();
        state.push_history("w");
        state.push_history("q");

        state.push_char('e');
        state.history_prev();
        assert_eq!(state.command_line, "q");

        state.history_prev();
        assert_eq!(state.command_line, "w");

        state.history_next();
        assert_eq!(state.command_line, "q");

        state.history_next();
        assert_eq!(state.command_line, "e");
        assert_eq!(state.history_index, None);
    }

    #[test]
    fn history_ignores_empty_and_duplicate_lines() {
        let mut state = BottomBarState::new();
        state.push_history("  ");
        state.push_history("w");
        state.push_history("w");

        assert_eq!(state.history, vec!["w".to_string()]);
    }

    #[test]
    fn clearing_resets_history_browsing() {
        let mut state = BottomBarState::new();
        state.push_history("w");
        state.history_prev();
        state.clear_command_line();

        assert_eq!(state.command_line, "");
        assert_eq!(state.history_index, None);
        assert!(state.history_draft.is_empty());
    }
}
