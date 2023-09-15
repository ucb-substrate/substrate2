//! Traits and definitions associated with schemas, or data formats
//! used for storing SCIR libraries.

use crate::Instance;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

/// A data format for storing SCIR libraries.
pub trait Schema {
    /// A primitive used for storing arbitrary data that is opaque to SCIR.
    type Primitive: Primitive;
}

/// A primitive of a SCIR schema.
pub trait Primitive {}

impl<T> Primitive for T {}

/// A schema that can be converted to another schema.
pub trait ToSchema<S: Schema>: Schema {
    /// The conversion error type.
    type Error;

    /// Converts a primitive of the original schema to a primitive of the other.
    fn convert_primitive(
        primitive: <Self as Schema>::Primitive,
    ) -> Result<<S as Schema>::Primitive, Self::Error>;

    /// Converts an instance from the original schema to a new instance
    /// based on its associated primitive.
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Self as Schema>::Primitive,
    ) -> Result<(), Self::Error>;
}

/// A schema that can be recovered from another schema.
pub trait FromSchema<S: Schema>: Schema {
    /// The recovery error type.
    type Error;

    /// Recovers a primitive of the original schema from a primitive of the
    /// other.
    fn recover_primitive(
        primitive: <S as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error>;

    /// Recovers an instance based on an instance from the other schema and
    /// its associated primitive.
    fn recover_instance(
        instance: &mut Instance,
        primitive: &<S as Schema>::Primitive,
    ) -> Result<(), Self::Error>;
}

impl<S1: Schema, S2: FromSchema<S1>> ToSchema<S2> for S1 {
    type Error = <S2 as FromSchema<S1>>::Error;

    fn convert_primitive(
        primitive: <Self as Schema>::Primitive,
    ) -> Result<<S2 as Schema>::Primitive, Self::Error> {
        <S2 as FromSchema<S1>>::recover_primitive(primitive)
    }

    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Self as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        <S2 as FromSchema<S1>>::recover_instance(instance, primitive)
    }
}

/// A schema with primitives stored as serialized bytes.
///
/// Useful for stripping types or converting between schemas for libraries
/// without associated primitive.
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

impl<S: Schema> FromSchema<NoSchema> for S {
    type Error = NoSchemaError;

    fn recover_primitive(
        _primitive: <NoSchema as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Err(NoSchemaError)
    }

    fn recover_instance(
        _instance: &mut Instance,
        _primitive: &<NoSchema as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Err(NoSchemaError)
    }
}

/// A schema with arbitrary string primitives.
pub struct StringSchema;

impl Schema for StringSchema {
    type Primitive = ArcStr;
}
