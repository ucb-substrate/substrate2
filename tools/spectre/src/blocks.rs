//! Spectre-specific blocks for use in testbenches.

use rust_decimal::Decimal;
use scir::ParamValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::io::{Array, Io, TwoTerminalIo};
use substrate::schematic::primitives::DcVsource;
use substrate::schematic::{CellBuilder, ExportsNestedData, PrimitiveBinding, Schematic};
use substrate::simulation::waveform::{TimeWaveform, Waveform};

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
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum Vsource {
    /// A dc voltage source.
    Dc(Decimal),
    /// An AC small-signal current source.
    Ac(AcSource),
    /// A pulse voltage source.
    Pulse(Pulse),
    /// A piecewise linear source
    Pwl(Waveform<Decimal>),
}

impl Vsource {
    /// Creates a new DC voltage source.
    #[inline]
    pub fn dc(value: Decimal) -> Self {
        Self::Dc(value)
    }

    /// Creates a new pulse voltage source.
    #[inline]
    pub fn pulse(value: Pulse) -> Self {
        Self::Pulse(value)
    }

    /// Creates a new piecewise linear voltage source.
    #[inline]
    pub fn pwl(value: Waveform<Decimal>) -> Self {
        Self::Pwl(value)
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
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
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
            Vsource::Pwl(waveform) => {
                let mut pwl = String::new();
                pwl.push('[');
                for (i, pt) in waveform.values().enumerate() {
                    use std::fmt::Write;
                    if i != 0 {
                        pwl.push(' ');
                    }
                    write!(&mut pwl, "{} {}", pt.t(), pt.x()).unwrap();
                }
                pwl.push(']');
                params.insert(literal!("type"), ParamValue::String(literal!("pwl")));
                params.insert(literal!("wave"), ParamValue::String(pwl.into()));
            }
            Vsource::Ac(ac) => {
                params.insert(literal!("type"), ParamValue::String(literal!("dc")));
                params.insert(literal!("dc"), ParamValue::Numeric(ac.dc));
                params.insert(literal!("mag"), ParamValue::Numeric(ac.mag));
                params.insert(literal!("phase"), ParamValue::Numeric(ac.phase));
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
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        cell.flatten();
        cell.instantiate_connected(Vsource::dc(self.value()), io);
        Ok(())
    }
}

/// An AC source.
#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AcSource {
    /// The DC value.
    pub dc: Decimal,
    /// The magnitude.
    pub mag: Decimal,
    /// The phase, **in degrees*.
    pub phase: Decimal,
}

/// A current source.
///
/// Positive current is drawn from the `p` node and enters the `n` node.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Isource {
    /// A DC current source.
    Dc(Decimal),
    /// An AC small-signal current source.
    Ac(AcSource),
    /// A pulse current source.
    Pulse(Pulse),
}

impl Isource {
    /// Creates a new DC current source.
    #[inline]
    pub fn dc(value: Decimal) -> Self {
        Self::Dc(value)
    }

    /// Creates a new pulse current source.
    #[inline]
    pub fn pulse(value: Pulse) -> Self {
        Self::Pulse(value)
    }

    /// Creates a new AC current source.
    #[inline]
    pub fn ac(value: AcSource) -> Self {
        Self::Ac(value)
    }
}

impl Block for Isource {
    type Io = TwoTerminalIo;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("isource")
    }
    fn name(&self) -> arcstr::ArcStr {
        // `isource` is a reserved Spectre keyword,
        // so we call this block `userisource`.
        arcstr::format!("userisource")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Isource {
    type NestedData = ();
}

impl Schematic<Spectre> for Isource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        use arcstr::literal;
        let mut params = HashMap::new();
        match self {
            Isource::Dc(dc) => {
                params.insert(literal!("type"), ParamValue::String(literal!("dc")));
                params.insert(literal!("dc"), ParamValue::Numeric(*dc));
            }
            Isource::Pulse(pulse) => {
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
            Isource::Ac(ac) => {
                params.insert(literal!("type"), ParamValue::String(literal!("dc")));
                params.insert(literal!("dc"), ParamValue::Numeric(ac.dc));
                params.insert(literal!("mag"), ParamValue::Numeric(ac.mag));
                params.insert(literal!("phase"), ParamValue::Numeric(ac.phase));
            }
        };

        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("isource"),
            ports: vec!["p".into(), "n".into()],
            params,
        });
        prim.connect("p", io.p);
        prim.connect("n", io.n);
        cell.set_primitive(prim);
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
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
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

/// An n-port black box.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Nport {
    parameter_file: PathBuf,
    ports: usize,
}

impl Nport {
    /// Creates a new n-port with the given parameter file and number of ports.
    pub fn new(ports: usize, parameter_file: impl Into<PathBuf>) -> Self {
        Self {
            parameter_file: parameter_file.into(),
            ports,
        }
    }
}

/// The interface of an [`Nport`].
#[derive(Io, Clone, Debug)]
pub struct NportIo {
    /// The ports.
    ///
    /// Each port contains two signals: a p terminal and an n terminal.
    pub ports: Array<TwoTerminalIo>,
}

impl Block for Nport {
    type Io = NportIo;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("nport")
    }
    fn name(&self) -> arcstr::ArcStr {
        // `nport` is a reserved Spectre keyword,
        // so we call this block `usernport`.
        arcstr::format!("usernport")
    }
    fn io(&self) -> Self::Io {
        NportIo {
            ports: Array::new(self.ports, Default::default()),
        }
    }
}

impl ExportsNestedData for Nport {
    type NestedData = ();
}

impl Schematic<Spectre> for Nport {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("nport"),
            ports: (1..=self.ports)
                .flat_map(|i| [arcstr::format!("t{i}"), arcstr::format!("b{i}")])
                .collect(),
            params: HashMap::from_iter([(
                arcstr::literal!("file"),
                ParamValue::String(arcstr::format!("{:?}", self.parameter_file)),
            )]),
        });
        for i in 0..self.ports {
            prim.connect(arcstr::format!("t{}", i + 1), io.ports[i].p);
            prim.connect(arcstr::format!("b{}", i + 1), io.ports[i].n);
        }
        cell.set_primitive(prim);
        Ok(())
    }
}
