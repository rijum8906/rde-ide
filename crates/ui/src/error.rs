#[derive(Debug, thiserror::Error)]
pub enum UiError {
    /// Represents an error that occurs when the terminal screen size is too small.
    #[error("Screen size too small: minimum required is {0}x{1}, current size is {2}x{3}")]
    ScreenTooSmall(u16, u16, u16, u16),
}

/// A type alias for `Result<T, IdeError>`.
pub type Result<T> = std::result::Result<T, UiError>;
