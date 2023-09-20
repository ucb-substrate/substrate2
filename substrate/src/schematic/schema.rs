//! Traits and types for specifying formats for storing Substrate schematics.
use scir::schema::Schema;
use scir::Expr;
use std::any::Any;
use substrate::schematic::Primitive;

use crate::block::{Block, SchemaPrimitive};
use crate::error::Result;
use crate::io::SchematicType;
use crate::pdk::Pdk;
use crate::schematic::primitives::Resistor;
use crate::schematic::{CellBuilder, ExportsNestedData, Schematic};

/// A schema that has a primitive associated with a certain block.
pub trait HasSchemaPrimitive<B: Block<Kind = SchemaPrimitive>>: Schema {
    /// Returns the schema primitive corresponding to `block`.
    fn primitive(block: &B) -> Self::Primitive;
}

impl<B: Block<Kind = SchemaPrimitive>> ExportsNestedData<SchemaPrimitive> for B {
    type NestedData = ();
}
impl<PDK: Pdk, S: HasSchemaPrimitive<B>, B: Block<Kind = SchemaPrimitive>>
    Schematic<PDK, S, SchemaPrimitive> for B
{
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, S>,
    ) -> Result<Self::NestedData> {
        cell.0.set_primitive(S::primitive(self));
        Ok(())
    }
}
