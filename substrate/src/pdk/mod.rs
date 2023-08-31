//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
mod data;
pub mod layers;
pub mod primitives;

use std::any::Any;
use std::sync::Arc;

use rust_decimal::Decimal;
use substrate::schematic::{PdkCellBuilder, Primitive, SchematicData};
use type_dispatch::impl_dispatch;

use crate::block::{self, Block, PdkBlock, PdkPrimitive, ScirBlock};
use crate::error::{Error, Result};
use crate::io::{LayoutType, SchematicType};
use crate::layout::{CellBuilder as LayoutCellBuilder, ExportsLayoutData, Layout};
use crate::schematic::schema::Schema;
use crate::schematic::{CellBuilder, ExportsNestedNodes, ScirCellInner};
use crate::sealed;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// An internal representation of PDK primitives.
    type Primitive: Primitive;
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The type representing a corner in this PDK.
    type Corner: Corner;
    /// The layout database unit for this PDK.
    const LAYOUT_DB_UNITS: Option<Decimal> = None;
}

/// A PDK whose primitives can be converted to primitives of schema `S`.
pub trait ToSchema<S: Schema>: Pdk {
    /// Converts a PDK primitive to a schema primitive.
    fn convert_primitive(primitive: <Self as Pdk>::Primitive) -> Option<<S as Schema>::Primitive>;
}

/// A PDK whose primitives can be created from primitives of schema `S`.
pub trait FromSchema<S: Schema>: Pdk {
    /// Converts a schema primitive to a PDK primitive.
    fn convert_primitive(primitive: <S as Schema>::Primitive) -> Option<<Self as Pdk>::Primitive>;
}

pub trait HasPdkPrimitive<B: Block<Kind = PdkPrimitive>>: Pdk {
    fn primitive(block: &B) -> Self::Primitive;
}

pub trait PdkScirSchematic<PDK: Pdk>: ScirBlock {
    fn schematic(&self) -> Result<(scir::Library<PDK::Primitive>, scir::CellId)>;
}

pub trait PdkSchematic<PDK: Pdk, K = <Self as Block>::Kind>: ExportsNestedNodes + PdkBlock {
    /// Generates the block's schematic.
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> Result<Self::NestedNodes>;
}

impl<B: Block<Kind = PdkPrimitive>> ExportsNestedNodes<PdkPrimitive> for B {
    type NestedNodes = ();
}

impl<B: Block<Kind = PdkPrimitive>, PDK: Pdk> PdkSchematic<PDK, PdkPrimitive> for B
where
    PDK: HasPdkPrimitive<B>,
{
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> Result<Self::NestedNodes> {
        cell.0.set_primitive(PDK::primitive(self));
        Ok(())
    }
}

impl<B: Block<Kind = block::PdkScir>> ExportsNestedNodes<block::PdkScir> for B {
    type NestedNodes = ();
}

impl<PDK: Pdk, B: Block<Kind = block::PdkScir> + PdkScirSchematic<PDK>>
    PdkSchematic<PDK, block::PdkScir> for B
{
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut PdkCellBuilder<PDK>,
    ) -> Result<Self::NestedNodes> {
        let (lib, id) = PdkScirSchematic::schematic(self)?;
        cell.0.set_scir(ScirCellInner {
            lib: Arc::new(lib),
            cell: id,
        });
        Ok(())
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
