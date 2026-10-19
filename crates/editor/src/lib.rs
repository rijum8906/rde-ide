//! Editor policy.
//!
//! Turns [`Command`](editor_core::command::Command)s into changes on the
//! domain state. The crate has two layers:
//!
//! - [`execute`] dispatches a command to the handler for its category.
//! - `mutation` (crate-private) contains the low-level buffer and cursor
//!   operations those handlers reuse.

pub mod execute;
pub(crate) mod mutation;
