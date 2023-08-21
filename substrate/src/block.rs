//! A block that can be instantiated by Substrate.

use std::{any::Any, hash::Hash};

use arcstr::ArcStr;
pub use codegen::Block;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::io::Io;
use crate::sealed;
use crate::sealed::Token;

pub trait BlockKind {
    const FLATTEN: bool;

    #[doc(hidden)]
    fn sealed(_: sealed::Token);
}

pub trait ScirKind: BlockKind {
    #[doc(hidden)]
    fn sealed(_: sealed::Token);
}
pub struct Cell;
impl BlockKind for Cell {
    const FLATTEN: bool = false;
    fn sealed(_: Token) {}
}

pub struct InlineCell;
impl BlockKind for InlineCell {
    const FLATTEN: bool = true;
    fn sealed(_: Token) {}
}

pub struct Scir;
impl BlockKind for Scir {
    const FLATTEN: bool = false;
    fn sealed(_: Token) {}
}
impl ScirKind for Scir {
    fn sealed(_: Token) {}
}

pub struct InlineScir;
impl BlockKind for InlineScir {
    const FLATTEN: bool = true;
    fn sealed(_: Token) {}
}
impl ScirKind for InlineScir {
    fn sealed(_: Token) {}
}

pub struct SchemaPrimitive;
impl BlockKind for SchemaPrimitive {
    const FLATTEN: bool = true;
    fn sealed(_: Token) {}
}

pub struct PdkPrimitive;
impl BlockKind for PdkPrimitive {
    const FLATTEN: bool = true;
    fn sealed(_: Token) {}
}

pub struct Opaque;
impl BlockKind for Opaque {
    const FLATTEN: bool = false;
    fn sealed(_: Token) {}
}

/// A block that can be instantiated by Substrate.
///
/// # Examples
///
#[doc = examples::get_snippets!("core", "inverter")]
pub trait Block: Serialize + DeserializeOwned + Hash + Eq + Send + Sync + Any {
    /// The kind of this block.
    type Kind: BlockKind;
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
