//! An `Opacity` represents something that is either opaque to users (`Opaque`),
//! or clear for users to inspect (`Clear`).
use serde::{Deserialize, Serialize};

/// Something that may be opaque or clear.
///
/// Often used to represent something that may or may not be a blackbox.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Opacity<O, C> {
    /// An item whose contents cannot be inspected except in summary form as type `O`.
    Opaque(O),

    /// An item whose contents are visible to users as type `C`.
    Clear(C),
}

impl<O, C> Opacity<O, C> {
    /// Returns the opaque data stored in this [`Opacity`].
    ///
    /// # Panics
    ///
    /// Panics if the opacity is not opaque.
    pub fn unwrap_opaque(self) -> O {
        match self {
            Self::Opaque(o) => o,
            _ => panic!("cannot unwrap non-opaque opacity as opaque"),
        }
    }

    /// Returns the clear data stored in this [`Opacity`].
    ///
    /// # Panics
    ///
    /// Panics if the opacity is not clear.
    pub fn unwrap_clear(self) -> C {
        match self {
            Self::Clear(c) => c,
            _ => panic!("cannot unwrap non-clear opacity as clear"),
        }
    }

    /// Returns `true` if this opacity is opaque.
    #[inline]
    pub fn is_opaque(&self) -> bool {
        matches!(self, Self::Opaque(_))
    }

    /// Returns `true` if this opacity is clear.
    #[inline]
    pub fn is_clear(&self) -> bool {
        matches!(self, Self::Clear(_))
    }

    /// Converts `Opacity<O, C>` to `Opacity<&O, &C>`.
    pub const fn as_ref(&self) -> Opacity<&O, &C> {
        match *self {
            Opacity::Opaque(ref o) => Opacity::Opaque(o),
            Opacity::Clear(ref c) => Opacity::Clear(c),
        }
    }

    /// Converts `&mut Opacity<O, C>` to `Opacity<&mut O, &mut C>`.
    pub fn as_mut(&mut self) -> Opacity<&mut O, &mut C> {
        match *self {
            Opacity::Opaque(ref mut o) => Opacity::Opaque(o),
            Opacity::Clear(ref mut c) => Opacity::Clear(c),
        }
    }
}
