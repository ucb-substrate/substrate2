//! Traits and utilities for defining process design kits (PDKs).

pub mod corner;
mod data;
pub mod layers;

use std::any::Any;

use rust_decimal::Decimal;

use crate::block::Block;
use crate::context::Installation;
use crate::error::Result;
use crate::io::LayoutType;
use crate::layout::{CellBuilder as LayoutCellBuilder, ExportsLayoutData, Layout};
use crate::sealed;

use self::corner::*;
use self::layers::Layers;

/// A process development kit.
///
/// The PDK's [`Installation::post_install`] hook should install
/// the PDK's layer set using [`ContextBuilder::install_pdk_layers`].
pub trait Pdk: Installation + Send + Sync + Any {
    /// A set of layers used by the PDK.
    type Layers: Layers;
    /// The layout database unit for this PDK.
    const LAYOUT_DB_UNITS: Option<Decimal> = None;
}

/// The type of a PDK's layer set.
pub type PdkLayers<PDK> = <PDK as Pdk>::Layers;
