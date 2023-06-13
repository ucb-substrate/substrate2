//! Describes the corners of axis-aligned rectangles.
//!
//! # Examples
//!
//! You can access the corners of a [`Rect`](crate::rect::Rect):
//!
//! ```
//! # use geometry::prelude::*;
//! let rect = Rect::from_sides(10, 20, 30, 40);
//! assert_eq!(rect.corner(Corner::LowerRight), Point::new(30, 20));
//! ```
//!
//! You can also increase the size of a [`Rect`](crate::rect::Rect) by pushing out one of its corners:
//!
//! ```
//! # use geometry::prelude::*;
//! let rect = Rect::from_sides(10, 20, 30, 40);
//! assert_eq!(rect.expand_corner(Corner::UpperLeft, 100), Rect::from_sides(-90, 20, 30, 140));
//! ```

use array_map::Indexable;
use serde::{Deserialize, Serialize};

use crate::dir::Dir;
use crate::side::Side;

/// An enumeration of the corners of an axis-aligned rectangle.
///
/// See the [module-level documentation](crate::corner) for examples.
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
