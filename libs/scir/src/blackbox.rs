//! APIs for defining blackbox primitives.

use crate::slice::NamedSlice;
use serde::{Deserialize, Serialize};

/// The contents of a blackbox cell.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BlackboxContents {
    /// The list of [`BlackboxElement`]s comprising this cell.
    ///
    /// During netlisting, each blackbox element will be
    /// injected into the final netlist.
    /// Netlister implementations should add spaces before each element
    /// in the list, except for the first element.
    pub elems: Vec<BlackboxElement>,
}

/// An element in the contents of a blackbox cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlackboxElement {
    /// A reference to a [`Slice`].
    Slice(NamedSlice),
    /// A raw, opaque [`String`].
    RawString(String),
}
impl FromIterator<BlackboxElement> for BlackboxContents {
    fn from_iter<T: IntoIterator<Item = BlackboxElement>>(iter: T) -> Self {
        Self {
            elems: iter.into_iter().collect(),
        }
    }
}

impl From<String> for BlackboxElement {
    #[inline]
    fn from(value: String) -> Self {
        Self::RawString(value)
    }
}

impl From<&str> for BlackboxElement {
    #[inline]
    fn from(value: &str) -> Self {
        Self::RawString(value.to_string())
    }
}

impl From<NamedSlice> for BlackboxElement {
    #[inline]
    fn from(value: NamedSlice) -> Self {
        Self::Slice(value)
    }
}

impl From<String> for BlackboxContents {
    fn from(value: String) -> Self {
        Self {
            elems: vec![BlackboxElement::RawString(value)],
        }
    }
}
