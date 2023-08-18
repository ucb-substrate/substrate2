//! Schematic primitives.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::arcstr;
use crate::arcstr::ArcStr;
use crate::block::Block;
use crate::io::TwoTerminalIo;
use crate::pdk::Pdk;

use super::{ExportsSchematicData, PrimitiveDeviceKind, PrimitiveNode, Schematic};

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
    const FLATTEN: bool = true;

    fn id() -> ArcStr {
        arcstr::literal!("resistor")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("resistor_{}", self.value)
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}
impl ExportsSchematicData for Resistor {
    type Data = ();
}
impl<PDK: Pdk> Schematic<PDK> for Resistor {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as crate::io::SchematicType>::Bundle,
        cell: &mut super::CellBuilder<PDK, Self>,
    ) -> crate::error::Result<Self::Data> {
        cell.add_primitive(
            PrimitiveDeviceKind::Res2 {
                pos: PrimitiveNode::new("p", io.p),
                neg: PrimitiveNode::new("n", io.n),
                value: self.value,
            }
            .into(),
        );
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
    const FLATTEN: bool = true;

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
impl ExportsSchematicData for Capacitor {
    type Data = ();
}
impl<PDK: Pdk> Schematic<PDK> for Capacitor {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as crate::io::SchematicType>::Bundle,
        cell: &mut super::CellBuilder<PDK, Self>,
    ) -> crate::error::Result<Self::Data> {
        cell.add_primitive(
            PrimitiveDeviceKind::Cap2 {
                pos: PrimitiveNode::new("p", io.p),
                neg: PrimitiveNode::new("n", io.n),
                value: self.value,
            }
            .into(),
        );
        Ok(())
    }
}
