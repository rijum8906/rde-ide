//! Command-line (`:`) mode actions.

/// Editing the `:` command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandLineCommand {
    Open,
    Submit,
    Cancel,
    Push(char),
    Pop,
    Clear,
    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    HistoryPrev,
    HistoryNext,
}
