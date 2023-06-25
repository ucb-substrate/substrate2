//! A block that can be instantiated by Substrate.

use std::{any::Any, hash::Hash};

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

/// A block that can be instantiated by Substrate.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/inverter.md")]
/// ```
pub trait Block: Serialize + Deserialize<'static> + Hash + Eq + Clone + Send + Sync + Any {
    // TODO: type Io: AnalogIO;

    /// A crate-wide unique identifier for this block.
    fn id() -> ArcStr;

    /// A name for a specific parametrization of this block.
    ///
    /// Instances of this block will initially be assigned this name,
    /// although Substrate may need to change the name
    /// (e.g. to avoid duplicates).
    fn name(&self) -> ArcStr {
        arcstr::literal!("unnamed")
    }

    // TODO: fn io(&self) -> Self::Io;
}