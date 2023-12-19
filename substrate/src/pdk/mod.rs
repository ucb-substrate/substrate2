//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
mod data;
pub mod layers;

use std::any::Any;

use rust_decimal::Decimal;

use crate::context::Installation;

use self::layers::Layers;

/// A process development kit.
///
/// The PDK's [`Installation::post_install`] hook should install
/// the PDK's layer set using
/// [`install_pdk_layers`](crate::context::ContextBuilder::install_pdk_layers).
pub trait Pdk: Installation + Send + Sync + Any {
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The layout database unit for this PDK.
    const LAYOUT_DB_UNITS: Decimal;
}

/// The type of a PDK's layer set.
pub type PdkLayers<PDK> = <PDK as Pdk>::Layers;
