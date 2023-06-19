//! The sides of an axis-aligned rectangle.

use crate::dir::Dir;
use crate::sign::Sign;
use array_map::{ArrayMap, Indexable};
use serde::{Deserialize, Serialize};

/// An enumeration of the sides of an axis-aligned rectangle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[repr(u8)]
#[derive(Indexable)]
pub enum Side {
    /// The left side.
    Left,
    /// The bottom side.
    Bot,
    /// The right side.
    Right,
    /// The top side.
    Top,
}

impl Side {
    /// Gets the direction of the coordinate corresponding to this side.
    ///
    /// Top and bottom edges are y-coordinates, so they are on the **vertical** axis.
    /// Left and right edges are x-coordinates, so they are on the **horizontal** axis.
    ///
    /// Also see [`Side::edge_dir`].
    pub fn coord_dir(&self) -> Dir {
        use Dir::*;
        use Side::*;
        match self {
            Top | Bot => Vert,
            Left | Right => Horiz,
        }
    }

    /// Gets the direction of the edge corresponding to this side.
    ///
    /// Top and bottom edges are **horizontal** line segments;
    /// left and right edges are **vertical** line segments.
    ///
    /// Also see [`Side::coord_dir`].
    pub fn edge_dir(&self) -> Dir {
        use Dir::*;
        use Side::*;
        match self {
            Top | Bot => Horiz,
            Left | Right => Vert,
        }
    }

    /// Returns the opposite direction.
    pub fn other(&self) -> Self {
        match self {
            Side::Top => Side::Bot,
            Side::Right => Side::Left,
            Side::Bot => Side::Top,
            Side::Left => Side::Right,
        }
    }

    /// Returns the sign corresponding to moving towards this side.
    pub fn sign(&self) -> Sign {
        use Side::*;
        use Sign::*;
        match self {
            Top | Right => Pos,
            Bot | Left => Neg,
        }
    }

    /// Returns the side corresponding with the given [`Dir`] and [`Sign`].
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// assert_eq!(Side::with_dir_and_sign(Dir::Horiz, Sign::Neg), Side::Left);
    /// assert_eq!(Side::with_dir_and_sign(Dir::Vert, Sign::Neg), Side::Bot);
    /// assert_eq!(Side::with_dir_and_sign(Dir::Horiz, Sign::Pos), Side::Right);
    /// assert_eq!(Side::with_dir_and_sign(Dir::Vert, Sign::Pos), Side::Top);
    /// ```
    pub fn with_dir_and_sign(dir: Dir, sign: Sign) -> Side {
        match dir {
            Dir::Horiz => match sign {
                Sign::Pos => Side::Right,
                Sign::Neg => Side::Left,
            },
            Dir::Vert => match sign {
                Sign::Pos => Side::Top,
                Sign::Neg => Side::Bot,
            },
        }
    }

    /// Returns sides that bound the given direction.
    ///
    /// Users should not rely upon the order of the sides returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// assert_eq!(Side::with_dir(Dir::Horiz), [Side::Left, Side::Right]);
    /// assert_eq!(Side::with_dir(Dir::Vert), [Side::Bot, Side::Top]);
    /// ```
    pub fn with_dir(dir: Dir) -> [Side; 2] {
        match dir {
            Dir::Horiz => [Side::Left, Side::Right],
            Dir::Vert => [Side::Bot, Side::Top],
        }
    }
}

impl std::ops::Not for Side {
    type Output = Self;
    /// Exclamation Operator returns the opposite direction
    fn not(self) -> Self::Output {
        self.other()
    }
}
/// An association of a value with type `T` to each of the four [`Side`]s.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sides<T> {
    inner: ArrayMap<Side, T, 4>,
}

impl<T> Sides<T>
where
    T: Clone,
{
    /// Creates a new [`Sides`] with `value` associated with all sides.
    ///
    /// The value will be cloned for each [`Side`].
    ///
    /// If your value is [`Copy`], consider using [`Sides::uniform`] instead.
    pub fn uniform_cloned(value: T) -> Self {
        Self {
            inner: ArrayMap::from_value(value),
        }
    }
}

impl<T> Sides<T>
where
    T: Copy,
{
    /// Creates a new [`Sides`] with `value` associated with all sides.
    pub const fn uniform(value: T) -> Self {
        Self {
            inner: ArrayMap::new([value; 4]),
        }
    }
}

impl<T> Sides<T> {
    /// Creates a new [`Sides`] with with the provided values for each side.
    pub const fn new(left: T, bot: T, right: T, top: T) -> Self {
        // IMPORTANT: the ordering of array elements here must match
        // the ordering of variants in the [`Side`] enum.
        Self {
            inner: ArrayMap::new([left, bot, right, top]),
        }
    }

    /// Maps a function over the provided [`Sides`], returning a new [`Sides`].
    pub fn map<B>(self, f: impl FnMut(&Side, T) -> B) -> Sides<B> {
        Sides {
            inner: self.inner.map(f),
        }
    }
}

impl<T> std::ops::Index<Side> for Sides<T> {
    type Output = T;
    fn index(&self, index: Side) -> &Self::Output {
        self.inner.index(index)
    }
}

impl<T> std::ops::IndexMut<Side> for Sides<T> {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        self.inner.index_mut(index)
    }
}
