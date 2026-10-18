//! Input Module
//!
//! This module helps in managing input methods and inputs

use crate::input::{keymap::Keymap, mode::InputMode, resolver::Resolver};

pub mod key;
pub mod keymap;
pub mod mode;
pub mod resolver;

#[derive(Debug, Default)]
pub struct Input {
    pub mode: InputMode,
    pub resolver: Resolver,
    pub key_map: Keymap,
}
