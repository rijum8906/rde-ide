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
