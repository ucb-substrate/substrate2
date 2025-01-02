//! Layout schemas.

use std::{any::Any, fmt::Debug};

/// Defines a layout schema.
pub trait Schema {
    /// The type representing allowable layers in this schema.
    type Layer: Debug + Send + Sync + Any + Clone + Eq;
}
