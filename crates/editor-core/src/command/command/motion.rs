/// Cursor movement. Every variant is a pure `(line, col) -> new_pos`
/// operation, applied within the current buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionCommand {
    // Single-step.
    Left,
    Right,
    Up,
    Down,

    // Line jumps.
    LineStart,
    LineEnd,
    FirstNonBlank,
    LineNumber(usize),

    // File jumps.
    FileStart,
    FileEnd,

    // Word movement.
    NextWordStart,
    PrevWordStart,
    WordEnd,
    NextBigWordStart,
    PrevBigWordStart,

    // Paragraph jumps.
    NextParagraphStart,
    PrevParagraphStart,

    // Page jumps.
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,

    // Screen-relative jumps (like Vim's H, M, L).
    ScreenTop,
    ScreenMiddle,
    ScreenBottom,

    // Bracket matching.
    MatchBracket,
}
