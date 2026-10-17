// crates/editor-core/src/command/mod.rs

pub mod command_line;
pub mod edit;
pub mod layout;
pub mod mode;
pub mod motion;
pub mod navigation;
pub mod session;

pub use command_line::CommandLineCommand;
pub use edit::EditCommand;
pub use layout::LayoutCommand;
pub use mode::ModeCommand;
pub use motion::Motion;
pub use navigation::NavigationCommand;
pub use session::SessionCommand;

/// The top-level command type. Every action the editor can take is one
/// of these variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Application lifecycle: quit, save, open.
    Session(SessionCommand),

    /// Tab and buffer navigation.
    Navigation(NavigationCommand),

    /// Cursor movement.
    Motion(Motion),

    /// Mode transitions (Normal/Insert/Visual/Command).
    Mode(ModeCommand),

    /// Layout changes (fullscreen, panel visibility).
    Layout(LayoutCommand),

    /// Editing the buffer.
    Edit(EditCommand),

    /// Command-line editing.
    CommandLine(CommandLineCommand),
}
