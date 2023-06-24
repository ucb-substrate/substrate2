//! A block that can be instantiated by Substrate.

use std::{any::Any, hash::Hash};

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

/// A block that can be instantiated by Substrate.
pub trait Block: Serialize + Deserialize<'static> + Hash + Eq + Clone + Send + Sync + Any {
    // TODO: type Io: AnalogIO;

    /// A crate-wide unique identifier for this block.
    fn id() -> ArcStr;

    /// A name for a specific parametrization of this block.
    ///
    /// Used to choose the default cell name during generation.
    fn name(&self) -> ArcStr {
        arcstr::literal!("unnamed")
    }

    // TODO: fn io(&self) -> Self::Io;
}
