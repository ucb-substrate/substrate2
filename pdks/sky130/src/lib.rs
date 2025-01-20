//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;

use arcstr::ArcStr;
use derive_builder::Builder;
use layers::Sky130Layer;
use ngspice::Ngspice;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use spectre::Spectre;
use unicase::UniCase;

use crate::mos::{MosKind, MosParams};
use scir::schema::{FromSchema, Schema};
use scir::{Instance, ParamValue};
use spice::Spice;
use substrate::context::Installation;

pub mod corner;
pub mod layers;
pub mod layout;
pub mod mos;
pub mod stdcells;
#[cfg(test)]
mod tests;

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

/// An error converting to/from the [`Sky130`] schema.
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

impl scir::schema::Schema for Sky130 {
    type Primitive = Primitive;
}

impl FromSchema<Spice> for Sky130 {
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

/// A schema for the open PDK.
#[derive(Debug, Clone)]
pub struct Sky130OpenSchema;

impl scir::schema::Schema for Sky130OpenSchema {
    type Primitive = Primitive;
}

impl FromSchema<Sky130> for Sky130OpenSchema {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <Sky130 as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130OpenSchema> for Sky130 {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <Sky130 as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130OpenSchema> for Spice {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130 as scir::schema::Schema>::Primitive,
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
        _primitive: &<Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130OpenSchema> for Ngspice {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<<Ngspice as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(ngspice::Primitive::Spice(<Spice as FromSchema<
            Sky130OpenSchema,
        >>::convert_primitive(
            primitive
        )?))
    }
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        <Spice as FromSchema<Sky130OpenSchema>>::convert_instance(instance, primitive)
    }
}

impl FromSchema<Sky130OpenSchema> for Spectre {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Primitive::RawInstance {
                cell,
                ports,
                params,
            } => spectre::Primitive::RawInstance {
                cell,
                ports,
                params: params.into_iter().collect(),
            },
            Primitive::Mos { kind, params } => spectre::Primitive::RawInstance {
                cell: kind.open_subckt(),
                ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                params: vec![
                    (arcstr::literal!("w"), Decimal::new(params.w, 3).into()),
                    (arcstr::literal!("l"), Decimal::new(params.l, 3).into()),
                    (arcstr::literal!("nf"), Decimal::from(params.nf).into()),
                ],
            },
        })
    }
    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A schema for the SRC NDA PDK.
#[derive(Debug, Clone)]
pub struct Sky130SrcNdaSchema;

impl scir::schema::Schema for Sky130SrcNdaSchema {
    type Primitive = Primitive;
}

impl FromSchema<Sky130> for Sky130SrcNdaSchema {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <Sky130 as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130SrcNdaSchema> for Sky130 {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <Sky130 as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130SrcNdaSchema> for Spice {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130 as scir::schema::Schema>::Primitive,
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
                model: kind.src_nda_subckt(),
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
        _primitive: &<Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130SrcNdaSchema> for Spectre {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Primitive::RawInstance {
                cell,
                ports,
                params,
            } => spectre::Primitive::RawInstance {
                cell,
                ports,
                params: params.into_iter().collect(),
            },
            Primitive::Mos { kind, params } => spectre::Primitive::RawInstance {
                cell: kind.src_nda_subckt(),
                ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                params: vec![
                    (arcstr::literal!("w"), Decimal::new(params.w, 3).into()),
                    (arcstr::literal!("l"), Decimal::new(params.l, 3).into()),
                    (arcstr::literal!("nf"), Decimal::from(params.nf).into()),
                ],
            },
        })
    }
    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A schema for the CDS PDK.
#[derive(Debug, Clone)]
pub struct Sky130CdsSchema;

impl scir::schema::Schema for Sky130CdsSchema {
    type Primitive = Primitive;
}

impl FromSchema<Sky130> for Sky130CdsSchema {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <Sky130 as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130CdsSchema> for Sky130 {
    type Error = Infallible;

