//! PDK corner interface.

use arcstr::ArcStr;
use rust_decimal::Decimal;

use crate::simulation::Simulator;

use super::Pdk;

/// A process-voltage-temperature corner.
///
/// Contains a process corner, a voltage, and a temperature (in Celsius).
pub struct Pvt<C> {
    /// The process corner.
    pub corner: C,
    /// The voltage.
    pub voltage: Decimal,
    /// The temperature, in degrees celsius.
    pub temp: Decimal,
}

/// A process-voltage-temperature corner reference.
///
/// Contains a process corner, a voltage, and a temperature (in Celsius).
pub struct PvtRef {
    /// The name of the process corner.
    ///
    /// The actual process corner used is governed by the implementation
    /// of [`Pdk::corner`].
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
pub trait Corner: Clone {
    /// The name of the corner.
    fn name(&self) -> ArcStr;
}

/// A PDK with process corners compatible with simulator `S`.
pub trait InstallCorner<S: Simulator>: Pdk {
    /// Install the given process corner in the given simulator.
    ///
    /// A typical corner installation involves telling the simulator to include
    /// process-specific model files. However, corners are free to configure
    /// other simulation options as well.
    fn install_corner(&self, corner: impl AsRef<<Self as Pdk>::Corner>, opts: &mut S::Options);
}

// pdk.install_corner::<Spectre>

// impl InstallCorner<Spectre> for Sky130Corner {
//
// }
