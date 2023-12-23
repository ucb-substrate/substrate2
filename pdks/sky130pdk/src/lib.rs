//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;

use arcstr::ArcStr;
use ngspice::Ngspice;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use spectre::Spectre;
use substrate::pdk::Pdk;
use unicase::UniCase;

use crate::layers::Sky130Layers;
use crate::mos::{MosKind, MosParams};
use scir::schema::{FromSchema, Schema};
use scir::{Instance, ParamValue};
use spice::Spice;
use substrate::context::{ContextBuilder, Installation};

pub mod atoll;
pub mod corner;
pub mod layers;
pub mod mos;
pub mod stdcells;

/// A primitive of the Sky 130 PDK.
#[derive(Debug, Clone)]
pub enum Primitive {
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
pub enum ConvError {
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
    type Primitive = Primitive;
}

impl FromSchema<Spice> for Sky130Pdk {
    type Error = ConvError;

    fn convert_primitive(
        primitive: <Spice as scir::schema::Schema>::Primitive,
    ) -> Result<<Self as scir::schema::Schema>::Primitive, Self::Error> {
        match &primitive {
            spice::Primitive::RawInstance {
                cell,
                ports,
                params,
            } => Ok(if let Some(kind) = MosKind::try_from_str(cell) {
                Primitive::Mos {
                    kind,
                    params: MosParams {
                        w: i64::try_from(
                            *params
                                .get(&UniCase::new(arcstr::literal!("w")))
                                .and_then(|expr| expr.get_numeric())
                                .ok_or(ConvError::MissingParameter)?
                                * dec!(1000),
                        )
                        .map_err(|_| ConvError::InvalidParameter)?,
                        l: i64::try_from(
                            *params
                                .get(&UniCase::new(arcstr::literal!("l")))
                                .and_then(|expr| expr.get_numeric())
                                .ok_or(ConvError::MissingParameter)?
                                * dec!(1000),
                        )
                        .map_err(|_| ConvError::InvalidParameter)?,
                        nf: i64::try_from(
                            params
                                .get(&UniCase::new(arcstr::literal!("nf")))
                                .and_then(|expr| expr.get_numeric())
                                .copied()
                                .unwrap_or(dec!(1)),
                        )
                        .map_err(|_| ConvError::InvalidParameter)?,
                    },
                }
            } else {
                Primitive::RawInstance {
                    cell: cell.clone(),
                    ports: ports.clone(),
                    params: params
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k.into_inner(), v))
                        .collect(),
                }
            }),
            _ => Err(ConvError::UnsupportedPrimitive),
        }
    }

    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Spice as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        match primitive {
            spice::Primitive::RawInstance { cell, ports, .. } => {
                if MosKind::try_from_str(cell).is_some() {
                    let connections = instance.connections_mut();
                    for (port, mapped_port) in ports.iter().zip(["D", "G", "S", "B"]) {
                        let concat = connections.remove(port).unwrap();
                        connections.insert(mapped_port.into(), concat);
                    }
                }
            }
            _ => return Err(ConvError::UnsupportedPrimitive),
        }
        Ok(())
    }
}

impl FromSchema<Sky130Pdk> for Spice {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Spice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Primitive::RawInstance {
                cell,
                ports,
                params,
            } => spice::Primitive::RawInstance {
                cell,
                ports,
                params: params
                    .into_iter()
                    .map(|(k, v)| (UniCase::new(k), v))
                    .collect(),
            },
            Primitive::Mos { kind, params } => spice::Primitive::RawInstance {
                cell: kind.open_subckt(),
                ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                params: HashMap::from_iter([
                    (
                        UniCase::new(arcstr::literal!("w")),
                        Decimal::new(params.w, 3).into(),
                    ),
                    (
                        UniCase::new(arcstr::literal!("l")),
                        Decimal::new(params.l, 3).into(),
                    ),
                    (
                        UniCase::new(arcstr::literal!("nf")),
                        Decimal::from(params.nf).into(),
                    ),
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
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Ngspice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(ngspice::Primitive::Spice(<Spice as FromSchema<
            Sky130Pdk,
        >>::convert_primitive(
            primitive
        )?))
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        <Spice as FromSchema<Sky130Pdk>>::convert_instance(instance, primitive)
    }
}

impl FromSchema<Sky130Pdk> for Spectre {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Primitive::RawInstance {
                cell,
                ports,
                params,
            } => spectre::Primitive::RawInstance {
                cell,
                ports,
                params,
            },
            Primitive::Mos { kind, params } => spectre::Primitive::RawInstance {
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

impl scir::schema::Schema for Sky130CommercialSchema {
    type Primitive = Primitive;
}

impl FromSchema<Sky130Pdk> for Sky130CommercialSchema {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <Sky130Pdk as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130Pdk as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130CommercialSchema> for Spice {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Spice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Primitive::RawInstance {
                cell,
                ports,
                params,
            } => spice::Primitive::RawInstance {
                cell,
                ports,
                params: params
                    .into_iter()
                    .map(|(k, v)| (UniCase::new(k), v))
                    .collect(),
            },
            Primitive::Mos { kind, params } => spice::Primitive::Mos {
                model: kind.commercial_subckt(),
                params: HashMap::from_iter([
                    (
                        UniCase::new(arcstr::literal!("w")),
                        Decimal::new(params.w, 3).into(),
                    ),
                    (
                        UniCase::new(arcstr::literal!("l")),
                        Decimal::new(params.l, 3).into(),
                    ),
                    (
                        UniCase::new(arcstr::literal!("nf")),
                        Decimal::from(params.nf).into(),
                    ),
                    (UniCase::new(arcstr::literal!("mult")), dec!(1).into()),
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

impl FromSchema<Sky130CommercialSchema> for Spectre {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130Pdk as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Primitive::RawInstance {
                cell,
                ports,
                params,
            } => spectre::Primitive::RawInstance {
                cell,
                ports,
                params,
            },
            Primitive::Mos { kind, params } => spectre::Primitive::RawInstance {
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

/// A schema for the commercial PDK.
#[derive(Debug, Clone)]
pub struct Sky130CommercialSchema;

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

impl Installation for Sky130Pdk {
    fn post_install(&self, ctx: &mut ContextBuilder) {
        let layers = ctx.install_pdk_layers::<Sky130Pdk>();

        ctx.install(layers.atoll_layer_stack());
    }
}

impl Pdk for Sky130Pdk {
    type Layers = Sky130Layers;
    const LAYOUT_DB_UNITS: Decimal = dec!(1e-9);
}
