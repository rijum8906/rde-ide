//! Error types for this project.
//!
//! This module defines the error types used throughout the project.
//! It provides a unified way to handle errors and propagate them through the application.

/// The unified error type for the IDE project.
///
/// Use `IdeError` to represent errors that can occur in the IDE.
/// This enum provides variants for common error scenarios,
/// such as file not found, IO errors, network errors, and parsing errors. It also includes a variant for unknown errors.
#[derive(Debug, thiserror::Error)]
pub enum IdeError {
    /// Represents an error that occurs when a file is not found.
    #[error("File not found: {0}")]
    FileNotFound(std::path::PathBuf),

    /// The requested path does not exist.
    #[error("Path not found: {0}")]
    PathNotFound(std::path::PathBuf),

    /// The file could not be read.
    #[error("Cannot read: {0}")]
    CannotReadFile(std::path::PathBuf),

    #[error("Invalid utf8 in file {0}")]
    InvalidUtf8(std::path::PathBuf),

    /// Represents an error that occurs when an underlying IO error happens.
    /// This automatically generates From<std::io::Error>
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Represents an error that occurs when a network request fails.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Represents an error that occurs when parsing data fails.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Ui error
    #[error("Ui error: {0}")]
    AppError(String),

    /// Represents an error that occurs when the terminal screen size is too small.
    #[error(
        "Screen size too small: minimum required is {min_w}x{min_h}, current size is {cur_w}x{cur_h}"
    )]
    ScreenTooSmall {
        min_w: u16,
        min_h: u16,
        cur_w: u16,
        cur_h: u16,
    },

    /// Represents an unknown error.
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

/// A type alias for `Result<T, IdeError>`.
pub type Result<T> = std::result::Result<T, IdeError>;
