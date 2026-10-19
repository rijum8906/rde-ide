//! View model for the floating command palette.
//!
//! A snapshot of the command-line editor: the prompt, the text typed so far,
//! and the caret position. The palette is only built while the editor is in
//! command mode; it borrows the state so no text is copied per frame.

/// Everything the command palette needs to draw its input row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandPaletteView<'a> {
    /// Prompt shown before the input, e.g. `:`.
    pub prompt: &'a str,

    /// Current command-line text.
    pub text: &'a str,

    /// Caret position inside `text`, measured in characters (not bytes).
    pub cursor: usize,
}
