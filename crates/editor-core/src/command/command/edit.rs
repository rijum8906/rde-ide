use crate::command::command::Motion;

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
    DeleteMotion(Motion),
    ChangeMotion(Motion),
    YankMotion(Motion),

    // Clipboard.
    Yank,
    Paste,
    PasteBefore,

    // Undo history.
    Undo,
    Redo,
    Repeat,
}
