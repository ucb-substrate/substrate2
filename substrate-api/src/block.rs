//! A block that can be instantiated by Substrate.

use std::{any::Any, hash::Hash};

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

use crate::schematic::HardwareType;

/// A block that can be instantiated by Substrate.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/inverter.md")]
/// ```
pub trait Block: Serialize + Deserialize<'static> + Hash + Eq + Clone + Send + Sync + Any {
    /// The ports of this block.
    type Io: AnalogIo;

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

    /// Returns a fully-specified instance of this cell's `Io`.
    fn io(&self) -> Self::Io;
}

/// A trait implemented by block input/output interfaces.
pub trait AnalogIo: HardwareType {
    // TODO
}

/// Blocks with no ports can declare their `Io` as `()`.
impl AnalogIo for () {}
