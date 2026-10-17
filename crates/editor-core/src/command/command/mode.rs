use crate::input::mode::Mode;

/// Mode transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeCommand {
    Enter(Mode),
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
