//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::path::PathBuf;

use arcstr::ArcStr;
use indexmap::IndexMap;
use ngspice::{Ngspice, NgspicePrimitive};
use rust_decimal::Decimal;
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
use scir::schema::{FromSchema, FromSchema};
use scir::Instance;
use substrate::spice::{Primitive, PrimitiveKind, Spice};

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
        Ok(match &primitive.kind {
            PrimitiveKind::RawInstance { cell, .. } => Sky130Primitive::Mos {
                kind: MosKind::try_from_str(cell).ok_or(())?,
                params: MosParams {
                    w: i64::try_from(
                        *primitive
                            .params
                            .get("w")
                            .and_then(|expr| expr.get_numeric_literal())
                            .ok_or(())?,
                    )
                    .map_err(|_| ())?,
                    l: i64::try_from(
                        *primitive
                            .params
                            .get("l")
                            .and_then(|expr| expr.get_numeric_literal())
                            .ok_or(())?,
                    )
                    .map_err(|_| ())?,
                    nf: i64::try_from(
                        primitive
                            .params
                            .get("nf")
                            .and_then(|expr| expr.get_numeric_literal())
                            .copied()
                            .unwrap_or(dec!(1)),
                    )
                    .map_err(|_| ())?,
                },
            },
            _ => todo!(),
        })
    }

    fn recover_instance(
        instance: &mut Instance,
        primitive: &<Spice as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        println!("{}", instance.name());
        match &primitive.kind {
            PrimitiveKind::RawInstance { cell, ports, .. } => {
                if MosKind::try_from_str(cell).is_some() {
                    let connections = instance.connections_mut();
                    for (port, mapped_port) in ports.iter().zip(["D", "G", "S", "B"]) {
                        let concat = connections.remove(port).unwrap();
                        connections.insert(mapped_port.into(), concat);
                    }
                }
            }
            _ => todo!(),
        }
        Ok(())
    }
}

impl FromSchema<Spice> for Sky130Pdk {
    type Error = ();
    fn convert_primitive(
        primitive: <Self as scir::schema::Schema>::Primitive,
    ) -> Result<<Spice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Sky130Primitive::Mos { kind, params } => Primitive {
                kind: PrimitiveKind::RawInstance {
                    cell: kind.open_subckt(),
                    ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                },
                params: HashMap::from_iter([
                    (arcstr::literal!("w"), Decimal::from(params.w).into()),
                    (arcstr::literal!("l"), Decimal::from(params.l).into()),
                    (arcstr::literal!("nf"), Decimal::from(params.nf).into()),
                ]),
            },
            _ => todo!(),
        })
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Self as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Ngspice> for Sky130Pdk {
    type Error = ();
    fn convert_primitive(
        primitive: <Self as scir::schema::Schema>::Primitive,
    ) -> Result<<Ngspice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(NgspicePrimitive::Spice(
            <Self as FromSchema<Spice>>::convert_primitive(primitive)?,
        ))
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Self as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Spectre> for Sky130Pdk {
    type Error = ();
    fn convert_primitive(
        primitive: <Self as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Sky130Primitive::Mos { kind, params } => SpectrePrimitive::RawInstance {
                cell: kind.commercial_subckt(),
                ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                params: HashMap::from_iter([
                    (arcstr::literal!("w"), Decimal::from(params.w).into()),
                    (arcstr::literal!("l"), Decimal::from(params.l).into()),
                    (arcstr::literal!("nf"), Decimal::from(params.nf).into()),
                ]),
            },
            _ => todo!(),
        })
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Self as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
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
