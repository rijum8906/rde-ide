//! Keymap: a table of key sequences → commands.
//!
//! Sequences are *chords*: one or more keys pressed in order. A single-key
//! binding is just a sequence of length 1.

use std::collections::HashMap;

use super::key::Key;
use super::mode::Mode;
use crate::command::command::Command;
use crate::input::keymap::command_line::default_command_line_keymap;
use crate::input::keymap::edit::default_edit_keymap;
use crate::input::keymap::layout::default_layout_keymap;
use crate::input::keymap::mode::default_mode_keymap;
use crate::input::keymap::motion::default_motion_keymap;
use crate::input::keymap::session::default_session_keymap;
use crate::input::keymap::workspace::default_workspace_keymap;

mod command_line;
mod edit;
mod layout;
mod mode;
mod motion;
mod session;
mod workspace;

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

/// Build the default editor keymap from the command-specific binding groups.
pub fn default_keymap() -> Keymap {
    let mut keymap = Keymap::empty();

    default_session_keymap(&mut keymap);
    default_workspace_keymap(&mut keymap);
    default_layout_keymap(&mut keymap);
    default_mode_keymap(&mut keymap);
    default_motion_keymap(&mut keymap);
    default_edit_keymap(&mut keymap);
    default_command_line_keymap(&mut keymap);

    keymap
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command::command::{
            Command,
            LayoutCommand::ToggleLeftPanel,
            ModeCommand::ExitToNormal,
            SessionCommand::{ForceQuit, Quit},
        },
        input::key::{KeyCode, KeyExt},
    };

    #[test]
    fn default_keymap_contains_session_bindings() {
        let keymap = Keymap::default();
        assert_eq!(
            keymap.lookup(Mode::Normal, &[Key::plain(KeyCode::Esc)]),
            Some(&Command::Session(Quit))
        );
        assert_eq!(
            keymap.lookup(Mode::Normal, &[Key::ctrl('q')]),
            Some(&Command::Session(ForceQuit))
        );
    }

    #[test]
    fn empty_keymap_contains_no_bindings() {
        assert_eq!(
            Keymap::empty().lookup(Mode::Normal, &[Key::plain(KeyCode::Esc)]),
            None
        );
    }

    #[test]
    fn default_keymap_contains_mode_and_layout_bindings() {
        let keymap = Keymap::default();
        assert_eq!(
            keymap.lookup(Mode::Normal, &[Key::ctrl('b')]),
            Some(&Command::Layout(ToggleLeftPanel))
        );
        assert_eq!(
            keymap.lookup(Mode::Insert, &[Key::ctrl('j')]),
            Some(&Command::Mode(ExitToNormal))
        );
    }
}
