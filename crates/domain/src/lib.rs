//! Core domain types shared by the whole project.
//!
//! - [`error`] — the unified error types (`IdeError`/`DomainError`).
//! - [`buffer`] — piece-table-backed documents and their save state.
//! - [`workspace`] — the set of open buffers and the active selection.
//!
//! The crate is UI-agnostic: it performs file I/O but no terminal or
//! rendering work.

pub mod buffer;
pub mod error;
pub mod workspace;
