//! An `Opacity` represents something that is either opaque to users (`Opaque`),
//! or clear for users to inspect (`Clear`).
use enumify::enumify;
use serde::{Deserialize, Serialize};

/// Something that may be opaque or clear.
///
/// Often used to represent something that may or may not be a blackbox.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[enumify]
pub enum Opacity<O, C> {
    /// An item whose contents cannot be inspected except in summary form as type `O`.
    Opaque(O),

    /// An item whose contents are visible to users as type `C`.
    Clear(C),
}
