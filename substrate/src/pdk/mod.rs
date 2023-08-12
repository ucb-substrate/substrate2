//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
pub mod data;
pub mod layers;

use std::any::Any;

use arcstr::ArcStr;
use rust_decimal::Decimal;

use crate::block::Block;
use crate::error::Result;
use crate::io::{LayoutType, SchematicType};
use crate::layout::{CellBuilder as LayoutCellBuilder, ExportsLayoutData, Layout};
use crate::schematic::{CellBuilder as SchematicCellBuilder, ExportsSchematicData, Schematic};
use crate::sealed;

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

    /// The names of all schematic primitives in the PDK.
    ///
    /// This should include the names of transistors,
    /// resistors, capacitors, inductors, etc.
    ///
    /// The default implementation returns an empty list.
    fn schematic_primitives(&self) -> Vec<ArcStr> {
        Vec::new()
    }
}

/// A PDK that has a schematic for block `B`.
///
/// This trait is intended to be used to impose bounds on supported PDKs based
/// on blocks that they have schematics for.
///
/// Automatically implemented for blocks that implement [`Schematic<PDK>`] and
/// cannot be implemented outside of Substrate.
pub trait HasSchematic<B: ExportsSchematicData>: Pdk + Sized {
    /// Generates the block's schematic by running [`Schematic::schematic`].
    #[doc(hidden)]
    fn schematic(
        block: &B,
        io: &mut <<B as Block>::Io as SchematicType>::Bundle,
        cell: &mut SchematicCellBuilder<Self, B>,
        _: sealed::Token,
    ) -> Result<B::Data>;
}

impl<PDK: Pdk, B: Schematic<PDK>> HasSchematic<B> for PDK {
    fn schematic(
        block: &B,
        io: &mut <<B as Block>::Io as SchematicType>::Bundle,
        cell: &mut SchematicCellBuilder<Self, B>,
        _: sealed::Token,
    ) -> Result<B::Data> {
        block.schematic(io, cell)
    }
}

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

/// The type of a PDK's layer set.
pub type PdkLayers<PDK> = <PDK as Pdk>::Layers;

/// The type of a PDK's corners.
pub type PdkCorner<PDK> = <PDK as Pdk>::Corner;
