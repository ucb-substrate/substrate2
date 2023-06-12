//! Axis-aligned directions: horizontal or vertical.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// An enumeration of axis-aligned directions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Dir {
    /// The horizontal, or x-aligned, direction.
    Horiz,
    /// The vertical, or y-aligned, direction.
    Vert,
}

impl Dir {
    pub fn other(&self) -> Self {
        match *self {
            Self::Horiz => Self::Vert,
            Self::Vert => Self::Horiz,
        }
    }
}

impl Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Horiz => write!(f, "horizontal"),
            Self::Vert => write!(f, "vertical"),
        }
    }
}

impl std::ops::Not for Dir {
    type Output = Self;
    /// The opposite direction.
    fn not(self) -> Self::Output {
        self.other()
    }
}
