use thiserror::Error;

/// Errors from text-buffer operations (indexing, slicing, editing).
///
/// All byte offsets are absolute byte positions in the document, not
/// line-relative. All "length" values are the current document length in
/// bytes at the time of the failing operation.
#[derive(Debug, Error)]
pub enum TextError {
    // -----------------------------------------------------------------------
    // Indexing / range
    // -----------------------------------------------------------------------
    /// A byte, char, line, or piece index was outside the valid range.
    #[error("index {index} out of bounds for length {len}")]
    IndexOutOfBounds { index: usize, len: usize },

    /// A `Range<usize>` was malformed (`start > end`) or exceeded `len`.
    #[error("invalid range {start}..{end} for length {len}")]
    InvalidRange {
        start: usize,
        end: usize,
        len: usize,
    },

    /// A byte offset was inside a multi-byte UTF-8 sequence.
    /// Slicing there would panic, so we return this instead.
    #[error("byte {byte} is not a UTF-8 char boundary")]
    NotACharBoundary { byte: usize },

    /// A line number was 0 (lines are 1-based) or exceeded `line_count()`.
    #[error("line {line} not found (buffer has {line_count} lines)")]
    LineNotFound { line: usize, line_count: usize },

    /// A char index was requested on a buffer whose char count exceeds
    /// `usize::MAX` (practically impossible, but keeps the contract total).
    #[error("char count overflow")]
    CharCountOverflow,

    // -----------------------------------------------------------------------
    // Editing
    // -----------------------------------------------------------------------
    /// Attempted to insert text at an offset that is not a char boundary
    /// or is past `len()`.
    #[error("cannot insert at byte {at}: {reason}")]
    InvalidInsertPosition { at: usize, reason: PositionError },

    /// Attempted to insert empty text.
    #[error("cannot insert empty text")]
    EmptyInsertText,

    /// Attempted to delete a range that is malformed, out of bounds,
    /// or not aligned to char boundaries.
    #[error("cannot delete {start}..{end}: {reason}")]
    InvalidDeleteRange {
        start: usize,
        end: usize,
        reason: PositionError,
    },

    // -----------------------------------------------------------------------
    // Loading / I/O
    // -----------------------------------------------------------------------
    /// Failed to read the file backing this buffer.
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// The file's bytes were not valid UTF-8.
    #[error("invalid UTF-8 in {path} at byte {valid_up_to}")]
    InvalidUtf8 { path: String, valid_up_to: usize },
}

/// Why an edit position/range was rejected. Kept as a separate enum so
/// `TextError` can format a single message and so callers can match on the
/// *reason* without matching on the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionError {
    OutOfBounds,
    NotACharBoundary,
    /// `start > end`
    Reversed,
}

/// Which indexing scheme the caller used. Included in the error message so
/// "index 5 out of bounds" is actionable ("was that a byte or a line?").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    Byte,
    Char,
    Line,
    Piece,
}

impl std::fmt::Display for PositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionError::OutOfBounds => f.write_str("out of bounds"),
            PositionError::NotACharBoundary => f.write_str("not a UTF-8 char boundary"),
            PositionError::Reversed => f.write_str("start > end"),
        }
    }
}

pub type TextResult<T> = Result<T, TextError>;
