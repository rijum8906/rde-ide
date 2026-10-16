//! Key-sequence resolver.
//!
//! The resolver tracks the current (possibly incomplete) key sequence, feeds
//! it to a `Keymap`, and reports one of four outcomes:
//!
//! - `Resolved(cmd)`   — a complete binding matched; execute `cmd`.
//! - `Pending`         — a proper prefix matched; wait for more keys.
//! - `Unhandled(keys)` — nothing matched; caller may fall through.
//! - `Abandoned(keys)` — a prefix broke partway; caller may beep.

use std::time::{Duration, Instant};

use smallvec::SmallVec;

use crate::command::command::Command;
use crate::input::key::Key;
use crate::input::keymap::Keymap;
use crate::input::mode::Mode;

/// How long to wait for the next key before giving up on a partial sequence.
///
/// Vim uses ~1000ms (`timeoutlen`). 500ms feels snappier for modern users.
pub const PREFIX_TIMEOUT: Duration = Duration::from_millis(500);

/// Pending keys for the current sequence.
///
/// Inline capacity 4 covers `gg`, `dd`, `ciw`, `C-w C-w`, and most Helix
/// `Space` sequences without heap allocation. Longer sequences spill to the
/// heap and still work correctly.
pub type Pending = SmallVec<[Key; 4]>;

/// Outcome of feeding a key to the resolver.
#[derive(Debug, Clone)]
pub enum ResolveResult {
    /// A complete binding matched; execute this command.
    Resolved(Command),
    /// A prefix matched; waiting for more keys.
    Pending,
    /// Nothing matched at all. The keys are returned so the caller can
    /// implement fall-through (e.g. insert-mode character entry).
    Unhandled(Pending),
    /// A prefix matched earlier but this key broke it.
    Abandoned(Pending),
}

/// Stateful key-sequence matcher.
///
/// Feed it keys with `feed`; poll it once per frame with `poll_timeout` to
/// expire stale prefixes. It never holds more than one sequence at a time.
#[derive(Debug)]
pub struct Resolver {
    /// Keys pressed so far for the current (possibly multi-key) sequence.
    pending: Pending,
    /// When the first pending key was pressed, for timeout.
    pending_since: Option<Instant>,
    /// Mode the pending sequence started in. If the mode changes mid-sequence,
    /// the pending state is dropped.
    pending_mode: Option<Mode>,
    /// If the current pending sequence is *also* a complete binding (e.g.
    /// `g` is bound to something AND is a prefix of `gg`), remember the
    /// exact match so we can fire it on timeout.
    ambiguous_exact: Option<Command>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            pending: Pending::new(),
            pending_since: None,
            pending_mode: None,
            ambiguous_exact: None,
        }
    }

    /// Borrow the pending keys (empty if no sequence in progress).
    pub fn pending(&self) -> &[Key] {
        &self.pending
    }

    /// True if the resolver is waiting for more keys.
    pub fn is_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Reset all pending state.
    fn reset(&mut self) {
        self.pending.clear();
        self.pending_since = None;
        self.pending_mode = None;
        self.ambiguous_exact = None;
    }

    /// Expire a stale prefix.
    ///
    /// If the pending sequence had an exact match *and* was a prefix of a
    /// longer binding, fires the exact match on timeout. Otherwise, drops
    /// the pending sequence. Returns `Some(cmd)` if the timeout resolved to
    /// a command, `None` otherwise.
    ///
    /// Call this once per frame, before feeding new keys.
    pub fn poll_timeout(&mut self, now: Instant) -> Option<Command> {
        let since = self.pending_since?;
        if now.duration_since(since) < PREFIX_TIMEOUT {
            return None;
        }

        if let Some(cmd) = self.ambiguous_exact.take() {
            self.reset();
            return Some(cmd);
        }

        self.reset();
        None
    }

    /// Feed a key.
    ///
    /// See `ResolveResult` for the four possible outcomes.
    pub fn feed(&mut self, mode: Mode, key: Key, keymap: &Keymap) -> ResolveResult {
        // 1. Mode change invalidates any in-flight sequence.
        if let Some(prev) = self.pending_mode {
            if prev != mode {
                self.reset();
            }
        }

        // 2. First key of a new sequence: start the timeout clock.
        if self.pending.is_empty() {
            self.pending_since = Some(Instant::now());
            self.ambiguous_exact = None;
        }
        self.pending_mode = Some(mode);

        // 3. Append the new key.
        self.pending.push(key);

        // 4. Is this a complete binding?
        let exact = keymap.lookup(mode, &self.pending).cloned();

        // 5. Is this also a prefix of a longer binding?
        let is_prefix = keymap.is_prefix(mode, &self.pending);

        match (exact, is_prefix) {
            // Complete match, not a prefix of anything longer: fire now.
            (Some(cmd), false) => {
                self.reset();
                ResolveResult::Resolved(cmd)
            }

            // Complete match, but also a prefix of a longer binding.
            // Ambiguous — wait for more keys, and remember the exact match
            // in case the timeout fires first.
            (Some(cmd), true) => {
                self.ambiguous_exact = Some(cmd);
                ResolveResult::Pending
            }

            // Not a complete match, but a prefix: wait.
            (None, true) => ResolveResult::Pending,

            // Neither exact nor prefix: this key broke the sequence.
            (None, false) => {
                let drained = std::mem::take(&mut self.pending);
                self.pending_since = None;
                self.pending_mode = None;
                self.ambiguous_exact = None;

                if drained.len() == 1 {
                    // Single unbound key — this is the common "insert mode
                    // printable char" case. Unhandled, not abandoned.
                    ResolveResult::Unhandled(drained)
                } else {
                    ResolveResult::Abandoned(drained)
                }
            }
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
