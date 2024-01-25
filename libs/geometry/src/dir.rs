//! Axis-aligned directions: horizontal or vertical.

use std::fmt::Display;

use array_map::{ArrayMap, Indexable};
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

/// An association of a value with type `T` to each of the two [`Dir`]s.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Dirs<T> {
    inner: ArrayMap<Dir, T, 2>,
}

impl<T> Dirs<T>
where
    T: Clone,
{
    /// Creates a new [`Dirs`] with `value` associated with all directions.
    ///
    /// The value will be cloned for each [`Dir`].
    ///
    /// If your value is [`Copy`], consider using [`Dirs::uniform`] instead.
    pub fn uniform_cloned(value: T) -> Self {
        Self {
            inner: ArrayMap::from_value(value),
        }
    }
}

impl<T> Dirs<T>
where
    T: Copy,
{
    /// Creates a new [`Dir`] with `value` associated with all directions.
    pub const fn uniform(value: T) -> Self {
        Self {
            inner: ArrayMap::new([value; 2]),
        }
    }
}

impl<T> Dirs<T> {
    /// Creates a new [`Dir`] with with the provided values for each direction.
    pub const fn new(horiz: T, vert: T) -> Self {
        // IMPORTANT: the ordering of array elements here must match
        // the ordering of variants in the [`Dir`] enum.
        Self {
            inner: ArrayMap::new([horiz, vert]),
        }
    }

    /// Maps a function over the provided [`Dir`], returning a new [`Dir`].
    pub fn map<B>(self, f: impl FnMut(&Dir, T) -> B) -> Dirs<B> {
        Dirs {
            inner: self.inner.map(f),
        }
    }
}

impl<T> std::ops::Index<Dir> for Dirs<T> {
    type Output = T;
    fn index(&self, index: Dir) -> &Self::Output {
        self.inner.index(index)
    }
}

impl<T> std::ops::IndexMut<Dir> for Dirs<T> {
    fn index_mut(&mut self, index: Dir) -> &mut Self::Output {
        self.inner.index_mut(index)
    }
}
