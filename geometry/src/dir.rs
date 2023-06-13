//! Axis-aligned directions: horizontal or vertical.

use std::fmt::Display;

use array_map::Indexable;
use serde::{Deserialize, Serialize};

/// An enumeration of axis-aligned directions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[repr(u8)]
#[derive(Indexable)]
pub enum Dir {
    /// The horizontal, or x-aligned, direction.
    Horiz,
    /// The vertical, or y-aligned, direction.
    Vert,
}

impl Dir {
    /// Returns the other direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// assert_eq!(Dir::Vert.other(), Dir::Horiz);
    /// assert_eq!(Dir::Horiz.other(), Dir::Vert);
    /// ```
    pub const fn other(&self) -> Self {
        match *self {
            Self::Horiz => Self::Vert,
            Self::Vert => Self::Horiz,
        }
    }
}

impl Display for Dir {
    /// Displays the direction in a human-readable format.
    ///
    /// Currently, [`Dir::Horiz`] becomes `horizontal`;
    /// [`Dir::Vert`] becomes `vertical`.
    /// However, users should not rely on these particular strings.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// assert_eq!(format!("{}", Dir::Horiz), "horizontal");
    /// assert_eq!(format!("{}", Dir::Vert), "vertical");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Horiz => write!(f, "horizontal"),
            Self::Vert => write!(f, "vertical"),
        }
    }
}

impl std::ops::Not for Dir {
    type Output = Self;
    /// Returns the other direction.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// assert_eq!(!Dir::Vert, Dir::Horiz);
    /// assert_eq!(!Dir::Horiz, Dir::Vert);
    /// ```
    fn not(self) -> Self::Output {
        self.other()
    }
}
