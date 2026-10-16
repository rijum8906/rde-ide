//! Piece Table
//!
//! this module contains [`PieceTable`]

use crate::error::{IdeError, Result};

/// contains everything
#[derive(Debug, Clone)]
pub struct PieceTable {}

impl Default for PieceTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceTable {
    pub fn new() -> Self {
        Self {}
    }
}

impl std::str::FromStr for PieceTable {
    type Err = IdeError;
    fn from_str(s: &str) -> Result<Self> {
        let _ = s;
        Ok(Self {})
    }
}

impl PieceTable {
    pub fn insert(&mut self, _offset: usize, _contents: &str) -> Result<()> {
        todo!()
    }

    pub fn delete(&mut self, _range: core::ops::Range<usize>) -> Result<()> {
        todo!()
    }

    pub fn line_count(&self) -> usize {
        // Stub implementation: return 1 until full piece table line indexing is implemented
        1
    }

    pub fn get_line_contents(&self, _idx: usize) -> Option<&str> {
        None
    }
}
