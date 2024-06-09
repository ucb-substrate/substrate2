//! Schematic primitives.

use rust_decimal::Decimal;
use scir::ParamValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use substrate::schematic::ExportsNestedData;

use crate::arcstr;
use crate::arcstr::ArcStr;
use crate::block::Block;
use crate::io::{Array, InOut, Signal, TwoTerminalIo};

/// An instance with a pre-defined cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawInstance {
    /// The name of the underlying cell.
    pub cell: ArcStr,
    /// The name of the ports of the underlying cell.
    pub ports: Vec<ArcStr>,
    /// The parameters to pass to the instance.
    pub params: HashMap<ArcStr, ParamValue>,
}

impl Hash for RawInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cell.hash(state);
        self.ports.hash(state);
        self.params.iter().collect::<Vec<_>>().hash(state);
    }
}

impl RawInstance {
    /// Create a new raw instance with the given parameters.
    #[inline]
    pub fn with_params(
        cell: ArcStr,
        ports: Vec<ArcStr>,
        params: impl Into<HashMap<ArcStr, ParamValue>>,
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
            params: HashMap::new(),
        }
    }
}
impl Block for RawInstance {
    type Io = InOut<Array<Signal>>;

    fn id() -> ArcStr {
        arcstr::literal!("raw_instance")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("raw_instance_{}", self.cell)
    }

    fn io(&self) -> Self::Io {
        InOut(Array::new(self.ports.len(), Default::default()))
    }
}
impl ExportsNestedData for RawInstance {
    type NestedData = ();
}

/// An ideal 2-terminal resistor.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

    fn id() -> ArcStr {
        arcstr::literal!("ideal_resistor")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("ideal_resistor_{}", self.value)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Resistor {
    type NestedData = ();
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

    fn id() -> ArcStr {
        arcstr::literal!("capacitor")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("capacitor_{}", self.value)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}
impl ExportsNestedData for Capacitor {
    type NestedData = ();
}

/// An ideal 2-terminal DC voltage source.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcVsource {
    /// The voltage of the source.
    value: Decimal,
}

impl DcVsource {
    /// Create a new DC voltage source.
    #[inline]
    pub fn new(value: impl Into<Decimal>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// The value of the voltage source.
    #[inline]
    pub fn value(&self) -> Decimal {
        self.value
    }
}

impl Block for DcVsource {
    type Io = TwoTerminalIo;

    fn id() -> ArcStr {
        arcstr::literal!("vsource")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("vsource_{}", self.value)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for DcVsource {
    type NestedData = ();
}
