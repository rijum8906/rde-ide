//! Stateful key-sequence resolver.
//!
//! Feed it keys, get back either:
//!   - `Resolved(cmd)`   — a complete binding matched
//!   - `Pending`         — wait for more keys (prefix matched)
//!   - `Unhandled(keys)` — nothing matched; caller may pass through
//!   - `Abandoned(keys)` — prefix failed partway; caller decides what to do

use std::time::{Duration, Instant};

use super::key::Key;
use super::keymap::Keymap;
use super::mode::Mode;
use crate::command::command::Command;

/// How long to wait for the next key before giving up on a partial sequence.
///
/// Vim uses ~1000ms (`timeoutlen`). 500ms feels snappier for modern users.
const PREFIX_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub enum ResolveResult {
    /// A complete binding matched; execute this command.
    Resolved(Command),
    /// Partial sequence matched; waiting for more keys.
    Pending,
    /// Nothing matched at all. The keys are returned so the caller can
    /// implement fall-through (e.g. insert-mode character entry).
    Unhandled(Vec<Key>),
    /// A prefix matched earlier but this key broke it. The caller may want
    /// to beep, reset, or re-feed the keys.
    Abandoned(Vec<Key>),
}

#[derive(Debug)]
pub struct Resolver {
    /// Keys pressed so far for the current (possibly multi-key) sequence.
    pending: Vec<Key>,
    /// When the first pending key was pressed. Used for timeout.
    pending_since: Option<Instant>,
    /// Mode the pending sequence started in. If the mode changes mid-sequence
    /// (shouldn't happen, but defensive), we abort.
    pending_mode: Option<Mode>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            pending_since: None,
            pending_mode: None,
        }
    }

    /// Reset pending state (e.g. on focus loss, mode change, or error).
    pub fn reset(&mut self) {
        self.pending.clear();
        self.pending_since = None;
        self.pending_mode = None;
    }

    pub fn pending_keys(&self) -> &[Key] {
        &self.pending
    }

    /// True if the resolver is waiting for more keys.
    pub fn is_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Expire a stale prefix. Returns `true` if a prefix was dropped.
    /// Call this once per frame *before* feeding a key.
    pub fn poll_timeout(&mut self, now: Instant) -> bool {
        if let Some(since) = self.pending_since {
            if now.duration_since(since) >= PREFIX_TIMEOUT {
                self.reset();
                return true;
            }
        }
        false
    }

    /// Feed a key. See `ResolveResult` for semantics.
    pub fn feed(&mut self, mode: Mode, key: Key, keymap: &Keymap) -> ResolveResult {
        // If the mode changed since the last key, drop pending state.
        if let Some(prev) = self.pending_mode {
            if prev != mode {
                self.reset();
            }
        }
        if self.pending.is_empty() {
            self.pending_since = Some(Instant::now());
        }
        self.pending_mode = Some(mode);
        self.pending.push(key);

        // 1. Exact match?
        if let Some(cmd) = keymap.lookup(mode, &self.pending) {
            let cmd = cmd.clone();
            // If this sequence is also a prefix of a longer binding, we have
            // ambiguity. Vim resolves this with `timeoutlen` — wait to see if
            // more keys come. For simplicity, prefer the exact match now.
            // (To prefer prefix + timeout, comment this out and let the
            //  `is_prefix` branch below handle it.)
            let had_prefix_before = self.pending.len() > 1;
            let _ = had_prefix_before;
            self.reset();
            return ResolveResult::Resolved(cmd);
        }

        // 2. Prefix of a longer binding? Wait.
        if keymap.is_prefix(mode, &self.pending) {
            return ResolveResult::Pending;
        }

        // 3. No match, no prefix. Abandon.
        let abandoned = std::mem::take(&mut self.pending);
        self.pending_since = None;
        self.pending_mode = None;

        if abandoned.len() == 1 {
            // Single unbound key — unhandled, not abandoned. This is the
            // common "insert mode printable char" case.
            ResolveResult::Unhandled(abandoned)
        } else {
            ResolveResult::Abandoned(abandoned)
        }
    }
}
