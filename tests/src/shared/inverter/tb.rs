use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sky130pdk::corner::Sky130Corner;
use sky130pdk::Sky130CommercialPdk;
use spectre::blocks::{Pulse, Vsource};
use spectre::{Options, Spectre, Tran};
use substrate::block::Block;
use substrate::io::Node;
use substrate::ios::TestbenchIo;
use substrate::pdk::corner::{InstallCorner, Pvt};
use substrate::schematic::{Cell, HasSchematic};
use substrate::simulation::{HasTestbenchSchematicImpl, Testbench};

use super::Inverter;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InverterTb {
    pvt: Pvt<Sky130Corner>,
}

impl InverterTb {
    #[inline]
    pub fn new(pvt: Pvt<Sky130Corner>) -> Self {
        Self { pvt }
    }
}

impl Block for InverterTb {
    type Io = TestbenchIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter_tb")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("inverter_tb")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for InverterTb {
    type Data = Node;
}

impl HasTestbenchSchematicImpl<Sky130CommercialPdk, Spectre> for InverterTb {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::TestbenchCellBuilder<Sky130CommercialPdk, Spectre, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let inv = cell.instantiate(Inverter {
            nw: 1_200,
            pw: 2_000,
            lch: 150,
        });
        let vdd = cell.instantiate_tb(Vsource::pulse(Pulse {
            val0: 0.into(),
            val1: self.pvt.voltage,
            delay: Some(dec!(0.1e-9)),
            width: Some(dec!(1e-9)),
            fall: Some(dec!(1e-15)),
            rise: Some(dec!(1e-15)),
            period: None,
        }));
        cell.connect(vdd.io().p, inv.io().vdd);
        cell.connect(vdd.io().n, io.vss);
        cell.connect(inv.io().vss, io.vss);
        Ok(*inv.io().dout)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdividerTbData {
    pub vdd: Vec<f64>,
    pub out: Vec<f64>,
}

impl Testbench<Sky130CommercialPdk, Spectre> for InverterTb {
    type Output = ();
    fn run(
        &self,
        _cell: &Cell<Self>,
        sim: substrate::simulation::SimController<Sky130CommercialPdk, Spectre>,
    ) -> Self::Output {
        let mut opts = Options::default();
        sim.pdk.pdk.install_corner(Sky130Corner::Tt, &mut opts);
        let output = sim
            .simulate(
                opts,
                Tran {
                    stop: dec!(2e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");

        println!("Output:\n{:?}", output);

        // VdividerTbData {
        //     vdd: output.get_data(&cell.data().io().pwr.vdd).unwrap().clone(),
        //     out: output.get_data(&cell.data().io().out).unwrap().clone(),
        // }
    }
}
