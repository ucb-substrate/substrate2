// begin-code-snippet imports
use rust_decimal::Decimal;
use spice::{Resistor, Spice};
use substrate::{
    block::Block,
    error::Result,
    schematic::{CellBuilder, Schematic},
    types::{schematic::IoNodeBundle, InOut, Io, Output, Signal},
};
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
#[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[substrate(io = "VdividerIo")]
pub struct Vdivider {
    /// The top resistance.
    pub r1: Decimal,
    /// The bottom resistance.
    pub r2: Decimal,
}
// end-code-snippet vdivider-struct

// begin-code-snippet vdivider-schematic
impl Schematic for Vdivider {
    type Schema = Spice;
    type NestedData = ();
    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> Result<Self::NestedData> {
        let r1 = cell.instantiate(Resistor::new(self.r1));
        let r2 = cell.instantiate(Resistor::new(self.r2));

        cell.connect(io.vdd, r1.io().p);
        cell.connect(io.dout, r1.io().n);
        cell.connect(io.dout, r2.io().p);
        cell.connect(io.vss, r2.io().n);

        Ok(())
    }
}
// end-code-snippet vdivider-schematic

// begin-code-snippet tests
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use spice::netlist::NetlistOptions;
    use std::path::PathBuf;
    use substrate::context::Context;
    use substrate::schematic::netlist::ConvertibleNetlister;

    #[test]
    pub fn netlist_vdivider() {
        let ctx = Context::new();
        Spice
            .write_netlist_to_file(
                &ctx,
                Vdivider {
                    r1: dec!(100),
                    r2: dec!(200),
                },
                PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/netlist_vdivider"
                ))
                .join("vdivider.spice"),
                NetlistOptions::default(),
            )
            .expect("failed to netlist vdivider");
    }
}
// end-code-snippet tests
