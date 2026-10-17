//! Keymap: a table of key sequences → commands.
//!
//! Sequences are *chords*: one or more keys pressed in order. A single-key
//! binding is just a sequence of length 1.

use std::collections::HashMap;
use std::vec;

use super::key::{Key, KeyCode, KeyExt};
use super::mode::Mode;
use crate::command::command::Command;
use crate::command::command::CommandLineCommand::{Cancel, Pop, Submit};
use crate::command::command::EditCommand::{Backspace, DeleteLine, InsertNewline};
use crate::command::command::LayoutCommand::{ToggleFullscreen, ToggleLeftPanel};
use crate::command::command::ModeCommand::{
    Enter, ExitToNormal, InsertAfter, InsertAtLineEnd, InsertAtLineStart, InsertNewLineAbove,
    InsertNewLineBelow,
};
use crate::command::command::Motion::{
    Down, FileEnd, FileStart, Left, LineEnd, LineStart, Right, Up,
};
use crate::command::command::NavigationCommand::{CloseTab, NextTab, PrevTab};
use crate::command::command::SessionCommand::{ForceQuit, NewScratchBuffer, Quit, SaveBuffer};

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
    km.bind(n, vec![Key::plain(KeyCode::Esc)], Command::Session(Quit));
    km.bind(n, vec![Key::ctrl('q')], Command::Session(ForceQuit));
    km.bind(n, vec![Key::ctrl('c')], Command::Session(ForceQuit));

    // Save
    km.bind(n, vec![Key::ctrl('s')], Command::Session(SaveBuffer));

    // Tabs
    km.bind(
        n,
        vec![Key::plain(KeyCode::Tab)],
        Command::Navigation(NextTab),
    );
    km.bind(
        n,
        vec![Key::plain(KeyCode::BackTab)],
        Command::Navigation(PrevTab),
    );
    km.bind(n, vec![Key::ctrl('n')], Command::Session(NewScratchBuffer));
    km.bind(n, vec![Key::ctrl('w')], Command::Navigation(CloseTab));

    // Layout
    km.bind(n, vec![Key::ctrl('f')], Command::Layout(ToggleFullscreen));

    // Mode transitions
    km.bind(n, vec![Key::char('i')], Command::Mode(Enter(Mode::Insert)));
    km.bind(n, vec![Key::char('v')], Command::Mode(Enter(Mode::Visual)));
    km.bind(n, vec![Key::char('a')], Command::Mode(InsertAfter));
    km.bind(n, vec![Key::char('A')], Command::Mode(InsertAtLineEnd));
    km.bind(n, vec![Key::char('I')], Command::Mode(InsertAtLineStart));
    km.bind(n, vec![Key::char('o')], Command::Mode(InsertNewLineBelow));
    km.bind(n, vec![Key::char('O')], Command::Mode(InsertNewLineAbove));
    km.bind(n, vec![Key::char(':')], Command::Mode(Enter(Mode::Command)));
    km.bind(
        n,
        vec![Key::char('d'), Key::char('d')],
        Command::Edit(DeleteLine),
    );
    km.bind(
        n,
        vec![Key::char(' '), Key::char('e')],
        Command::Layout(ToggleLeftPanel),
    );
    km.bind(n, vec![Key::ctrl('b')], Command::Layout(ToggleLeftPanel));

    // Cursor movement (hjkl + arrows)
    km.bind(n, vec![Key::char('h')], Command::Motion(Left));
    km.bind(n, vec![Key::char('j')], Command::Motion(Down));
    km.bind(n, vec![Key::char('k')], Command::Motion(Up));
    km.bind(n, vec![Key::char('l')], Command::Motion(Right));
    km.bind(n, vec![Key::plain(KeyCode::Left)], Command::Motion(Left));
    km.bind(n, vec![Key::plain(KeyCode::Down)], Command::Motion(Down));
    km.bind(n, vec![Key::plain(KeyCode::Up)], Command::Motion(Up));
    km.bind(n, vec![Key::plain(KeyCode::Right)], Command::Motion(Right));

    // Multi-key: `gg` jumps to start of file
    km.bind(
        n,
        vec![Key::char('g'), Key::char('g')],
        Command::Motion(FileStart),
    );
    km.bind(n, vec![Key::char('G')], Command::Motion(FileEnd));
    km.bind(n, vec![Key::char('$')], Command::Motion(LineEnd));
    km.bind(n, vec![Key::char('0')], Command::Motion(LineStart));

    // --- Insert mode ----------------------------------------------------
    let i = Mode::Insert;
    km.bind(
        i,
        vec![Key::plain(KeyCode::Esc)],
        Command::Mode(ExitToNormal),
    );
    km.bind(
        i,
        vec![Key::plain(KeyCode::Enter)],
        Command::Edit(InsertNewline),
    );
    km.bind(
        i,
        vec![Key::plain(KeyCode::Backspace)],
        Command::Edit(Backspace),
    );
    km.bind(i, vec![Key::plain(KeyCode::Left)], Command::Motion(Left));
    km.bind(i, vec![Key::plain(KeyCode::Down)], Command::Motion(Down));
    km.bind(i, vec![Key::plain(KeyCode::Up)], Command::Motion(Up));
    km.bind(i, vec![Key::plain(KeyCode::Right)], Command::Motion(Right));
    km.bind(i, vec![Key::ctrl('j')], Command::Mode(ExitToNormal));
    km.bind(i, vec![Key::ctrl('k')], Command::Mode(ExitToNormal));
    // Printable chars handled by fall-through in the resolver, not bound here.

    // --- Command-line mode ---------------------------------------------
    let c = Mode::Command;
    km.bind(
        c,
        vec![Key::plain(KeyCode::Esc)],
        Command::CommandLine(Cancel),
    );
    km.bind(
        c,
        vec![Key::plain(KeyCode::Enter)],
        Command::CommandLine(Submit),
    );
    km.bind(
        c,
        vec![Key::plain(KeyCode::Backspace)],
        Command::CommandLine(Pop),
    );
    // Printable chars handled by fall-through.
    //
    let v = Mode::Visual;
    km.bind(
        v,
        vec![Key::plain(KeyCode::Esc)],
        Command::Mode(ExitToNormal),
    );

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
            km.lookup(Mode::Normal, &[Key::plain(KeyCode::Esc)]),
            Some(&Command::Session(Quit))
        );
        // Ctrl+q binds to ForceQuit
        assert_eq!(
            km.lookup(Mode::Normal, &[Key::ctrl('q')]),
            Some(&Command::Session(ForceQuit))
        );
        // Ctrl+c binds to ForceQuit
        assert_eq!(
            km.lookup(Mode::Normal, &[Key::ctrl('c')]),
            Some(&Command::Session(ForceQuit))
        );
    }

    #[test]
    fn test_empty_keymap() {
        let km = Keymap::empty();
        assert_eq!(km.lookup(Mode::Normal, &[Key::plain(KeyCode::Esc)]), None);
    }

    #[test]
    fn test_insert_mode_ctrl_j_and_ctrl_b() {
        let km = Keymap::default();
        assert_eq!(
            km.lookup(Mode::Insert, &[Key::ctrl('b')]),
            Some(&Command::Layout(ToggleLeftPanel))
        );
        assert_eq!(
            km.lookup(Mode::Insert, &[Key::ctrl('j')]),
            Some(&Command::Mode(Enter(Mode::Normal)))
        );
        // Also verify case insensitivity with Key::ctrl('J')
        assert_eq!(
            km.lookup(Mode::Insert, &[Key::ctrl('J')]),
            Some(&Command::Mode(Enter(Mode::Normal)))
        );
    }
}
