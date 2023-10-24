//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
mod data;
pub mod layers;

use std::any::Any;
use std::sync::Arc;

use rust_decimal::Decimal;
use substrate::schematic::{Instance, SchematicData};
use type_dispatch::impl_dispatch;

use crate::block::{self, Block};
use crate::error::{Error, Result};
use crate::io::{LayoutType, SchematicType};
use crate::layout::{CellBuilder as LayoutCellBuilder, ExportsLayoutData, Layout};
use crate::schematic::schema::{Primitive, Schema};
use crate::schematic::{Cell, CellBuilder, ExportsNestedData, RawCell, ScirCell};
use crate::sealed;
use crate::sealed::Token;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The type representing a corner in this PDK.
    type Corner: Corner;
    /// The layout database unit for this PDK.
    const LAYOUT_DB_UNITS: Option<Decimal> = None;
}

/// The type of a PDK's layer set.
pub type PdkLayers<PDK> = <PDK as Pdk>::Layers;

/// The type of a PDK's corners.
pub type PdkCorner<PDK> = <PDK as Pdk>::Corner;

/// A PDK that has a layout for block `B`.
///
/// This trait is intended to be used to impose bounds on supported PDKs based
/// on blocks that they have layouts for.
///
/// Automatically implemented for blocks that implement [`Layout<PDK>`] and
/// cannot be implemented outside of Substrate.
pub trait HasLayout<B: ExportsLayoutData>: Pdk + Sized {
    /// Generates the block's layout by running [`Layout::layout`].
    #[doc(hidden)]
    fn layout(
        block: &B,
        io: &mut <<B as Block>::Io as LayoutType>::Builder,
        cell: &mut LayoutCellBuilder<Self, B>,
        _: sealed::Token,
    ) -> Result<B::Data>;
}

impl<PDK: Pdk, B: Layout<PDK>> HasLayout<B> for PDK {
    fn layout(
        block: &B,
        io: &mut <<B as Block>::Io as LayoutType>::Builder,
        cell: &mut LayoutCellBuilder<Self, B>,
        _: sealed::Token,
    ) -> Result<B::Data> {
        block.layout(io, cell)
    }
}
