//! PDK corner interface.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
