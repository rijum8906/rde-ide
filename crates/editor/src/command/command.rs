//! Commands are the vocabulary of the editor.
//!
//! A `Command` is *what* should happen. Keys, macros, the command line,
//! and tests all produce `Command`s. The executor turns them into effects.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // --- Application lifecycle ------------------------------------------
    Quit,
    ForceQuit,
    SaveBuffer,
    SaveBufferAs(String),

    // --- Tab / buffer navigation ----------------------------------------
    NextTab,
    PrevTab,
    SelectTab(usize),
    CloseTab,
    NewScratchBuffer,

    // --- Mode transitions -----------------------------------------------
    EnterMode(Mode),
    ExitToNormal,

    // --- Layout ---------------------------------------------------------
    ToggleFullscreen,
    ToggleLeftPanel,
    ToggleRightPanel,

    // --- Cursor (stubs for later) ---------------------------------------
    MoveCursor(Direction),
    MoveToLineStart,
    MoveToLineEnd,
    MoveToFileStart,
    MoveToFileEnd,

    // --- Editing (stubs) ------------------------------------------------
    InsertChar(char),
    InsertNewline,
    DeleteChar,
    Backspace,
    Undo,
    Redo,

    // --- Command line ---------------------------------------------------
    OpenCommandLine,
    SubmitCommandLine,
    CancelCommandLine,
    CommandLinePush(char),
    CommandLinePop,
}

use crate::input::mode::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
