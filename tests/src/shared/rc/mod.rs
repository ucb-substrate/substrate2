use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::tran::Tran;
use spectre::{Options, Spectre};
use substrate::block::{self, Block};
use substrate::io::{Node, Signal};
use substrate::io::{SchematicType, TestbenchIo};
use substrate::pdk::corner::SupportsSimulator;
use substrate::pdk::Pdk;
use substrate::schematic::primitives::{Capacitor, Resistor};
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellBuilder, CellSchematic, ExportsNestedData, Schematic};
use substrate::simulation::data::HasSimData;
use substrate::simulation::Testbench;

/// An RC testbench.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo", kind = "Cell")]
pub struct RcTb {
    ic: Decimal,
}

impl RcTb {
    /// Create a new RC testbench with the given initial capacitor value.
    #[inline]
    pub fn new(ic: Decimal) -> Self {
        Self { ic }
    }
}

impl ExportsNestedData for RcTb {
    type NestedData = Node;
}

impl<PDK: Pdk> CellSchematic<PDK, Spectre> for RcTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<PDK, Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vout = cell.signal("vout", Signal);

        let r = cell.instantiate(Resistor::new(dec!(1000)));
        cell.connect(r.io().p, vout);
        cell.connect(r.io().n, io.vss);

        let c = cell.instantiate(Capacitor::new(dec!(1e-9)));
        cell.connect(c.io().p, vout);
        cell.connect(c.io().n, io.vss);

        Ok(vout)
    }
}

impl<PDK: Pdk + SupportsSimulator<Spectre>> Testbench<PDK, Spectre> for RcTb {
    type Output = (f64, f64);
    fn run(&self, sim: substrate::simulation::SimController<PDK, Spectre, Self>) -> Self::Output {
        let mut opts = Options::default();
        sim.set_initial_condition(sim.tb.data(), self.ic, &mut opts);
        let output = sim
            .simulate_default(
                opts,
                None,
                Tran {
                    stop: dec!(10e-6),
                    ..Default::default()
                },
            )
            .unwrap();

        let vout = output.get_data(&sim.tb.data()).unwrap();

        let first = vout.first().unwrap();
        let last = vout.last().unwrap();
        (*first, *last)
    }
}
