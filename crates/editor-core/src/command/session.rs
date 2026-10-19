//! Application lifecycle: quit, save, open.

/// Application lifecycle: quit, save, open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    Quit,
    ForceQuit,
    SaveBuffer,
    SaveBufferAs(String),
    SaveAll,
    OpenFile(String),
    OpenFileTree,
    NewScratchBuffer,
}
