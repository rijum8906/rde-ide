//! Error types for this project.
//!
//! This module defines the error types used throughout the project.
//! It provides a unified way to handle errors and propagate them through the application.

mod text;

use std::path::PathBuf;

pub use text::{IndexKind, PositionError, TextError, TextResult};

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

    #[error("IO error: {0}")]
    DomainError(#[from] DomainError),

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

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    // --- Paths ---
    #[error("path does not exist: {path}")]
    PathNotFound { path: PathBuf },

    #[error("path is neither a file nor a directory: {path}")]
    UnsupportedPath { path: PathBuf },

    // --- File I/O ---
    #[error("cannot read file '{path}': {source}")]
    CannotReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("cannot write file '{path}': {source}")]
    CannotWriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("cannot rename '{from}' to '{to}': {source}")]
    CannotRename {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("file '{path}' contains invalid UTF-8")]
    InvalidUtf8 { path: PathBuf },

    #[error("buffer has no associated path")]
    NoPath,

    // --- Buffer / piece-table ---
    /// A text-buffer edit was rejected (bad offset, range, or encoding).
    #[error("text edit failed: {0}")]
    Text(#[from] TextError),

    #[error("offset {offset} is out of bounds (buffer length is {len})")]
    OffsetOutOfBounds { offset: usize, len: usize },

    #[error("offset {offset} is not on a UTF-8 char boundary")]
    NotCharBoundary { offset: usize },

    #[error("invalid range: start ({start}) exceeds end ({end})")]
    InvalidRange { start: usize, end: usize },

    // --- Workspace ---
    #[error("no buffer at index {index} (workspace has {count} buffers)")]
    NoSuchBuffer { index: usize, count: usize },

    #[error("cannot close the last buffer")]
    LastBuffer,

    #[error("no active buffer")]
    NoActiveBuffer,
}

pub type DomainResult<T> = std::result::Result<T, DomainError>;

/// A type alias for `Result<T, IdeError>`.
pub type Result<T> = std::result::Result<T, IdeError>;
