use std::any::Any;

use crate::block::{Block, SchemaPrimitive};
use crate::error::Result;
use crate::io::SchematicType;
use crate::pdk::Pdk;
use crate::schematic::{CellBuilder, ExportsSchematicData, Schematic};

/// A format for storing Substrate schematics.
///
/// Any tool that uses Substrate schematics (e.g. netlisters, LVS tools,
/// autorouters, etc.) can implement this trait in order to receive
/// schematics in the desired format.
pub trait Schema: Send + Sync + Any {
    type Primitive: Send + Sync + Any;
}

pub trait HasSchemaPrimitive<B: Block<Kind = SchemaPrimitive>>: Schema {
    fn primitive(block: &B) -> Self::Primitive;
}

impl<PDK: Pdk, S: HasSchemaPrimitive<B>, B: Block<Kind = SchemaPrimitive>>
    ExportsSchematicData<PDK, S, SchemaPrimitive> for B
{
    type Data = ();
}
impl<PDK: Pdk, S: HasSchemaPrimitive<B>, B: Block<Kind = SchemaPrimitive>>
    Schematic<PDK, S, SchemaPrimitive> for B
{
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::Data> {
        cell.set_primitive(S::primitive(self));
        Ok(())
    }
}

pub struct Spice;

impl Schema for Spice {
    type Primitive = spice::Primitive;
}
