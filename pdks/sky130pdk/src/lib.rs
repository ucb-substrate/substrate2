//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::path::PathBuf;

use arcstr::ArcStr;
use ngspice::{Ngspice, NgspicePrimitive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use spectre::{Spectre, SpectrePrimitive};
use substrate::pdk::corner::SupportsSimulator;
use substrate::pdk::Pdk;

use crate::layers::Sky130Layers;
use crate::mos::{MosKind, MosParams};
use corner::*;
use scir::schema::FromSchema;
use scir::{Instance, ParamValue};
use substrate::spice::{Primitive, PrimitiveKind, Spice};

pub mod corner;
pub mod layers;
pub mod mos;

/// A primitive of the Sky 130 PDK.
#[derive(Debug, Clone)]
pub enum Sky130Primitive {
    /// A raw instance with associated cell `cell`.
    RawInstance {
        /// The associated cell.
        cell: ArcStr,
        /// The ordered ports of the instance.
        ports: Vec<ArcStr>,
        /// The parameters of the instance.
        params: HashMap<ArcStr, ParamValue>,
    },
    /// A Sky 130 MOSFET with ports "D", "G", "S", and "B".
    Mos {
        /// The MOSFET kind.
        kind: MosKind,
        /// The MOSFET parameters.
        params: MosParams,
    },
}

/// An error converting to/from the [`Sky130Pdk`] schema.
#[derive(Debug, Clone, Copy)]
pub enum Sky130ConvError {
    /// A primitive that is not supported by the target schema was encountered.
    UnsupportedPrimitive,
    /// A primitive is missing a required parameter.
    MissingParameter,
    /// A primitive has an extra parameter.
    ExtraParameter,
    /// A primitive has an invalid value for a certain parameter.
    InvalidParameter,
}

impl scir::schema::Schema for Sky130Pdk {
    type Primitive = Sky130Primitive;
}

impl FromSchema<Spice> for Sky130Pdk {
    type Error = Sky130ConvError;

    fn convert_primitive(
        primitive: <Spice as scir::schema::Schema>::Primitive,
    ) -> Result<<Self as scir::schema::Schema>::Primitive, Self::Error> {
        match &primitive.kind {
            PrimitiveKind::RawInstance { cell, ports } => {
                Ok(if let Some(kind) = MosKind::try_from_str(cell) {
                    Sky130Primitive::Mos {
                        kind,
                        params: MosParams {
                            w: i64::try_from(
                                *primitive
                                    .params
                                    .get("w")
                                    .and_then(|expr| expr.get_numeric())
                                    .ok_or(Sky130ConvError::MissingParameter)?,
                            )
                            .map_err(|_| Sky130ConvError::InvalidParameter)?,
                            l: i64::try_from(
                                *primitive
                                    .params
                                    .get("l")
                                    .and_then(|expr| expr.get_numeric())
                                    .ok_or(Sky130ConvError::MissingParameter)?,
                            )
                            .map_err(|_| Sky130ConvError::InvalidParameter)?,
                            nf: i64::try_from(
                                primitive
                                    .params
                                    .get("nf")
                                    .and_then(|expr| expr.get_numeric())
                                    .copied()
                                    .unwrap_or(dec!(1)),
                            )
                            .map_err(|_| Sky130ConvError::InvalidParameter)?,
                        },
                    }
                } else {
                    Sky130Primitive::RawInstance {
                        cell: cell.clone(),
                        ports: ports.clone(),
                        params: primitive.params.clone(),
                    }
                })
            }
            _ => Err(Sky130ConvError::UnsupportedPrimitive),
        }
    }

    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Spice as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
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
            _ => return Err(Sky130ConvError::UnsupportedPrimitive),
        }
        Ok(())
    }
}

impl FromSchema<Sky130Pdk> for Spice {
    type Error = ();
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Spice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Sky130Primitive::RawInstance {
                cell,
                ports,
                params,
            } => Primitive {
                kind: PrimitiveKind::RawInstance { cell, ports },
                params,
            },
            Sky130Primitive::Mos { kind, params } => Primitive {
                kind: PrimitiveKind::RawInstance {
                    cell: kind.open_subckt(),
                    ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                },
                params: HashMap::from_iter([
                    (arcstr::literal!("w"), Decimal::new(params.w, 3).into()),
                    (arcstr::literal!("l"), Decimal::new(params.l, 3).into()),
                    (arcstr::literal!("nf"), Decimal::from(params.nf).into()),
                ]),
            },
        })
    }
    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130Pdk> for Ngspice {
    type Error = ();
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Ngspice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(NgspicePrimitive::Spice(
            <Spice as FromSchema<Sky130Pdk>>::convert_primitive(primitive)?,
        ))
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        <Spice as FromSchema<Sky130Pdk>>::convert_instance(instance, primitive)
    }
}

impl FromSchema<Sky130Pdk> for Spectre {
    type Error = ();
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Sky130Primitive::RawInstance {
                cell,
                ports,
                params,
            } => SpectrePrimitive::RawInstance {
                cell,
                ports,
                params,
            },
            Sky130Primitive::Mos { kind, params } => SpectrePrimitive::RawInstance {
                cell: kind.commercial_subckt(),
                ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                params: HashMap::from_iter([
                    (arcstr::literal!("w"), Decimal::new(params.w, 3).into()),
                    (arcstr::literal!("l"), Decimal::new(params.l, 3).into()),
                    (arcstr::literal!("nf"), Decimal::from(params.nf).into()),
                ]),
            },
        })
    }
    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130Pdk as scir::schema::Schema>::Primitive,
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
    /// Creates an instantiation of the open PDK.
    #[inline]
    pub fn open(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            open_root_dir: Some(root_dir.into()),
            commercial_root_dir: None,
        }
    }

    /// Creates an instantiation of the commercial PDK.
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
    const LAYOUT_DB_UNITS: Option<Decimal> = Some(dec!(1e-9));
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
