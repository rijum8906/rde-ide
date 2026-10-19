//! Buffer and cursor mutations shared by the command handlers.
//!
//! These functions know how to translate between cursor positions and buffer
//! offsets, but they never decide *when* to run: that is the job of
//! [`crate::execute`].

pub(crate) mod edit;
pub(crate) mod goto;
pub(crate) mod text;
