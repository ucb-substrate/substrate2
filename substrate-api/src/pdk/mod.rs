//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
pub mod data;
pub mod layers;

use std::any::Any;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The type representing a corner in this PDK.
    type Corner: Corner;

    /// Gets a corner by name.
    ///
    /// The names accepted by this function must match
    /// the names provided by [`Corner::name`].
    fn corner(&self, name: &str) -> Option<Self::Corner>;
}

/// The type of a PDK's layer set.
pub type PdkLayers<PDK> = <PDK as Pdk>::Layers;

/// The type of a PDK's corners.
pub type PdkCorner<PDK> = <PDK as Pdk>::Corner;
