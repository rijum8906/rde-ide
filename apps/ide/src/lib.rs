//! Application shell for the IDE binary.
//!
//! This crate is the composition root: it depends on the domain, editor, and
//! TUI crates and wires them together. Keeping the shell here (rather than in
//! `editor`) means none of the library crates has to know about the others'
//! runtime concerns.

pub mod app;
