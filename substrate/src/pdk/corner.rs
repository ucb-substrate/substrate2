//! PDK corner interface.

use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
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

/// A corner in a given PDK.
///
/// Corners are expected to be cheaply cloneable, and ideally copy.
/// For example, a corner may simply be an enum variant with no inner fields.
pub trait Corner: Clone + Serialize + DeserializeOwned {}
impl<T: Clone + Serialize + DeserializeOwned> Corner for T {}
