//! Spectre-specific blocks for use in testbenches.

use indexmap::IndexMap;
use rust_decimal::Decimal;
use scir::Expr;
use serde::{Deserialize, Serialize};
use substrate::block::{Block, SchemaPrimitive};
use substrate::io::TwoTerminalIo;
use substrate::pdk::Pdk;
use substrate::schematic::schema::HasSchemaPrimitive;
use substrate::schematic::Schematic;

use crate::{Spectre, SpectrePrimitive};

/// Data associated with a pulse [`Vsource`].
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Pulse {
    /// The zero value of the pulse.
    pub val0: Decimal,
    /// The one value of the pulse.
    pub val1: Decimal,
    /// The period of the pulse.
    pub period: Option<Decimal>,
    /// Rise time.
    pub rise: Option<Decimal>,
    /// Fall time.
    pub fall: Option<Decimal>,
    /// The pulse width.
    pub width: Option<Decimal>,
    /// Waveform delay.
    pub delay: Option<Decimal>,
}

/// A voltage source.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Vsource {
    /// A dc voltage source.
    Dc(Decimal),
    /// A pulse voltage source.
    Pulse(Pulse),
}

impl Vsource {
    /// Creates a new DC voltage source.
    pub fn dc(value: Decimal) -> Self {
        Self::Dc(value)
    }

    /// Creates a new pulse voltage source.
    pub fn pulse(value: Pulse) -> Self {
        Self::Pulse(value)
    }
}

impl Block for Vsource {
    type Io = TwoTerminalIo;
    type Kind = SchemaPrimitive;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("vsource")
    }
    fn name(&self) -> arcstr::ArcStr {
        // `vsource` is a reserved Spectre keyword,
        // so we call this block `uservsource`.
        arcstr::format!("uservsource")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchemaPrimitive<Vsource> for Spectre {
    fn primitive(block: &Vsource) -> Self::Primitive {
        use arcstr::literal;
        let mut params = IndexMap::new();
        match block {
            Vsource::Dc(dc) => {
                params.insert(literal!("type"), Expr::StringLiteral(literal!("dc")));
                params.insert(literal!("dc"), Expr::NumericLiteral(*dc));
            }
            Vsource::Pulse(pulse) => {
                params.insert(literal!("type"), Expr::StringLiteral(literal!("pulse")));
                params.insert(literal!("val0"), Expr::NumericLiteral(pulse.val0));
                params.insert(literal!("val1"), Expr::NumericLiteral(pulse.val1));
                if let Some(period) = pulse.period {
                    params.insert(literal!("period"), Expr::NumericLiteral(period));
                }
                if let Some(rise) = pulse.rise {
                    params.insert(literal!("rise"), Expr::NumericLiteral(rise));
                }
                if let Some(fall) = pulse.fall {
                    params.insert(literal!("fall"), Expr::NumericLiteral(fall));
                }
                if let Some(width) = pulse.width {
                    params.insert(literal!("width"), Expr::NumericLiteral(width));
                }
                if let Some(delay) = pulse.delay {
                    params.insert(literal!("delay"), Expr::NumericLiteral(delay));
                }
            }
        };

        SpectrePrimitive::RawInstance {
            cell: arcstr::literal!("vsource"),
            ports: vec!["p".into(), "n".into()],
            params,
        }
    }
}

/// A current probe.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Iprobe;

impl Block for Iprobe {
    type Io = TwoTerminalIo;
    type Kind = SchemaPrimitive;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("iprobe")
    }
    fn name(&self) -> arcstr::ArcStr {
        // `iprobe` is a reserved Spectre keyword,
        // so we call this block `useriprobe`.
        arcstr::format!("useriprobe")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchemaPrimitive<Iprobe> for Spectre {
    fn primitive(block: &Iprobe) -> Self::Primitive {
        SpectrePrimitive::RawInstance {
            cell: arcstr::literal!("iprobe"),
            ports: vec!["in".into(), "out".into()],
            params: IndexMap::new(),
        }
    }
}
