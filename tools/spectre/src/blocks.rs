//! Spectre-specific blocks for use in testbenches.

use rust_decimal::Decimal;
use scir::ParamValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use substrate::block::Block;
use substrate::io::{SchematicType, TwoTerminalIo};
use substrate::schematic::primitives::DcVsource;
use substrate::schematic::{CellBuilder, ExportsNestedData, PrimitiveBinding, Schematic};

use crate::{Primitive, Spectre};

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

impl ExportsNestedData for Vsource {
    type NestedData = ();
}

impl Schematic<Spectre> for Vsource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        use arcstr::literal;
        let mut params = HashMap::new();
        match self {
            Vsource::Dc(dc) => {
                params.insert(literal!("type"), ParamValue::String(literal!("dc")));
                params.insert(literal!("dc"), ParamValue::Numeric(*dc));
            }
            Vsource::Pulse(pulse) => {
                params.insert(literal!("type"), ParamValue::String(literal!("pulse")));
                params.insert(literal!("val0"), ParamValue::Numeric(pulse.val0));
                params.insert(literal!("val1"), ParamValue::Numeric(pulse.val1));
                if let Some(period) = pulse.period {
                    params.insert(literal!("period"), ParamValue::Numeric(period));
                }
                if let Some(rise) = pulse.rise {
                    params.insert(literal!("rise"), ParamValue::Numeric(rise));
                }
                if let Some(fall) = pulse.fall {
                    params.insert(literal!("fall"), ParamValue::Numeric(fall));
                }
                if let Some(width) = pulse.width {
                    params.insert(literal!("width"), ParamValue::Numeric(width));
                }
                if let Some(delay) = pulse.delay {
                    params.insert(literal!("delay"), ParamValue::Numeric(delay));
                }
            }
        };

        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("vsource"),
            ports: vec!["p".into(), "n".into()],
            params,
        });
        prim.connect("p", io.p);
        prim.connect("n", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}

impl Schematic<Spectre> for DcVsource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        cell.flatten();
        cell.instantiate_connected(Vsource::dc(self.value()), io);
        Ok(())
    }
}

/// A current probe.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Iprobe;

impl Block for Iprobe {
    type Io = TwoTerminalIo;

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

impl ExportsNestedData for Iprobe {
    type NestedData = ();
}

impl Schematic<Spectre> for Iprobe {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("iprobe"),
            ports: vec!["in".into(), "out".into()],
            params: HashMap::new(),
        });
        prim.connect("in", io.p);
        prim.connect("out", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}
