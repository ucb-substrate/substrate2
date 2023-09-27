//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
mod data;
pub mod layers;
pub mod primitives;

use std::any::Any;
use std::sync::Arc;

use rust_decimal::Decimal;
use substrate::schematic::{PdkCellBuilder, SchematicData};
use type_dispatch::impl_dispatch;

use crate::block::{self, Block, PdkBlock, PdkPrimitive, PdkScir, ScirBlock};
use crate::error::{Error, Result};
use crate::io::{LayoutType, SchematicType};
use crate::layout::{CellBuilder as LayoutCellBuilder, ExportsLayoutData, Layout};
use crate::schematic::schema::Schema;
use crate::schematic::{Cell, CellBuilder, ExportsNestedData, RawCell, ScirCellInner};
use crate::sealed;
use crate::sealed::Token;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// The schema for storing PDK schematics.
    type Schema: Schema;
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The type representing a corner in this PDK.
    type Corner: Corner;
    /// The layout database unit for this PDK.
    const LAYOUT_DB_UNITS: Option<Decimal> = None;
}

pub trait PdkPrimitiveSchematic<PDK: Pdk>: Block<Kind = PdkPrimitive> {
    fn primitive(block: &Self) -> <PDK::Schema as Schema>::Primitive;
}

pub trait PdkScirSchematic<PDK: Pdk>: Block<Kind = PdkScir> {
    fn schematic(&self) -> Result<(scir::Library<PDK::Schema>, scir::CellId)>;
}

pub trait PdkCellSchematic<PDK: Pdk, K = <Self as Block>::Kind>:
    ExportsNestedData + PdkBlock
{
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> Result<Self::NestedData>;
}

pub trait PdkSchematic<PDK: Pdk, K = <Self as Block>::Kind>: ExportsNestedData + PdkBlock {
    /// Generates the block's schematic.
    #[doc(hidden)]
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        cell: PdkCellBuilder<PDK>,
        _: sealed::Token,
    ) -> Result<(RawCell<PDK::Schema>, Cell<Self>)>;
}

impl<B: Block<Kind = PdkPrimitive>> ExportsNestedData<PdkPrimitive> for B {
    type NestedData = ();
}

impl<PDK: Pdk, B: Block<Kind = PdkPrimitive> + PdkPrimitiveSchematic<PDK>>
    PdkSchematic<PDK, PdkPrimitive> for B
{
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        mut cell: PdkCellBuilder<PDK>,
        _: Token,
    ) -> Result<(RawCell<PDK::Schema>, Cell<Self>)> {
        cell.0
            .set_primitive(PdkPrimitiveSchematic::primitive(block.as_ref()));
        let id = cell.0.metadata.id;
        Ok((cell.0.finish(), Cell::new(id, io, block, Arc::new(()))))
    }
}

impl<B: Block<Kind = block::PdkScir>> ExportsNestedData<block::PdkScir> for B {
    type NestedData = ();
}

impl<PDK: Pdk, B: Block<Kind = block::PdkScir> + PdkScirSchematic<PDK>>
    PdkSchematic<PDK, block::PdkScir> for B
{
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        mut cell: PdkCellBuilder<PDK>,
        _: Token,
    ) -> Result<(RawCell<PDK::Schema>, Cell<Self>)> {
        let (lib, id) = PdkScirSchematic::schematic(block.as_ref())?;
        cell.0.set_scir(ScirCellInner { lib, cell: id });
        let id = cell.0.metadata.id;
        Ok((cell.0.finish(), Cell::new(id, io, block, Arc::new(()))))
    }
}

impl<PDK: Pdk, B: Block<Kind = block::PdkCell> + PdkCellSchematic<PDK>>
    PdkSchematic<PDK, block::PdkCell> for B
{
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        mut cell: PdkCellBuilder<PDK>,
        _: Token,
    ) -> Result<(RawCell<PDK::Schema>, Cell<Self>)> {
        let data = PdkCellSchematic::schematic(block.as_ref(), io.as_ref(), &mut cell);
        data.map(|data| {
            let id = cell.0.metadata.id;
            (cell.0.finish(), Cell::new(id, io, block, Arc::new(data)))
        })
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
