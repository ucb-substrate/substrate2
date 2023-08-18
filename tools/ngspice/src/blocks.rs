//! ngspice-specific blocks for use in testbenches.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::TwoTerminalIo;
use substrate::pdk::Pdk;
use substrate::schematic::{BlackboxContents, ExportsSchematicData};
use substrate::simulation::HasSimSchematic;

use crate::Ngspice;

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
    /// Number of pulses.
    pub num_pulses: Option<Decimal>,
}

/// A voltage source.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq, Block)]
#[substrate(io = "TwoTerminalIo")]
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

impl ExportsSchematicData for Vsource {
    type Data = ();
}

impl<PDK: Pdk> HasSimSchematic<PDK, Ngspice> for Vsource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Bundle,
        cell: &mut substrate::schematic::SimCellBuilder<PDK, Ngspice, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut contents = BlackboxContents::new();
        contents.push("V0");
        contents.push(io.p);
        contents.push(io.n);

        match self {
            Self::Dc(dc) => {
                contents.push("DC");
                contents.push(format!("{}", dc));
            }
            Self::Pulse(pulse) => {
                contents.push(format!(
                    "PULSE({} {} {} {} {} {} {} {})",
                    pulse.val0,
                    pulse.val1,
                    pulse.delay.unwrap_or_default(),
                    pulse.rise.unwrap_or_default(),
                    pulse.fall.unwrap_or_default(),
                    pulse.width.unwrap_or_default(),
                    pulse.period.unwrap_or_default(),
                    pulse.num_pulses.unwrap_or_default(),
                ));
            }
        };
        cell.set_blackbox(contents);
        Ok(())
    }
}

/// A resistor.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq, Block)]
#[substrate(io = "TwoTerminalIo")]
pub struct Resistor(pub Decimal);

impl ExportsSchematicData for Resistor {
    type Data = ();
}

impl<PDK: Pdk> HasSimSchematic<PDK, Ngspice> for Resistor {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Bundle,
        cell: &mut substrate::schematic::SimCellBuilder<PDK, Ngspice, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut contents = BlackboxContents::new();
        contents.push("R0");
        contents.push(io.p);
        contents.push(io.n);
        contents.push(format!("{}", self.0));
        cell.set_blackbox(contents);
        Ok(())
    }
}
