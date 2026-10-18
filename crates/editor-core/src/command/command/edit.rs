use crate::command::command::MotionCommand;

/// Editing the buffer contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditCommand {
    InsertChar(char),
    InsertNewline,
    InsertTab,
    DeleteChar,
    Backspace,
    DeleteWord,
    DeleteToLineEnd,
    DeleteLine,
    DeleteToFileEnd,

    // Motion-prefixed variants.
    DeleteMotion(MotionCommand),
    ChangeMotion(MotionCommand),
    YankMotion(MotionCommand),

    // Clipboard.
    Yank,
    Paste,
    PasteBefore,

    // Undo history.
    Undo,
    Redo,
    Repeat,
}
