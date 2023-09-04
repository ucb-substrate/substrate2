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
use substrate::pdk::{FromSchema, Pdk, ToSchema};
use substrate::schematic::schema::{Schema, Spice};
use substrate::scir::Expr;
use substrate::spice;

use crate::layers::Sky130Layers;
use crate::mos::{MosKind, MosParams};
use corner::*;
use substrate::spice::Primitive;

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
    type Primitive = Sky130Primitive;
    type Layers = Sky130Layers;
    type Corner = Sky130Corner;
    const LAYOUT_DB_UNITS: Option<rust_decimal::Decimal> = Some(dec!(1e-9));
}

impl ToSchema<Spice> for Sky130Pdk {
    fn convert_primitive(
        primitive: <Self as Pdk>::Primitive,
    ) -> Option<<Spice as Schema>::Primitive> {
        Some(match primitive {
            Sky130Primitive::Mos { kind, params } => Primitive::Mos {
                name: kind.open_subckt(),
                params: IndexMap::from_iter([
                    (arcstr::literal!("w"), Expr::NumericLiteral(params.w.into())),
                    (arcstr::literal!("l"), Expr::NumericLiteral(params.l.into())),
                    (
                        arcstr::literal!("nf"),
                        Expr::NumericLiteral(params.nf.into()),
                    ),
                ]),
            },
            Sky130Primitive::RawInstance {
                cell,
                ports,
                params,
            } => Primitive::RawInstance {
                cell,
                ports,
                params,
            },
        })
    }
}

impl ToSchema<Ngspice> for Sky130Pdk {
    fn convert_primitive(
        primitive: <Self as Pdk>::Primitive,
    ) -> Option<<Ngspice as Schema>::Primitive> {
        Some(NgspicePrimitive::Spice(
            <Sky130Pdk as ToSchema<Spice>>::convert_primitive(primitive)?,
        ))
    }
}

impl ToSchema<Spectre> for Sky130Pdk {
    fn convert_primitive(
        primitive: <Self as Pdk>::Primitive,
    ) -> Option<<Spectre as Schema>::Primitive> {
        Some(match primitive {
            Sky130Primitive::Mos { kind, params } => SpectrePrimitive::RawInstance {
                cell: kind.commercial_subckt(),
                ports: vec![
                    arcstr::literal!("d"),
                    arcstr::literal!("g"),
                    arcstr::literal!("s"),
                    arcstr::literal!("b"),
                ],
                params: IndexMap::from_iter([
                    (arcstr::literal!("w"), Expr::NumericLiteral(params.w.into())),
                    (arcstr::literal!("l"), Expr::NumericLiteral(params.l.into())),
                    (
                        arcstr::literal!("nf"),
                        Expr::NumericLiteral(params.nf.into()),
                    ),
                ]),
            },
            Sky130Primitive::RawInstance {
                cell,
                ports,
                params,
            } => SpectrePrimitive::RawInstance {
                cell,
                ports,
                params,
            },
        })
    }
}

impl FromSchema<Spice> for Sky130Pdk {
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
