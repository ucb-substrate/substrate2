//! The Sky 130 nm process development kit.
//!
//! Includes both open source and commercial PDK flavors.
#![warn(missing_docs)]

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use arcstr::ArcStr;
use derive_builder::Builder;
use layers::Sky130Layer;
use ngspice::Ngspice;
use res::PrecisionResistor;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use spectre::Spectre;
use thiserror::Error;
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
pub mod res;
pub mod stdcells;
#[cfg(test)]
mod tests;

pub const SKY130_LVS: &str = concat!(env!("SKY130_CDS_PDK_ROOT"), "/Sky130_LVS");
pub const SKY130_LVS_RULES_PATH: &str =
    concat!(env!("SKY130_CDS_PDK_ROOT"), "/Sky130_LVS/sky130.lvs.pvl");
pub const SKY130_TECHNOLOGY_DIR: &str =
    concat!(env!("SKY130_CDS_PDK_ROOT"), "/quantus/extraction/typical");
pub const SKY130_CDS_TT_MODEL_PATH: &str =
    concat!(env!("SKY130_CDS_PDK_ROOT"), "/models/corners/tt.spice");
pub const SKY130_DRC: &str = concat!(env!("SKY130_CDS_PDK_ROOT"), "/Sky130_DRC");
pub const SKY130_DRC_RULES_PATH: &str = concat!(
    env!("SKY130_CDS_PDK_ROOT"),
    "/Sky130_DRC/sky130_rev_0.0_1.0.drc.pvl",
);

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
    PrecisionResistor(PrecisionResistor),
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

fn convert_spice_mos(
    kind: &str,
    params: &HashMap<UniCase<ArcStr>, ParamValue>,
) -> Result<Primitive, ConvError> {
    let schema = MosKind::schema(kind).ok_or(ConvError::UnsupportedPrimitive)?;
    let kind = MosKind::try_from_str(kind).ok_or(ConvError::UnsupportedPrimitive)?;
    let scale = match schema {
        Sky130Schema::Open | Sky130Schema::SrcNda => dec!(1e3),
        Sky130Schema::Cds => dec!(1e9),
    };
    let mult = i64::try_from(
        params
            .get(&UniCase::new(arcstr::literal!("mult")))
            .and_then(|expr| expr.get_numeric())
            .copied()
            .unwrap_or(dec!(1)),
    )
    .map_err(|_| ConvError::InvalidParameter)?;
    let m = i64::try_from(
        params
            .get(&UniCase::new(arcstr::literal!("m")))
            .and_then(|expr| expr.get_numeric())
            .copied()
            .unwrap_or(dec!(1)),
    )
    .map_err(|_| ConvError::InvalidParameter)?;
    Ok(Primitive::Mos {
        kind,
        params: MosParams {
            w: i64::try_from(
                *params
                    .get(&UniCase::new(arcstr::literal!("w")))
                    .and_then(|expr| expr.get_numeric())
                    .ok_or(ConvError::MissingParameter)?
                    * scale,
            )
            .map_err(|_| ConvError::InvalidParameter)?,
            l: i64::try_from(
                *params
                    .get(&UniCase::new(arcstr::literal!("l")))
                    .and_then(|expr| expr.get_numeric())
                    .ok_or(ConvError::MissingParameter)?
                    * scale,
            )
            .map_err(|_| ConvError::InvalidParameter)?,
            nf: i64::try_from(
                params
                    .get(&UniCase::new(arcstr::literal!("nf")))
                    .and_then(|expr| expr.get_numeric())
                    .copied()
                    .unwrap_or(dec!(1)),
            )
            .map_err(|_| ConvError::InvalidParameter)?
                * m
                * mult,
        },
    })
}

impl FromSchema<Spice> for Sky130 {
    type Error = ConvError;

    fn convert_primitive(
        primitive: <Spice as scir::schema::Schema>::Primitive,
    ) -> Result<<Self as scir::schema::Schema>::Primitive, Self::Error> {
        match primitive {
            spice::Primitive::RawInstance {
                cell,
                ports,
                params,
            } => {
                if MosKind::try_from_str(&cell).is_some() {
                    convert_spice_mos(&cell, &params)
                } else {
                    Ok(Primitive::RawInstance {
                        cell,
                        ports,
                        params: params
                            .clone()
                            .into_iter()
                            .map(|(k, v)| (k.into_inner(), v))
                            .collect(),
                    })
                }
            }
            spice::Primitive::Mos { model, params } => convert_spice_mos(&model, &params),
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
            spice::Primitive::Mos { model, .. } => {
                if MosKind::try_from_str(model).is_none() {
                    return Err(ConvError::UnsupportedPrimitive);
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
                        // Calibre decks don't support nf, so assign mult=nf instead.
                        UniCase::new(arcstr::literal!("mult")),
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

/// The available SKY130 schemas.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Sky130Schema {
    /// Open-source sky130 schema.
    #[default]
    Open,
    /// Cadence sky130 schema.
    Cds,
    /// NDA sky130 schema.
    SrcNda,
}

impl Display for Sky130Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Cds => write!(f, "cds"),
            Self::SrcNda => write!(f, "src-nda"),
        }
    }
}

/// Error parsing sky130 schema.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
#[error("error parsing sky130 schema")]
pub struct Sky130SchemaParseErr;

impl FromStr for Sky130Schema {
    type Err = Sky130SchemaParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "cds" => Ok(Self::Cds),
            "src-nda" => Ok(Self::SrcNda),
            _ => Err(Sky130SchemaParseErr),
        }
    }
}
