//! Signs: positive or negative.

use array_map::Indexable;
use serde::{Deserialize, Serialize};

/// Enumeration over possible signs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[repr(u8)]
#[derive(Indexable)]
pub enum Sign {
    /// Positive.
    Pos,
    /// Negative.
    Neg,
}

impl Sign {
    /// Converts this sign to +1 (if positive) or -1 (if negative).
    #[inline]
    pub const fn as_int(&self) -> i64 {
        match self {
            Self::Pos => 1,
            Self::Neg => -1,
        }
    }

    /// Returns true if the sign is positive.
    #[inline]
    pub const fn is_pos(&self) -> bool {
        matches!(self, Sign::Pos)
    }

    /// Returns true if the sign is negative.
    #[inline]
    pub const fn is_neg(&self) -> bool {
        matches!(self, Sign::Neg)
    }
}

impl std::ops::Not for Sign {
    type Output = Self;
    /// Flips the [`Sign`].
    fn not(self) -> Self::Output {
        match self {
            Self::Pos => Self::Neg,
            Self::Neg => Self::Pos,
        }
    }
}
