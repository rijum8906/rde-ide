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

    /// Represents an unknown error.
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

/// A type alias for `Result<T, IdeError>`.
pub type Result<T> = std::result::Result<T, IdeError>;
