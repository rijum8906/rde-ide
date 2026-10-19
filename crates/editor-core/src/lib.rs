//! Editor core: the UI-agnostic command and input model.
//!
//! - [`command`] defines every action the editor can perform.
//! - [`input`] turns key presses into those actions.
//!
//! The crate knows nothing about buffers, rendering, or the filesystem, so the
//! same model can be reused by any frontend.

pub mod command;
pub mod input;
