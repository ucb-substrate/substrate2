//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
mod data;
pub mod layers;

use std::any::Any;
use std::sync::Arc;

use rust_decimal::Decimal;
use substrate::schematic::SchematicData;
use type_dispatch::impl_dispatch;

use crate::block::{self, Block, PdkPrimitive, ScirBlock};
use crate::error::{Error, Result};
use crate::io::{LayoutType, SchematicType};
use crate::layout::{CellBuilder as LayoutCellBuilder, ExportsLayoutData, Layout};
use crate::schematic::schema::Schema;
use crate::schematic::{CellBuilder, ScirCellInner};
use crate::sealed;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// An internal representation of PDK primitives.
    type Primitive: Clone;
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

/// A block that exports data from its schematic.
///
/// All blocks that have a schematic implementation must export data.
pub trait ExportsPdkSchematicData<PDK: Pdk, K = <Self as Block>::Kind>: Block {
    /// Extra schematic data to be stored with the block's generated cell.
    ///
    /// When the block is instantiated, all contained data will be nested
    /// within that instance.
    type Data<S>: SchematicData
    where
        PDK: ToSchema<S>;
}

pub trait PdkSchematic<PDK: Pdk, K = <Self as Block>::Kind>: ExportsPdkSchematicData<PDK> {
    /// Generates the block's schematic.
    fn schematic<S: Schema>(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data<S>>
    where
        PDK: ToSchema<S>;
}

impl<PDK: HasPdkPrimitive<B>, B: Block<Kind = PdkPrimitive>>
    ExportsPdkSchematicData<PDK, PdkPrimitive> for B
{
    type Data<S>
    = ()
    where
    PDK: ToSchema<S>;
}

impl<B: Block<Kind = PdkPrimitive>, PDK: Pdk> PdkSchematic<PDK, PdkPrimitive> for B
where
    PDK: HasPdkPrimitive<B>,
{
    fn schematic<S: Schema>(
        &self,
        _io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data<S>>
    where
        PDK: ToSchema<S>,
    {
        cell.set_primitive(
            PDK::convert_primitive(PDK::primitive(self)).ok_or(Error::UnsupportedPrimitive)?,
        );
        Ok(())
    }
}

#[impl_dispatch({block::Scir; block::InlineScir})]
impl<T, PDK: Pdk, B: Block<Kind = T> + PdkScirSchematic<PDK>> ExportsPdkSchematicData<PDK, T>
    for B
{
    type Data<S>
    = ()
        where
            PDK: ToSchema<S>;
}

#[impl_dispatch({block::Scir; block::InlineScir})]
impl<T, PDK: Pdk, B: Block<Kind = T> + PdkScirSchematic<PDK>> PdkSchematic<PDK, T> for B {
    fn schematic<S: Schema>(
        &self,
        _io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data<S>>
    where
        PDK: ToSchema<S>,
    {
        let (lib, id) = PdkScirSchematic::schematic(self)?;
        let lib = lib
            .convert_primitives(PDK::convert_primitive)
            .ok_or(Error::UnsupportedPrimitive)?;
        cell.set_scir(ScirCellInner {
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
