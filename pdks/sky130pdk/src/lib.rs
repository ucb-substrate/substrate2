//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use std::path::PathBuf;

use crate::layers::Sky130Layers;
use corner::*;
use rust_decimal_macros::dec;
use substrate::pdk::Pdk;

pub mod corner;
pub mod layers;
pub mod mos;

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
    type Layers = Sky130Layers;
    type Corner = Sky130Corner;
    const LAYOUT_DB_UNITS: Option<rust_decimal::Decimal> = Some(dec!(1e-9));
    fn schematic_primitives(&self) -> Vec<arcstr::ArcStr> {
        vec![
            arcstr::literal!("sky130_fd_pr__nfet_01v8"),
            arcstr::literal!("sky130_fd_pr__nfet_01v8_lvt"),
            arcstr::literal!("sky130_fd_pr__nfet_03v3_nvt"),
            arcstr::literal!("sky130_fd_pr__nfet_05v0_nvt"),
            arcstr::literal!("sky130_fd_pr__nfet_20v0"),
            arcstr::literal!("sky130_fd_pr__special_nfet_latch"),
            arcstr::literal!("sky130_fd_pr__special_nfet_pass"),
            arcstr::literal!("sky130_fd_pr__special_pfet_pass"),
            arcstr::literal!("sky130_fd_pr__pfet_01v8"),
            arcstr::literal!("sky130_fd_pr__pfet_01v8_lvt"),
            arcstr::literal!("sky130_fd_pr__pfet_01v8_hvt"),
            arcstr::literal!("sky130_fd_pr__pfet_20v0"),
        ]
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
    type Layers = Sky130Layers;
    type Corner = Sky130Corner;
    const LAYOUT_DB_UNITS: Option<rust_decimal::Decimal> = Some(dec!(1e-9));

    fn schematic_primitives(&self) -> Vec<arcstr::ArcStr> {
        vec![
            arcstr::literal!("nshort"),
            arcstr::literal!("nlowvt"),
            arcstr::literal!("ntvnative"),
            arcstr::literal!("nhvnative"),
            arcstr::literal!("nvhv"),
            arcstr::literal!("npd"),
            arcstr::literal!("npass"),
            arcstr::literal!("ppu"),
            arcstr::literal!("pshort"),
            arcstr::literal!("phighvt"),
            arcstr::literal!("plowvt"),
            arcstr::literal!("pvhv"),
        ]
    }
}
