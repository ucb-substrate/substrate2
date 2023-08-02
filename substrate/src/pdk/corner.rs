//! PDK corner interface.

use arcstr::ArcStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::simulation::Simulator;

use super::Pdk;

/// A process-voltage-temperature corner.
///
/// Contains a process corner, a voltage, and a temperature (in Celsius).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Pvt<C> {
    /// The process corner.
    pub corner: C,
    /// The voltage.
    pub voltage: Decimal,
    /// The temperature, in degrees celsius.
    pub temp: Decimal,
}

impl<C> Pvt<C> {
    /// Create a new PVT corner.
    #[inline]
    pub fn new(corner: C, voltage: Decimal, temp: Decimal) -> Self {
        Self {
            corner,
            voltage,
            temp,
        }
    }
}

/// A process-voltage-temperature corner reference.
///
/// Contains a process corner, a voltage, and a temperature (in Celsius).
pub struct PvtRef {
    /// The name of the process corner.
    pub corner: ArcStr,
    /// The voltage.
    pub voltage: Decimal,
    /// The temperature, in degrees celsius.
    pub temp: Decimal,
}

/// A corner in a given PDK.
///
/// Corners are expected to be cheaply cloneable, and ideally copy.
/// For example, a corner may simply be an enum variant with no inner fields.
pub trait Corner: Clone + Serialize + Deserialize<'static> {}

/// A PDK with process corners compatible with simulator `S`.
pub trait InstallCorner<S: Simulator>: Pdk {
    /// Install the given process corner in the given simulator.
    ///
    /// A typical corner installation involves telling the simulator to include
    /// process-specific model files. However, corners are free to configure
    /// other simulation options as well.
    fn install_corner(&self, corner: &<Self as Pdk>::Corner, opts: &mut S::Options);
}
