//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use std::path::PathBuf;

use arcstr::ArcStr;
use indexmap::IndexMap;
use ngspice::{Ngspice, NgspicePrimitive};
use rust_decimal_macros::dec;
use spectre::{Spectre, SpectrePrimitive};
use substrate::pdk::corner::SupportsSimulator;
use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::scir::Expr;
use substrate::spice;

use crate::layers::Sky130Layers;
use crate::mos::{MosKind, MosParams};
use corner::*;
use scir::schema::{FromSchema, ToSchema};
use scir::Instance;
use substrate::spice::{Primitive, Spice};

pub mod corner;
pub mod layers;
pub mod mos;

#[derive(Debug, Clone)]
pub enum Sky130Primitive {
    RawInstance {
        cell: ArcStr,
        ports: Vec<ArcStr>,
        params: IndexMap<ArcStr, Expr>,
    },
    Mos {
        kind: MosKind,
        params: MosParams,
    },
}

impl scir::schema::Schema for Sky130Pdk {
    type Primitive = Sky130Primitive;
}

impl FromSchema<Spice> for Sky130Pdk {
    type Error = ();

    fn recover_primitive(
        primitive: <Spice as scir::schema::Schema>::Primitive,
    ) -> Result<<Self as scir::schema::Schema>::Primitive, Self::Error> {
        todo!()
    }

    fn recover_instance(
        instance: &mut Instance,
        primitive: &<Spice as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

impl ToSchema<Ngspice> for Sky130Pdk {
    type Error = ();
    fn convert_primitive(
        primitive: <Self as scir::schema::Schema>::Primitive,
    ) -> Result<<Ngspice as scir::schema::Schema>::Primitive, Self::Error> {
        todo!()
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Self as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

impl ToSchema<Spectre> for Sky130Pdk {
    type Error = ();
    fn convert_primitive(
        primitive: <Self as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        todo!()
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Self as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

/// The Sky 130 PDK.
#[derive(Debug, Clone)]
pub struct Sky130Pdk {
    open_root_dir: Option<PathBuf>,
    commercial_root_dir: Option<PathBuf>,
}

impl Sky130Pdk {
    #[inline]
    pub fn open(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            open_root_dir: Some(root_dir.into()),
            commercial_root_dir: None,
        }
    }
    #[inline]
    pub fn commercial(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            open_root_dir: None,
            commercial_root_dir: Some(root_dir.into()),
        }
    }
    /// Creates an instance of the PDK with the given root directories.
    #[inline]
    pub fn new(open_root_dir: impl Into<PathBuf>, commercial_root_dir: impl Into<PathBuf>) -> Self {
        Self {
            open_root_dir: Some(open_root_dir.into()),
            commercial_root_dir: Some(commercial_root_dir.into()),
        }
    }
}

impl Pdk for Sky130Pdk {
    type Layers = Sky130Layers;
    type Corner = Sky130Corner;
    const LAYOUT_DB_UNITS: Option<rust_decimal::Decimal> = Some(dec!(1e-9));
}

impl SupportsSimulator<Spectre> for Sky130Pdk {
    fn install_corner(
        &self,
        corner: &<Self as substrate::pdk::Pdk>::Corner,
        opts: &mut <Spectre as substrate::simulation::Simulator>::Options,
    ) {
        opts.include(self.commercial_root_dir.as_ref().unwrap().join(format!(
            "MODELS/SPECTRE/s8phirs_10r/Models/{}.cor",
            corner.name()
        )));
    }
}

impl SupportsSimulator<Ngspice> for Sky130Pdk {
    fn install_corner(
        &self,
        corner: &<Self as substrate::pdk::Pdk>::Corner,
        opts: &mut <Ngspice as substrate::simulation::Simulator>::Options,
    ) {
        opts.include_section(
            self.open_root_dir
                .as_ref()
                .unwrap()
                .join("libraries/sky130_fd_pr/latest/models/sky130.lib.spice"),
            corner.name(),
        );
    }
}
