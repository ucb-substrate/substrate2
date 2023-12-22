//! Traits and types for specifying formats for storing Substrate schematics.
use std::any::Any;

/// A Substrate wrapper for [`scir::schema::Schema`].
///
/// This trait should never be directly implemented. Implementing [`scir::schema::Schema`]
/// should suffice provided that the necessary trait bounds are satisfied.
pub trait Schema:
    scir::schema::Schema<Primitive = <Self as Schema>::Primitive> + Send + Sync + Any
{
    /// A primitive used for storing arbitrary data that is opaque to SCIR.
    type Primitive: Primitive;
}
impl<T: scir::schema::Schema<Primitive = impl Primitive> + Send + Sync + Any> Schema for T {
    type Primitive = <T as scir::schema::Schema>::Primitive;
}

/// A Substrate wrapper for [`scir::schema::Primitive`].
///
/// This trait should never be directly implemented. Implementing [`scir::schema::Primitive`]
/// should suffice provided that the necessary trait bounds are satisfied.
pub trait Primitive: Clone + Send + Sync + Any {}

impl<T: Clone + Send + Sync + Any> Primitive for T {}

/// A Substrate wrapper for [`scir::schema::FromSchema`].
///
/// This trait should never be directly implemented. Implementing [`scir::schema::FromSchema`]
/// should suffice provided that the necessary trait bounds are satisfied.
pub trait FromSchema<S: Schema + ?Sized>: Schema + scir::schema::FromSchema<S> {}

impl<S1: Schema + ?Sized, S2: Schema + ?Sized + scir::schema::FromSchema<S1>> FromSchema<S1>
    for S2
{
}
