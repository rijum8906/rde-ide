//! The command hierarchy.
//!
//! [`Command`] is the single top-level action type. Each variant wraps a
//! category enum (`SessionCommand`, `MotionCommand`, ...), so a dispatcher can
//! match on broad intent first and delegate to a focused handler that only
//! receives the state it needs.
//!
//! Every category lives in its own submodule and is re-exported here, so
//! callers only need `editor_core::command::*`.

pub mod command_line;
pub mod edit;
pub mod layout;
pub mod mode;
pub mod motion;
pub mod session;
pub mod workspace;

pub use command_line::CommandLineCommand;
pub use edit::EditCommand;
pub use layout::LayoutCommand;
pub use mode::ModeCommand;
pub use motion::MotionCommand;
pub use session::SessionCommand;
pub use workspace::WorkspaceCommand;

/// The top-level command type. Every action the editor can take is one
/// of these variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Application lifecycle: quit, save, open files.
    Session(SessionCommand),

    /// Workspace, tab, and buffer management.
    Workspace(WorkspaceCommand),

    /// In-buffer cursor movement and text motions.
    Motion(MotionCommand),

    /// Mode transitions (Normal, Insert, Visual, etc.).
    Mode(ModeCommand),

    /// Layout changes (splits, panel visibility, fullscreen).
    Layout(LayoutCommand),

    /// Mutating text in the buffer (insert, delete, undo/redo).
    Edit(EditCommand),

    /// Command-line mode inputs and prompts.
    CommandLine(CommandLineCommand),
}
