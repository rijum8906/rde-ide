//! Input handling.
//!
//! Turns terminal key presses into [`Command`](crate::command::Command)s. The
//! pieces are deliberately separate:
//!
//! - [`key`] normalizes terminal events into the editor's own key type,
//! - [`keymap`] stores bindings per mode,
//! - [`mode`] names the input modes those bindings are grouped by,
//! - [`resolver`] matches key sequences and reports what to do next.
//!
//! [`Input`] bundles the persistent parts so a frontend can hold them as one
//! value.

use crate::input::{keymap::Keymap, mode::InputMode, resolver::Resolver};

pub mod key;
pub mod keymap;
pub mod mode;
pub mod resolver;

#[derive(Debug, Default)]
pub struct Input {
    pub mode: InputMode,
    pub resolver: Resolver,
    pub key_map: Keymap,
}

impl AsRef<InputMode> for Input {
    fn as_ref(&self) -> &InputMode {
        &self.mode
    }
}
