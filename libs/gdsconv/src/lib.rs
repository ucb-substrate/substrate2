use serde::{Deserialize, Serialize};

pub mod export;
pub mod import;

#[cfg(test)]
mod tests;

/// A GDS layer specification.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct GdsLayer(pub u16, pub u16);
