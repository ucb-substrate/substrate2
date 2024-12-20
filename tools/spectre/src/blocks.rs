//! Spectre-specific blocks for use in testbenches.

use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::ParamValue;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use substrate::block::Block;
use substrate::schematic::{CellBuilder, PrimitiveBinding, Schematic};
use substrate::simulation::waveform::{TimeWaveform, Waveform};
use substrate::types::{Array, InOut, Io, Signal, TwoTerminalIo};

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

    fn name(&self) -> arcstr::ArcStr {
        // `vsource` is a reserved Spectre keyword,
        // so we call this block `uservsource`.
        arcstr::format!("uservsource")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Schematic for Vsource {
    type Schema = Spectre;
    type NestedData = ();

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        use arcstr::literal;
        let mut params = Vec::new();
        match self {
            Vsource::Dc(dc) => {
                params.push((literal!("type"), ParamValue::String(literal!("dc"))));
                params.push((literal!("dc"), ParamValue::Numeric(*dc)));
            }
            Vsource::Pulse(pulse) => {
                params.push((literal!("type"), ParamValue::String(literal!("pulse"))));
                params.push((literal!("val0"), ParamValue::Numeric(pulse.val0)));
                params.push((literal!("val1"), ParamValue::Numeric(pulse.val1)));
                if let Some(period) = pulse.period {
                    params.push((literal!("period"), ParamValue::Numeric(period)));
                }
                if let Some(rise) = pulse.rise {
                    params.push((literal!("rise"), ParamValue::Numeric(rise)));
                }
                if let Some(fall) = pulse.fall {
                    params.push((literal!("fall"), ParamValue::Numeric(fall)));
                }
                if let Some(width) = pulse.width {
                    params.push((literal!("width"), ParamValue::Numeric(width)));
                }
                if let Some(delay) = pulse.delay {
                    params.push((literal!("delay"), ParamValue::Numeric(delay)));
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
                params.push((literal!("type"), ParamValue::String(literal!("pwl"))));
                params.push((literal!("wave"), ParamValue::String(pwl.into())));
            }
            Vsource::Ac(ac) => {
                params.push((literal!("type"), ParamValue::String(literal!("dc"))));
                params.push((literal!("dc"), ParamValue::Numeric(ac.dc)));
                params.push((literal!("mag"), ParamValue::Numeric(ac.mag)));
                params.push((literal!("phase"), ParamValue::Numeric(ac.phase)));
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

    fn name(&self) -> arcstr::ArcStr {
        // `isource` is a reserved Spectre keyword,
        // so we call this block `userisource`.
        arcstr::format!("userisource")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Schematic for Isource {
    type Schema = Spectre;
    type NestedData = ();

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        use arcstr::literal;
        let mut params = Vec::new();
        match self {
            Isource::Dc(dc) => {
                params.push((literal!("type"), ParamValue::String(literal!("dc"))));
                params.push((literal!("dc"), ParamValue::Numeric(*dc)));
            }
            Isource::Pulse(pulse) => {
                params.push((literal!("type"), ParamValue::String(literal!("pulse"))));
                params.push((literal!("val0"), ParamValue::Numeric(pulse.val0)));
                params.push((literal!("val1"), ParamValue::Numeric(pulse.val1)));
                if let Some(period) = pulse.period {
                    params.push((literal!("period"), ParamValue::Numeric(period)));
                }
                if let Some(rise) = pulse.rise {
                    params.push((literal!("rise"), ParamValue::Numeric(rise)));
                }
                if let Some(fall) = pulse.fall {
                    params.push((literal!("fall"), ParamValue::Numeric(fall)));
                }
                if let Some(width) = pulse.width {
                    params.push((literal!("width"), ParamValue::Numeric(width)));
                }
                if let Some(delay) = pulse.delay {
                    params.push((literal!("delay"), ParamValue::Numeric(delay)));
                }
            }
            Isource::Ac(ac) => {
                params.push((literal!("type"), ParamValue::String(literal!("dc"))));
                params.push((literal!("dc"), ParamValue::Numeric(ac.dc)));
                params.push((literal!("mag"), ParamValue::Numeric(ac.mag)));
                params.push((literal!("phase"), ParamValue::Numeric(ac.phase)));
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

    fn name(&self) -> arcstr::ArcStr {
        // `iprobe` is a reserved Spectre keyword,
        // so we call this block `useriprobe`.
        arcstr::format!("useriprobe")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Schematic for Iprobe {
    type Schema = Spectre;
    type NestedData = ();
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("iprobe"),
            ports: vec!["in".into(), "out".into()],
            params: Vec::new(),
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
#[derive(Clone, Debug, Io)]
pub struct NportIo {
    /// The ports.
    ///
    /// Each port contains two signals: a p terminal and an n terminal.
    pub ports: Array<TwoTerminalIo>,
}

impl Block for Nport {
    type Io = NportIo;

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

impl Schematic for Nport {
    type Schema = Spectre;
    type NestedData = ();
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("nport"),
            ports: (1..=self.ports)
                .flat_map(|i| [arcstr::format!("t{i}"), arcstr::format!("b{i}")])
                .collect(),
            params: Vec::from_iter([(
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

/// An ideal 2-terminal resistor.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Resistor {
    /// The resistor value.
    value: Decimal,
}
impl Resistor {
    /// Create a new resistor with the given value.
    #[inline]
    pub fn new(value: impl Into<Decimal>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// The value of the resistor.
    #[inline]
    pub fn value(&self) -> Decimal {
        self.value
    }
}
impl Block for Resistor {
    type Io = TwoTerminalIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("ideal_resistor_{}", self.value)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Schematic for Resistor {
    type Schema = Spectre;
    type NestedData = ();
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("resistor"),
            ports: vec![arcstr::literal!("1"), arcstr::literal!("2")],
            params: Vec::from_iter([(arcstr::literal!("r"), ParamValue::Numeric(self.value()))]),
        });
        prim.connect("1", io.p);
        prim.connect("2", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// An ideal 2-terminal capacitor.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capacitor {
    /// The resistor value.
    value: Decimal,
}

impl Capacitor {
    /// Create a new capacitor with the given value.
    #[inline]
    pub fn new(value: impl Into<Decimal>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// The value of the capacitor.
    #[inline]
    pub fn value(&self) -> Decimal {
        self.value
    }
}

impl Block for Capacitor {
    type Io = TwoTerminalIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("capacitor_{}", self.value)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Schematic for Capacitor {
    type Schema = Spectre;
    type NestedData = ();
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: arcstr::literal!("capacitor"),
            ports: vec![arcstr::literal!("1"), arcstr::literal!("2")],
            params: Vec::from_iter([(arcstr::literal!("c"), ParamValue::Numeric(self.value()))]),
        });
        prim.connect("1", io.p);
        prim.connect("2", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}

/// An instance with a pre-defined cell.
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct RawInstance {
    /// The name of the underlying cell.
    pub cell: ArcStr,
    /// The name of the ports of the underlying cell.
    pub ports: Vec<ArcStr>,
    /// The parameters to pass to the instance.
    pub params: Vec<(ArcStr, ParamValue)>,
}

impl RawInstance {
    /// Create a new raw instance with the given parameters.
    #[inline]
    pub fn with_params(
        cell: ArcStr,
        ports: Vec<ArcStr>,
        params: impl Into<Vec<(ArcStr, ParamValue)>>,
    ) -> Self {
        Self {
            cell,
            ports,
            params: params.into(),
        }
    }
    /// Create a new raw instance with no parameters.
    #[inline]
    pub fn new(cell: ArcStr, ports: Vec<ArcStr>) -> Self {
        Self {
            cell,
            ports,
            params: Vec::new(),
        }
    }
}
impl Block for RawInstance {
    type Io = InOut<Array<Signal>>;

    fn name(&self) -> ArcStr {
        arcstr::format!("raw_instance_{}", self.cell)
    }

    fn io(&self) -> Self::Io {
        InOut(Array::new(self.ports.len(), Default::default()))
    }
}

impl Schematic for RawInstance {
    type Schema = Spectre;
    type NestedData = ();
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::RawInstance {
            cell: self.cell.clone(),
            ports: self.ports.clone(),
            params: self.params.clone(),
        });
        for (i, port) in self.ports.iter().enumerate() {
            prim.connect(port, io[i]);
        }
        cell.set_primitive(prim);
        Ok(())
    }
}
