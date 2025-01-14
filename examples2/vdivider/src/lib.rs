use arcstr::ArcStr;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use spectre::blocks::{Iprobe, Resistor, Vsource};
use spectre::Spectre;
use substrate::block::Block;
use substrate::schematic::{CellBuilder, Instance, NestedData, Schematic};
use substrate::types::codegen::{NestedNodeBundle, NestedTerminalBundle};
use substrate::types::{Array, InOut, Io, Output, PowerIo, Signal, TestbenchIo};

#[derive(Debug, Default, Clone, Io)]
pub struct VdividerIo {
    pub pwr: PowerIo,
    pub out: Output<Signal>,
}

#[derive(Debug, Default, Clone, Io)]
pub struct VdividerFlatIo {
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
    pub out: Output<Signal>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vdivider {
    pub r1: Resistor,
    pub r2: Resistor,
}

impl Vdivider {
    #[inline]
    pub fn new(r1: impl Into<Decimal>, r2: impl Into<Decimal>) -> Self {
        Self {
            r1: Resistor::new(r1),
            r2: Resistor::new(r2),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VdividerArray {
    pub vdividers: Vec<Vdivider>,
}

impl Block for Vdivider {
    type Io = VdividerIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_{}_{}", self.r1.value(), self.r2.value())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(Debug, Clone, Io)]
pub struct VdividerArrayIo {
    pub elements: Array<PowerIo>,
}

impl Block for VdividerArray {
    type Io = VdividerArrayIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_array_{}", self.vdividers.len())
    }

    fn io(&self) -> Self::Io {
        VdividerArrayIo {
            elements: Array::new(self.vdividers.len(), Default::default()),
        }
    }
}

#[derive(NestedData)]
pub struct VdividerData {
    r1: Instance<Resistor>,
    r2: Instance<Resistor>,
}

impl Schematic for Vdivider {
    type Schema = Spectre;
    type NestedData = VdividerData;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let r1 = cell.instantiate(self.r1);
        let r2 = cell.instantiate(self.r2);

        cell.connect(io.pwr.vdd, r1.io().p);
        cell.connect(io.out, r1.io().n);
        cell.connect(io.out, r2.io().p);
        cell.connect(io.pwr.vss, r2.io().n);
        Ok(VdividerData { r1, r2 })
    }
}

impl Schematic for VdividerArray {
    type Schema = Spectre;
    type NestedData = Vec<Instance<Vdivider>>;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut vdividers = Vec::new();

        for (i, vdivider) in self.vdividers.iter().enumerate() {
            let vdiv = cell.instantiate(*vdivider);

            cell.connect(&vdiv.io().pwr, &io.elements[i]);

            vdividers.push(vdiv);
        }

        Ok(vdividers)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct VdividerTb;

#[derive(NestedData)]
pub struct VdividerTbData {
    iprobe: Instance<Iprobe>,
    dut: Instance<Vdivider>,
}

impl Schematic for VdividerTb {
    type Schema = Spectre;
    type NestedData = VdividerTbData;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vdd_a = cell.signal("vdd_a", Signal);
        let vdd = cell.signal("vdd", Signal);
        let out = cell.signal("out", Signal);
        let dut = cell.instantiate(Vdivider {
            r1: Resistor::new(20),
            r2: Resistor::new(20),
        });

        cell.connect(dut.io().pwr.vdd, vdd);
        cell.connect(dut.io().pwr.vss, io.vss);
        cell.connect(dut.io().out, out);

        let iprobe = cell.instantiate(Iprobe);
        cell.connect(iprobe.io().p, vdd_a);
        cell.connect(iprobe.io().n, vdd);

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd_a);
        cell.connect(vsource.io().n, io.vss);

        Ok(VdividerTbData { iprobe, dut })
    }
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;
    use rust_decimal_macros::dec;
    use spectre::{analysis::tran::Tran, ErrPreset};
    use substrate::context::Context;

    use super::*;
    use std::path::PathBuf;

    pub const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    pub const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");

    #[inline]
    pub fn get_path(test_name: &str, file_name: &str) -> PathBuf {
        PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
    }

    #[inline]
    pub fn test_data(file_name: &str) -> PathBuf {
        PathBuf::from(TEST_DATA_DIR).join(file_name)
    }
    #[test]
    fn vdivider_tran() {
        let test_name = "spectre_vdivider_tran";
        let sim_dir = get_path(test_name, "sim/");
        let ctx = Context::builder().install(Spectre::default()).build();
        let sim = ctx.get_sim_controller(VdividerTb, sim_dir).unwrap();
        let output = sim
            .simulate(
                Default::default(),
                Tran {
                    stop: dec!(1e-6),
                    errpreset: Some(ErrPreset::Conservative),
                    ..Default::default()
                },
            )
            .unwrap();

        for (actual, expected) in [
            // (&*output.current, 1.8 / 40.),
            (&*output.iprobe, 1.8 / 40.),
            // (&*output.vdd, 1.8),
            // (&*output.out, 0.9),
        ] {
            // assert!(actual
            //     .iter()
            //     .cloned()
            //     .all(|val| relative_eq!(val, expected)));
        }
    }
}
