//! Traits and utilities for defining process design kits (PDKs).

pub mod data;
pub mod layers;

use std::any::Any;

use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// A set of layers used by the PDK.
    type Layers: Layers;
}
