use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::tran::Tran;
use spectre::{Options, Spectre};
use substrate::block::Block;
use substrate::io::{Node, Signal};
use substrate::io::{SchematicType, TestbenchIo};
use substrate::schematic::primitives::{Capacitor, Resistor};
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, Schematic};
use substrate::simulation::data::{tran, FromSaved, Save};
use substrate::simulation::options::ic;
use substrate::simulation::options::ic::InitialCondition;
use substrate::simulation::{SimulationContext, Simulator, Testbench};

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

impl Schematic<Spectre> for RcTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as SchematicType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
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

#[derive(FromSaved, Serialize, Deserialize)]
pub struct RcTbOutput {
    pub vout: tran::Voltage,
}

impl Save<Spectre, Tran, &Cell<RcTb>> for RcTbOutput {
    fn save(
        ctx: &SimulationContext<Spectre>,
        to_save: &Cell<RcTb>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> Self::Key {
        Self::Key {
            vout: tran::Voltage::save(ctx, to_save.data(), opts),
        }
    }
}

impl Testbench<Spectre> for RcTb {
    type Output = (f64, f64);
    fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
        let mut opts = Options::default();
        sim.set_option(
            InitialCondition {
                path: sim.tb.data(),
                value: ic::Voltage(self.ic),
            },
            &mut opts,
        );
        let output: RcTbOutput = sim
            .simulate(
                opts,
                Tran {
                    stop: dec!(10e-6),
                    ..Default::default()
                },
            )
            .unwrap();

        let first = output.vout.first().unwrap();
        let last = output.vout.last().unwrap();
        (*first, *last)
    }
}
