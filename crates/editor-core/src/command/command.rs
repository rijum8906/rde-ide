// crates/editor-core/src/command/mod.rs

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

    /// Workspace, tab, and buffer management.x
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
