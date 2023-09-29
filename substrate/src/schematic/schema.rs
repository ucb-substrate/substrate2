//! Traits and types for specifying formats for storing Substrate schematics.
use scir::Expr;
use std::any::Any;
use std::sync::Arc;
use substrate::pdk::SupportsSchema;

use crate::block::{Block, SchemaPrimitive};
use crate::error::Result;
use crate::io::SchematicType;
use crate::pdk::Pdk;
use crate::schematic::primitives::Resistor;
use crate::schematic::{Cell, CellBuilder, ExportsNestedData, RawCell, Schematic};
use crate::sealed::Token;

pub trait Schema:
    scir::schema::Schema<Primitive = <Self as Schema>::Primitive> + Send + Sync + Any
{
    type Primitive: Primitive;
}

impl<T: scir::schema::Schema<Primitive = impl Primitive> + Send + Sync + Any> Schema for T {
    type Primitive = <T as scir::schema::Schema>::Primitive;
}

pub trait Primitive: Clone + Send + Sync + Any {}

impl<T: Clone + Send + Sync + Any> Primitive for T {}

/// A schema that has a primitive associated with a certain block.
pub trait SchemaPrimitiveWrapper<S: Schema>: Block<Kind = SchemaPrimitive> {
    /// Returns the schema primitive corresponding to `block`.
    fn primitive(&self) -> <S as Schema>::Primitive;
}

impl<B: Block<Kind = SchemaPrimitive>> ExportsNestedData<SchemaPrimitive> for B {
    type NestedData = ();
}

impl<
        PDK: SupportsSchema<S>,
        S: Schema,
        B: Block<Kind = SchemaPrimitive> + SchemaPrimitiveWrapper<S>,
    > Schematic<PDK, S, SchemaPrimitive> for B
{
    fn schematic(
        block: Arc<Self>,
        io: Arc<<<Self as Block>::Io as SchematicType>::Bundle>,
        mut cell: CellBuilder<PDK, S>,
        _: Token,
    ) -> Result<(RawCell<PDK, S>, Cell<Self>)> {
        cell.0
            .set_primitive(SchemaPrimitiveWrapper::primitive(block.as_ref()));
        let id = cell.0.metadata.id;
        Ok((cell.0.finish(), Cell::new(id, io, block, Arc::new(()))))
    }
}
