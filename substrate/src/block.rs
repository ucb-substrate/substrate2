//! A block that can be instantiated by Substrate.

use std::sync::Arc;
use std::{any::Any, hash::Hash};

use arcstr::ArcStr;
pub use codegen::Block;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::io::Io;

/// A block that can be instantiated by Substrate.
///
/// # Examples
///
#[doc = examples::get_snippets!("core", "inverter")]
pub trait Block: Serialize + DeserializeOwned + Hash + Eq + Send + Sync + Any {
    /// The ports of this block.
    type Io: Io;

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

impl<T: Block> Block for Arc<T> {
    type Io = T::Io;

    fn id() -> ArcStr {
        T::id()
    }

    fn name(&self) -> ArcStr {
        T::name(self)
    }

    fn io(&self) -> Self::Io {
        T::io(self)
    }
}
