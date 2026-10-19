//! Mode transitions.
//!
//! Some variants model the edit implied by entering a mode ("insert above",
//! "open a line below"), which keeps the keymap declarative: one binding
//! produces one command.

use crate::input::mode::InputMode;

/// Mode transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeCommand {
    Enter(InputMode),
    ExitToNormal,
    InsertBefore,
    InsertAfter,
    InsertAtLineStart,
    InsertAtLineEnd,
    InsertNewLineBelow,
    InsertNewLineAbove,
    OpenBelow,
    OpenAbove,
}
