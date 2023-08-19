//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
pub mod data;
pub mod layers;

use std::any::Any;

use arcstr::ArcStr;
use rust_decimal::Decimal;
use substrate::schematic::SchematicData;

use crate::block::{Block, PdkPrimitive};
use crate::error::Result;
use crate::io::{LayoutType, SchematicType};
use crate::layout::{CellBuilder as LayoutCellBuilder, ExportsLayoutData, Layout};
use crate::schematic::{
    CellBuilder as SchematicCellBuilder, CellBuilder, ExportsSchematicData, Schema, Schematic,
};
use crate::sealed;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// An internal representation of PDK primitives.
    type Primitive;
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The type representing a corner in this PDK.
    type Corner: Corner;
    /// The layout database unit for this PDK.
    const LAYOUT_DB_UNITS: Option<Decimal> = None;
}

pub trait ToSchema<S: Schema>: Pdk {
    fn to_schema(primitive: Self::Primitive) -> Option<S::Primitive>;
}
pub trait FromSchema<S: Schema>: Pdk {
    fn from_schema(primitive: S::Primitive) -> Option<Self::Primitive>;
}

pub trait HasPdkPrimitive<B: Block<Kind = PdkPrimitive>>: Pdk {
    fn primitive(block: &B, io: &<<B as Block>::Io as SchematicType>::Bundle) -> Self::Primitive;
}

/// A block that exports data from its schematic.
///
/// All blocks that have a schematic implementation must export data.
pub trait ExportsPdkSchematicData<PDK: Pdk>: Block {
    /// Extra schematic data to be stored with the block's generated cell.
    ///
    /// When the block is instantiated, all contained data will be nested
    /// within that instance.
    type Data<S>: SchematicData
    where
        PDK: ToSchema<S>;
}

pub trait PdkSchematic<PDK: Pdk>: ExportsPdkSchematicData<PDK> {
    /// Generates the block's schematic.
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data<S>>
    where
        PDK: ToSchema<S>;
}

impl<B: Block<Kind = PdkPrimitive> + ExportsPdkSchematicData<PDK>, PDK: Pdk> PdkSchematic<PDK> for B
where
    PDK: HasPdkPrimitive<B>,
{
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data<S>>
    where
        PDK: ToSchema<S>,
    {
        todo!()
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
