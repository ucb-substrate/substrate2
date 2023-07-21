//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
pub mod data;
pub mod layers;

use std::any::Any;

use arcstr::ArcStr;
use rust_decimal::Decimal;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The type representing a corner in this PDK.
    type Corner: Corner;
    /// The layout database unit for this PDK.
    const LAYOUT_DB_UNITS: Option<Decimal> = None;

    /// The names of all schematic primitives in the PDK.
    ///
    /// This should include the names of transistors,
    /// resistors, capacitors, inductors, etc.
    ///
    /// The default implementation returns an empty list.
    fn schematic_primitives(&self) -> Vec<ArcStr> {
        Vec::new()
    }
}

/// The type of a PDK's layer set.
pub type PdkLayers<PDK> = <PDK as Pdk>::Layers;

/// The type of a PDK's corners.
pub type PdkCorner<PDK> = <PDK as Pdk>::Corner;
