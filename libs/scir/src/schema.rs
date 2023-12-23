//! Traits and definitions associated with schemas, or data formats
//! used for storing SCIR libraries.

use crate::Instance;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

/// A data format for storing SCIR libraries.
// TODO: Add method of validating primitive instances.
pub trait Schema {
    /// A primitive used for storing arbitrary data that is opaque to SCIR.
    type Primitive: Primitive + Sized;
}

/// A primitive of a SCIR schema.
pub trait Primitive {}

impl<T> Primitive for T {}

/// A schema that can be converted from another schema.
pub trait FromSchema<S: Schema + ?Sized>: Schema {
    /// The conversion error type.
    type Error;

    /// Converts a primitive of the other schema to a primitive of this schema.
    fn convert_primitive(
        primitive: <S as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error>;

    /// Converts an instance from the other schema to a new instance
    /// based on its associated primitive.
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<S as Schema>::Primitive,
    ) -> Result<(), Self::Error>;
}

impl<S: Schema + ?Sized> FromSchema<S> for S {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <S as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<S as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A schema with no primitives.
pub struct NoSchema;

/// An error converting to/from [`NoSchema`].
#[derive(
    Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Serialize, Deserialize, thiserror::Error,
)]
#[error("attempted to convert a library containing primitives to/from `NoSchema`")]
pub struct NoSchemaError;

/// The primitive type of the [`NoSchema`] schema.
///
/// Cannot be instantiated as [`NoSchema`] cannot have
/// primitives.
#[derive(Clone, Serialize, Deserialize)]
pub struct NoPrimitive(());

impl Schema for NoSchema {
    type Primitive = NoPrimitive;
}

/// A schema with arbitrary string primitives.
pub struct StringSchema;

impl Schema for StringSchema {
    type Primitive = ArcStr;
}

impl FromSchema<NoSchema> for StringSchema {
    type Error = NoSchemaError;

    fn convert_primitive(
        _primitive: <NoSchema as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Err(NoSchemaError)
    }
    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<NoSchema as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Err(NoSchemaError)
    }
}
