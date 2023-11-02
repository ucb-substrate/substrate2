// begin-code-snippet imports
use serde::{Deserialize, Serialize};
// end-code-snippet imports

// begin-code-snippet vdivider-io
#[derive(Io, Clone, Default, Debug)]
pub struct VdividerIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub dout: Output<Signal>,
}
// end-code-snippet vdivider-io

// begin-code-snippet vdivider-struct
#[derive(Serialize, Deserialize, Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "VdividerIo")]
pub struct Vdivider {
    /// The top resistance.
    pub r1: Decimal,
    /// The bottom resistance.
    pub r2: Decimal,
}
// end-code-snippet vdivider-struct

// begin-code-snippet vdivider-schematic
impl HasSchematicData for Vdivider {
    type Data = ();
}

impl HasSchematic<Spice> for Vdivider {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Bundle,
        cell: &mut substrate::schematic::CellBuilder<Sky130CommercialPdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
    }
}
// end-code-snippet inverter-schematic
// begin-code-snippet tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn netlist_vdivider() {
        let work_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/design_inverter");
        let mut ctx = sky130_commercial_ctx();
        let script = InverterDesign {
            nw: 1_200,
            pw: (1_200..=5_000).step_by(200).collect(),
            lch: 150,
        };
        let inv = script.run(&mut ctx, work_dir);
        println!("Designed inverter:\n{:#?}", inv);
    }
}
// end-code-snippet tests
