//! Spectre-specific blocks for use in testbenches.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::ios::VsourceIo;
use substrate::pdk::Pdk;
use substrate::schematic::HasSchematic;
use substrate::simulation::HasTestbenchSchematicImpl;

use crate::Spectre;

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
///
/// Currently only spuports DC values.
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
    type Io = VsourceIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("vsource")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("uservsource")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for Vsource {
    type Data = ();
}

impl<PDK: Pdk> HasTestbenchSchematicImpl<PDK, Spectre> for Vsource {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::TestbenchCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
        use std::fmt::Write;
        let contents = match self {
            Self::Dc(dc) => format!("V0 ( io_p io_n ) vsource type=dc dc={}", dc),
            Self::Pulse(pulse) => {
                let mut s = String::new();
                write!(
                    &mut s,
                    "V0 ( io_p io_n ) vsource type=pulse val0={} val1={}",
                    pulse.val0, pulse.val1
                )
                .unwrap();
                if let Some(period) = pulse.period {
                    write!(&mut s, " period={period}").unwrap();
                }
                if let Some(rise) = pulse.rise {
                    write!(&mut s, " rise={rise}").unwrap();
                }
                if let Some(fall) = pulse.fall {
                    write!(&mut s, " fall={fall}").unwrap();
                }
                if let Some(width) = pulse.width {
                    write!(&mut s, " width={width}").unwrap();
                }
                if let Some(delay) = pulse.delay {
                    write!(&mut s, " delay={delay}").unwrap();
                }
                s
            }
        };
        cell.set_blackbox(contents);
        Ok(())
    }
}
