use array_map::Indexable;
/// An enumeration of the corners of an axis-aligned rectangle.
use serde::{Deserialize, Serialize};

use crate::dir::Dir;
use crate::side::Side;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[repr(u8)]
#[derive(Indexable)]
pub enum Corner {
    /// The lower-left corner.
    LowerLeft,
    /// The lower-right corner.
    LowerRight,
    /// The upper-left corner.
    UpperLeft,
    /// The upper-right corner.
    UpperRight,
}

impl Corner {
    /// Gets the [`Side`] corresponding to the given [`Dir`] for this corner.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// assert_eq!(Corner::LowerLeft.side(Dir::Horiz), Side::Left);
    /// assert_eq!(Corner::LowerLeft.side(Dir::Vert), Side::Bot);
    /// assert_eq!(Corner::LowerRight.side(Dir::Horiz), Side::Right);
    /// assert_eq!(Corner::LowerRight.side(Dir::Vert), Side::Bot);
    /// assert_eq!(Corner::UpperLeft.side(Dir::Horiz), Side::Left);
    /// assert_eq!(Corner::UpperLeft.side(Dir::Vert), Side::Top);
    /// assert_eq!(Corner::UpperRight.side(Dir::Horiz), Side::Right);
    /// assert_eq!(Corner::UpperRight.side(Dir::Vert), Side::Top);
    /// ```
    pub fn side(&self, dir: Dir) -> Side {
        use Corner::*;
        use Dir::*;
        use Side::*;
        match dir {
            Horiz => match self {
                LowerLeft | UpperLeft => Left,
                LowerRight | UpperRight => Right,
            },
            Vert => match self {
                LowerLeft | LowerRight => Bot,
                UpperLeft | UpperRight => Top,
            },
        }
    }
}
