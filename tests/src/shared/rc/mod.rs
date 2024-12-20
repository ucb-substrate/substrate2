use num::complex::Complex64;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::analysis::ac::{Ac, Sweep};
use spectre::analysis::tran::Tran;
use spectre::blocks::{AcSource, Isource};
use spectre::{ErrPreset, Options, Spectre};
use substrate::block::Block;
use substrate::io::schematic::{HardwareType, Node};
use substrate::io::Signal;
use substrate::io::TestbenchIo;
use substrate::schematic::primitives::{Capacitor, Resistor};
use substrate::schematic::{Cell, CellBuilder, ExportsNestedData, Schematic};
use substrate::simulation::data::{ac, tran, FromSaved, Save, SaveTb};
use substrate::simulation::options::ic;
use substrate::simulation::options::ic::InitialCondition;
use substrate::simulation::{SimulationContext, Simulator, Testbench};

/// An RC testbench.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
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
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spectre>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vout = cell.signal("vout", Signal);

        let r = cell.instantiate(Resistor::new(dec!(1000)));
        cell.connect(r.io().p, vout);
        cell.connect(r.io().n, io.vss);

        let c = cell.instantiate(Capacitor::new(dec!(1e-9)));
        cell.connect(c.io().p, vout);
        cell.connect(c.io().n, io.vss);

        let isource = cell.instantiate(Isource::ac(AcSource {
            dc: dec!(0),
            mag: dec!(1),
            phase: dec!(0),
        }));
        cell.connect(isource.io().p, vout);
        cell.connect(isource.io().n, io.vss);

        Ok(vout)
    }
}

impl SaveTb<Spectre, (Tran, Ac), (tran::Voltage, ac::Voltage)> for RcTb {
    fn save_tb(
        ctx: &SimulationContext<Spectre>,
        cell: &Cell<Self>,
        opts: &mut <Spectre as Simulator>::Options,
    ) -> <(tran::Voltage, ac::Voltage) as FromSaved<Spectre, (Tran, Ac)>>::SavedKey {
        (
            tran::Voltage::save(ctx, cell.data(), opts),
            ac::Voltage::save(ctx, cell.data(), opts),
        )
    }
}

impl Testbench<Spectre> for RcTb {
    type Output = (f64, f64, Complex64);
    fn run(&self, sim: substrate::simulation::SimController<Spectre, Self>) -> Self::Output {
        let mut opts = Options::default();
        sim.set_option(
            InitialCondition {
                path: sim.tb.data(),
                value: ic::Voltage(self.ic),
            },
            &mut opts,
        );
        let (tran_vout, ac_vout) = sim
            .simulate(
                opts,
                (
                    Tran {
                        stop: dec!(10e-6),
                        ..Default::default()
                    },
                    Ac {
                        start: dec!(1e6),
                        stop: dec!(2e6),
                        sweep: Sweep::Linear(10),
                        errpreset: Some(ErrPreset::Conservative),
                    },
                ),
            )
            .unwrap();

        let first = tran_vout.first().unwrap();
        let last = tran_vout.last().unwrap();
        (*first, *last, ac_vout[2])
    }
}
