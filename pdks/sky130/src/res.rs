//! Resistors.

use substrate::layout::Layout;

pub struct PrecisionResistor {
    pub width: PrecisionResistorWidth,
    pub length: i64,
}

pub enum PrecisionResistorWidth {
    /// 0.35um width.
    W035,
    /// 0.69um width.
    W069,
    /// 1.41um width.
    W141,
    /// 2.85um width.
    W285,
    /// 5.73um width.
    W573,
}

impl Layout for PrecisionResistor {}
