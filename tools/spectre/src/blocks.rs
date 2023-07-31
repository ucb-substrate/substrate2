//! Spectre-specific blocks for use in testbenches.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::io::VsourceIo;
use substrate::pdk::Pdk;
use substrate::schematic::{BlackboxContents, HasSchematicData};
use substrate::simulation::HasSimSchematic;

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
        // `vsource` is a reserved Spectre keyword,
        // so we call this block `uservsource`.
        arcstr::format!("uservsource")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematicData for Vsource {
    type Data = ();
}

impl<PDK: Pdk> HasSimSchematic<PDK, Spectre> for Vsource {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::SimCellBuilder<PDK, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let mut contents = BlackboxContents::new();
        match self {
            Self::Dc(dc) => {
                contents.push("V0 (");
                contents.push(*io.p);
                contents.push(*io.n);
                contents.push(format!(") vsource type=dc dc={}", dc));
            }
            Self::Pulse(pulse) => {
                contents.push("V0 (");
                contents.push(*io.p);
                contents.push(*io.n);
                contents.push(format!(
                    ") vsource type=pulse val0={} val1={}",
                    pulse.val0, pulse.val1
                ));
                if let Some(period) = pulse.period {
                    contents.push(format!("period={period}"));
                }
                if let Some(rise) = pulse.rise {
                    contents.push(format!("rise={rise}"));
                }
                if let Some(fall) = pulse.fall {
                    contents.push(format!("fall={fall}"));
                }
                if let Some(width) = pulse.width {
                    contents.push(format!("width={width}"));
                }
                if let Some(delay) = pulse.delay {
                    contents.push(format!("delay={delay}"));
                }
            }
        };
        cell.set_blackbox(contents);
        Ok(())
    }
}
