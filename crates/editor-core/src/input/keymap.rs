//! Keymap: a table of key sequences → commands.
//!
//! Sequences are *chords*: one or more keys pressed in order. A single-key
//! binding is just a sequence of length 1.

use std::collections::HashMap;
use std::vec;

use super::key::{Key, KeyCode};
use super::mode::Mode;
use crate::command::command::{Command, Direction};

/// A sequence of keys, e.g. `[g, g]` or `[Ctrl+w, Ctrl+w]`.
pub type KeySeq = Vec<Key>;

/// A keymap: bindings organized by mode.
#[derive(Debug)]
pub struct Keymap {
    /// mode -> full sequence -> command
    bindings: HashMap<Mode, HashMap<KeySeq, Command>>,
}

impl Default for Keymap {
    fn default() -> Self {
        default_keymap()
    }
}

impl Keymap {
    /// Construct a keymap with default bindings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an empty keymap with no bindings.
    pub fn empty() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, mode: Mode, seq: impl Into<KeySeq>, cmd: Command) {
        self.bindings
            .entry(mode)
            .or_default()
            .insert(seq.into(), cmd);
    }

    /// Look up an exact sequence in a mode.
    pub fn lookup(&self, mode: Mode, seq: &[Key]) -> Option<&Command> {
        self.bindings.get(&mode)?.get(seq)
    }

    /// Is `seq` a *prefix* of any longer binding in this mode?
    ///
    /// Used by the resolver to decide whether to wait for the next key
    /// (multi-key binding in progress) or to give up.
    pub fn is_prefix(&self, mode: Mode, seq: &[Key]) -> bool {
        let Some(table) = self.bindings.get(&mode) else {
            return false;
        };
        table
            .keys()
            .any(|k| k.len() > seq.len() && k.starts_with(seq))
    }

    /// Does any binding in this mode *start with* `seq`? Includes exact
    /// matches, so callers can check "is this a partial path at all".
    pub fn has_any_match(&self, mode: Mode, seq: &[Key]) -> bool {
        let Some(table) = self.bindings.get(&mode) else {
            return false;
        };
        table.keys().any(|k| k.starts_with(seq))
    }
}

/// Build the default keymap.
///
/// Bindings are grouped by mode. Add new ones here; consumers can layer
/// overrides on top via `Keymap::bind`.
pub fn default_keymap() -> Keymap {
    let mut km = Keymap::empty();

    // --- Normal mode ----------------------------------------------------
    let n = Mode::Normal;

    // Quit
    km.bind(n, vec![Key::plain(KeyCode::Escape)], Command::Quit);
    km.bind(n, vec![Key::ctrl('q')], Command::ForceQuit);
    km.bind(n, vec![Key::ctrl('c')], Command::ForceQuit);

    // Save
    km.bind(n, vec![Key::ctrl('s')], Command::SaveBuffer);

    // Tabs
    km.bind(n, vec![Key::plain(KeyCode::Tab)], Command::NextTab);
    km.bind(n, vec![Key::plain(KeyCode::BackTab)], Command::PrevTab);
    km.bind(n, vec![Key::ctrl('n')], Command::NewScratchBuffer);
    km.bind(n, vec![Key::ctrl('w')], Command::CloseTab);

    // Layout
    km.bind(n, vec![Key::ctrl('f')], Command::ToggleFullscreen);

    // Mode transitions
    km.bind(n, vec![Key::char('i')], Command::EnterMode(Mode::Insert));
    km.bind(n, vec![Key::char('v')], Command::EnterMode(Mode::Visual));
    km.bind(n, vec![Key::char(':')], Command::OpenCommandLine);
    km.bind(
        n,
        vec![Key::char(' '), Key::char('e')],
        Command::ToggleLeftPanel,
    );

    // Cursor movement (hjkl + arrows)
    km.bind(
        n,
        vec![Key::char('h')],
        Command::MoveCursor(Direction::Left),
    );
    km.bind(
        n,
        vec![Key::char('j')],
        Command::MoveCursor(Direction::Down),
    );
    km.bind(n, vec![Key::char('k')], Command::MoveCursor(Direction::Up));
    km.bind(
        n,
        vec![Key::char('l')],
        Command::MoveCursor(Direction::Right),
    );
    km.bind(
        n,
        vec![Key::plain(KeyCode::Left)],
        Command::MoveCursor(Direction::Left),
    );
    km.bind(
        n,
        vec![Key::plain(KeyCode::Down)],
        Command::MoveCursor(Direction::Down),
    );
    km.bind(
        n,
        vec![Key::plain(KeyCode::Up)],
        Command::MoveCursor(Direction::Up),
    );
    km.bind(
        n,
        vec![Key::plain(KeyCode::Right)],
        Command::MoveCursor(Direction::Right),
    );

    // Multi-key: `gg` jumps to start of file
    km.bind(
        n,
        vec![Key::char('g'), Key::char('g')],
        Command::MoveToFileStart,
    );
    // Multi-key: `dd` deletes a line (stub for later)
    // km.bind(n, vec![Key::char('d'), Key::char('d')], Command::DeleteLine);

    // --- Insert mode ----------------------------------------------------
    let i = Mode::Insert;
    km.bind(i, vec![Key::plain(KeyCode::Escape)], Command::ExitToNormal);
    km.bind(i, vec![Key::plain(KeyCode::Enter)], Command::InsertNewline);
    km.bind(i, vec![Key::plain(KeyCode::Backspace)], Command::Backspace);
    // Printable chars handled by fall-through in the resolver, not bound here.

    // --- Command-line mode ---------------------------------------------
    let c = Mode::Command;
    km.bind(
        c,
        vec![Key::plain(KeyCode::Escape)],
        Command::CancelCommandLine,
    );
    km.bind(
        c,
        vec![Key::plain(KeyCode::Enter)],
        Command::SubmitCommandLine,
    );
    km.bind(
        c,
        vec![Key::plain(KeyCode::Backspace)],
        Command::CommandLinePop,
    );
    // Printable chars handled by fall-through.

    km
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keymap_bindings() {
        let km = Keymap::default();
        // Esc binds to Quit
        assert_eq!(
            km.lookup(Mode::Normal, &[Key::plain(KeyCode::Escape)]),
            Some(&Command::Quit)
        );
        // Ctrl+q binds to ForceQuit
        assert_eq!(
            km.lookup(Mode::Normal, &[Key::ctrl('q')]),
            Some(&Command::ForceQuit)
        );
        // Ctrl+c binds to ForceQuit
        assert_eq!(
            km.lookup(Mode::Normal, &[Key::ctrl('c')]),
            Some(&Command::ForceQuit)
        );
    }

    #[test]
    fn test_empty_keymap() {
        let km = Keymap::empty();
        assert_eq!(
            km.lookup(Mode::Normal, &[Key::plain(KeyCode::Escape)]),
            None
        );
    }
}
