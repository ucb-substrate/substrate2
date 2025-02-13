use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod conv;
pub mod export;
pub mod import;

#[cfg(test)]
mod tests;

/// A GDS layer specification.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct GdsLayer(pub u16, pub u16);

impl Display for GdsLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}