    fn convert_primitive(
        primitive: <Sky130 as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        Ok(primitive)
    }

    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130CdsSchema> for Spice {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130 as scir::schema::Schema>::Primitive,
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
                model: kind.cds_subckt(),
                params: HashMap::from_iter([
                    (
                        UniCase::new(arcstr::literal!("w")),
                        Decimal::new(params.w, 9).into(),
                    ),
                    (
                        UniCase::new(arcstr::literal!("l")),
                        Decimal::new(params.l, 9).into(),
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
        _primitive: &<Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl FromSchema<Sky130CdsSchema> for Spectre {
    type Error = Infallible;
    fn convert_primitive(
        primitive: <Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<<Spectre as scir::schema::Schema>::Primitive, Self::Error> {
        Ok(match primitive {
            Primitive::RawInstance {
                cell,
                ports,
                params,
            } => spectre::Primitive::RawInstance {
                cell,
                ports,
                params: params.into_iter().collect(),
            },
            Primitive::Mos { kind, params } => spectre::Primitive::RawInstance {
                cell: kind.cds_subckt(),
                ports: vec!["D".into(), "G".into(), "S".into(), "B".into()],
                params: vec![
                    (arcstr::literal!("w"), Decimal::new(params.w, 9).into()),
                    (arcstr::literal!("l"), Decimal::new(params.l, 9).into()),
                    (arcstr::literal!("nf"), Decimal::from(params.nf).into()),
                ],
            },
        })
    }
    fn convert_instance(
        _instance: &mut Instance,
        _primitive: &<Sky130 as scir::schema::Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// The algorithm for selecting which Spectre models to include.
#[derive(Debug, Default, Clone, Copy)]
pub enum SpectreModelSelect {
    /// Includes all of the available model files.
    #[default]
    All,
    /// Takes model files from the SRC NDA PDK, throwing an error if they are not present.
    SrcNda,
    /// Takes model files from the CDS PDK, throwing an error if they are not present.
    Cds,
    /// Takes model files from the open PDK, throwing an error if they are not present.
    Open,
}

/// The Sky 130 PDK.
#[derive(Debug, Clone, Builder)]
pub struct Sky130 {
    /// The open PDK root directory.
    #[builder(setter(into, strip_option), default)]
    open_root_dir: Option<PathBuf>,
    /// The SRC NDA PDK root directory.
    #[builder(setter(into, strip_option), default)]
    src_nda_root_dir: Option<PathBuf>,
    /// The CDS PDK root directory.
    #[builder(setter(into, strip_option), default)]
    cds_root_dir: Option<PathBuf>,
    /// The Spectre model selection algorithm.
    #[builder(default)]
    spectre_model_select: SpectreModelSelect,
}

impl Sky130 {
    /// Returns a new [`Sky130Builder`].
    #[inline]
    pub fn builder() -> Sky130Builder {
        Sky130Builder::default()
    }

    /// Creates an instantiation of the open PDK.
    #[inline]
    pub fn open(root_dir: impl Into<PathBuf>) -> Self {
        Sky130::builder()
            .open_root_dir(root_dir)
            .spectre_model_select(SpectreModelSelect::Open)
            .build()
            .unwrap()
    }

    /// Creates an instantiation of the SRC NDA PDK.
    #[inline]
    pub fn src_nda(
        open_root_dir: impl Into<PathBuf>,
        src_nda_root_dir: impl Into<PathBuf>,
    ) -> Self {
        Sky130::builder()
            .open_root_dir(open_root_dir)
            .src_nda_root_dir(src_nda_root_dir)
            .spectre_model_select(SpectreModelSelect::SrcNda)
            .build()
            .unwrap()
    }

    /// Creates an instantiation of the SRC NDA PDK without open source PDK support (no standard
    /// cells).
    #[inline]
    pub fn src_nda_only(src_nda_root_dir: impl Into<PathBuf>) -> Self {
        Sky130::builder()
            .src_nda_root_dir(src_nda_root_dir)
            .spectre_model_select(SpectreModelSelect::SrcNda)
            .build()
            .unwrap()
    }

    /// Creates an instantiation of the CDS PDK.
    #[inline]
    pub fn cds(open_root_dir: impl Into<PathBuf>, cds_root_dir: impl Into<PathBuf>) -> Self {
        Sky130::builder()
            .open_root_dir(open_root_dir)
            .cds_root_dir(cds_root_dir)
            .spectre_model_select(SpectreModelSelect::Cds)
            .build()
            .unwrap()
    }

    /// Creates an instantiation of the CDS PDK without open source PDK support.
    #[inline]
    pub fn cds_only(cds_root_dir: impl Into<PathBuf>) -> Self {
        Sky130::builder()
            .cds_root_dir(cds_root_dir)
            .spectre_model_select(SpectreModelSelect::Cds)
            .build()
            .unwrap()
    }
}

impl Installation for Sky130 {}

impl substrate::layout::schema::Schema for Sky130 {
    type Layer = Sky130Layer;
}
