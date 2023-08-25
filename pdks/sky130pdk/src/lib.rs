//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use arcstr::ArcStr;
use indexmap::IndexMap;
use std::path::PathBuf;

use crate::layers::Sky130Layers;
use crate::mos::MosParams;
use corner::*;
use rust_decimal_macros::dec;
use substrate::pdk::{FromSchema, Pdk};
use substrate::schematic::schema::{Schema, Spice};
use substrate::scir::Expr;
use substrate::spice;

pub mod corner;
pub mod layers;
pub mod mos;

#[derive(Clone)]
pub enum Sky130Primitive {
    RawInstance {
        cell: ArcStr,
        ports: Vec<ArcStr>,
        params: IndexMap<ArcStr, Expr>,
    },
    Mos(MosParams),
}

/// Flavors of the Sky 130 PDK.
pub enum Sky130PdkFlavor {
    /// The open PDK that supports ngspice.
    Open,
    /// The commercial PDK that supports Spectre.
    Commercial,
}

/// A trait defining shared behavior between the open and commercial PDKs.
pub trait Sky130Pdk: Pdk<Layers = Sky130Layers, Corner = Sky130Corner> {
    /// The flavor of PDK.
    const FLAVOR: Sky130PdkFlavor;
}

/// The open Sky 130 PDK.
#[derive(Debug, Clone)]
pub struct Sky130OpenPdk {
    #[allow(dead_code)]
    root_dir: PathBuf,
}

impl Sky130Pdk for Sky130OpenPdk {
    const FLAVOR: Sky130PdkFlavor = Sky130PdkFlavor::Open;
}

impl Sky130OpenPdk {
    /// Creates an instance of the open PDK with the given root directory.
    #[inline]
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }
}

impl Pdk for Sky130OpenPdk {
    type Primitive = Sky130Primitive;
    type Layers = Sky130Layers;
    type Corner = Sky130Corner;
    const LAYOUT_DB_UNITS: Option<rust_decimal::Decimal> = Some(dec!(1e-9));
}

impl FromSchema<Spice> for Sky130OpenPdk {
    fn convert_primitive(
        primitive: <Spice as Schema>::Primitive,
    ) -> Option<<Self as Pdk>::Primitive> {
        Some(match primitive {
            spice::Primitive::RawInstance {
                cell,
                ports,
                params,
            } => Sky130Primitive::RawInstance {
                cell,
                ports,
                params,
            },
            _ => todo!(),
        })
    }
}

/// The commercial Sky 130 PDK.
#[derive(Debug, Clone)]
pub struct Sky130CommercialPdk {
    root_dir: PathBuf,
}

impl Sky130Pdk for Sky130CommercialPdk {
    const FLAVOR: Sky130PdkFlavor = Sky130PdkFlavor::Commercial;
}

impl Sky130CommercialPdk {
    /// Creates an instance of the commercial PDK with the given root directory.
    #[inline]
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }
}

impl Pdk for Sky130CommercialPdk {
    type Primitive = Sky130Primitive;
    type Layers = Sky130Layers;
    type Corner = Sky130Corner;
    const LAYOUT_DB_UNITS: Option<rust_decimal::Decimal> = Some(dec!(1e-9));
}
