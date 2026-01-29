use ngspice::Ngspice;
use rust_decimal_macros::dec;
use spectre::Spectre;
use spectre::blocks::Vsource;
use spice::Spice;
use substrate::arcstr;
use substrate::block::Block;
use substrate::schematic::{NestedData, Schematic};
use substrate::types::schematic::{NestedNode, Node, NodeBundle};
use substrate::types::{Array, InOut, Input, Io, Output, Signal, TestbenchIo};

#[derive(Clone, Debug, Default, Io)]
pub struct ColInvIo {
    pub din: Input<Signal>,
    pub din_b: Output<Signal>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Sky130Schema {
    Open,
    SrcNda,
    Cds,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
#[substrate(io = "ColInvIo")]
pub struct ColInv(Sky130Schema);

impl Schematic for ColInv {
    type Schema = Spice;
    type NestedData = ();

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        // TODO: Use intended schematic flow.
        let spice = match self.0 {
            Sky130Schema::Open => {
                r#"
                .subckt col_data_inv din din_b vdd vss
                X0 din_b din vss vss sky130_fd_pr__nfet_01v8 w=1.4 l=0.15
                X1 din_b din vdd vdd sky130_fd_pr__pfet_01v8 w=2.6 l=0.15
                .ends
            "#
            }
            Sky130Schema::SrcNda => {
                r#"
                .subckt col_data_inv din din_b vdd vss
                M0 din_b din vss vss nshort w=1.4u l=0.15u
                M1 din_b din vdd vdd pshort w=2.6u l=0.15u
                .ends
            "#
            }
            Sky130Schema::Cds => {
                r#"
                .subckt col_data_inv din din_b vdd vss
                M0 din_b din vss vss nfet_01v8 w=1.4u l=0.15u
                M1 din_b din vdd vdd pfet_01v8 w=2.6u l=0.15u
                .ends
            "#
            }
        };
        let mut scir = Spice::scir_cell_from_str(spice, "col_data_inv");

        scir.connect("din", io.din);
        scir.connect("din_b", io.din_b);
        scir.connect("vss", io.vss);
        scir.connect("vdd", io.vdd);

        cell.set_scir(scir);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Io)]
pub struct ColBufIo {
    pub din: Input<Signal>,
    pub dout: Output<Signal>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
#[substrate(io = "ColBufIo")]
pub struct ColBuf(Sky130Schema);

#[derive(NestedData)]
pub struct ColBufData {
    pub x: Node,
}

impl Schematic for ColBuf {
    type Schema = Spice;
    type NestedData = ColBufData;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let x = cell.signal("x", Signal);

        let _inv1 = cell.instantiate_connected(
            ColInv(self.0),
            NodeBundle::<ColInvIo> {
                din: io.din,
                din_b: x,
                vdd: io.vdd,
                vss: io.vss,
            },
        );

        let _inv2 = cell.instantiate_connected(
            ColInv(self.0),
            NodeBundle::<ColInvIo> {
                din: x,
                din_b: io.dout,
                vdd: io.vdd,
                vss: io.vss,
            },
        );

        Ok(ColBufData { x })
    }
}

#[derive(Clone, Debug, Io)]
pub struct ColBufArrayIo {
    pub din: Input<Array<Signal>>,
    pub dout: Output<Array<Signal>>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ColBufArray(pub Sky130Schema);

impl Block for ColBufArray {
    type Io = ColBufArrayIo;

    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("col_buf_array")
    }

    fn io(&self) -> Self::Io {
        ColBufArrayIo {
            din: Input(Array::new(32, Signal)),
            dout: Output(Array::new(32, Signal)),
            vdd: InOut(Signal),
            vss: InOut(Signal),
        }
    }
}

#[derive(NestedData)]
pub struct ColBufArrayData {
    pub x_31: NestedNode,
}

impl Schematic for ColBufArray {
    type Schema = Spice;
    type NestedData = ColBufArrayData;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let bufs = (0..32)
            .map(|i| {
                cell.instantiate_connected(
                    ColBuf(self.0),
                    NodeBundle::<ColBufIo> {
                        din: io.din[i],
                        dout: io.dout[i],
                        vdd: io.vdd,
                        vss: io.vss,
                    },
                )
            })
            .collect::<Vec<_>>();

        Ok(ColBufArrayData {
            x_31: bufs[31].x.clone(),
        })
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Block)]
#[substrate(io = "TestbenchIo")]
pub struct CdsPexTb {
    pub dut: quantus::pex::Pex<ColBufArray>,
}

impl Schematic for CdsPexTb {
    type Schema = Spectre;
    type NestedData = substrate::schematic::pex::NestedPexData<ColBufArray>;
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        assert_eq!(self.dut.schematic.0, Sky130Schema::Cds);
        let vdd = cell.signal("vdd", Signal);
        let mut spice_builder = cell.sub_builder::<Spice>();
        let dut = spice_builder.instantiate(self.dut.clone());

        cell.connect(dut.io().vdd, vdd);
        cell.connect(dut.io().vss, io.vss);
        for i in 0..31 {
            cell.connect(dut.io().din[i], vdd);
        }
        cell.connect(dut.io().din[31], io.vss);

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);

        Ok(dut.data())
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Block)]
#[substrate(io = "TestbenchIo")]
pub struct OpenPexTb {
    pub dut: magic_netgen::Pex<ColBufArray>,
}

impl Schematic for OpenPexTb {
    type Schema = Ngspice;
    type NestedData = magic_netgen::NestedPexData<ColBufArray>;
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        assert_eq!(self.dut.schematic.0, Sky130Schema::Open);
        let vdd = cell.signal("vdd", Signal);
        let mut spice_builder = cell.sub_builder::<Spice>();
        let dut = spice_builder.instantiate(self.dut.clone());

        cell.connect(dut.io().vdd, vdd);
        cell.connect(dut.io().vss, io.vss);
        for i in 0..31 {
            cell.connect(dut.io().din[i], vdd);
        }
        cell.connect(dut.io().din[31], io.vss);

        let vsource = cell.instantiate(ngspice::blocks::Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);

        Ok(dut.data())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use approx::assert_relative_eq;
    use sky130::{
        sky130_cds_tt_model_path, sky130_lvs, sky130_lvs_rules_path, sky130_magic_tech_file,
        sky130_netgen_setup_file, sky130_ngspice_model_path, sky130_technology_dir,
    };
    use spectre::{ErrPreset, Options, analysis::tran::Tran};
    use substrate::{
        context::Context,
        simulation::{SimController, waveform::TimeWaveform},
    };

    use super::*;
    pub const TEST_BUILD_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    pub const COLBUF_LAYOUT_PATH: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/test_col_buffer_array.gds");

    #[test]
    fn test_sim_cadence_pex() {
        fn run(sim: SimController<Spectre, CdsPexTb>) -> f64 {
            let mut opts = Options::default();
            opts.include(sky130_cds_tt_model_path());
            let out = sim
                .simulate(
                    opts,
                    Tran {
                        stop: dec!(2e-9),
                        errpreset: Some(ErrPreset::Conservative),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");

            out.x_31.first_x().unwrap()
        }

        let test_name = "test_sim_cadence_pex";
        let sim_dir = PathBuf::from(TEST_BUILD_PATH).join(test_name).join("sim");
        let ctx = Context::builder().install(Spectre::default()).build();

        let layout_path = PathBuf::from(COLBUF_LAYOUT_PATH);
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join(test_name).join("pex");

        let output = run(ctx
            .get_sim_controller(
                CdsPexTb {
                    dut: quantus::pex::Pex {
                        schematic: Arc::new(ColBufArray(Sky130Schema::Cds)),
                        gds_path: layout_path,
                        layout_cell_name: "test_col_buffer_array".into(),
                        work_dir,
                        lvs_rules_path: sky130_lvs_rules_path(),
                        lvs_rules_dir: sky130_lvs(),
                        technology_dir: sky130_technology_dir(),
                    },
                },
                &sim_dir,
            )
            .unwrap());

        assert_relative_eq!(output, 1.8, max_relative = 1e-2);
    }

    #[test]
    fn test_sim_open_pex() {
        fn run(sim: SimController<Ngspice, OpenPexTb>) -> f64 {
            let mut opts = ngspice::Options::default();
            opts.include_section(sky130_ngspice_model_path(), "tt");
            let out = sim
                .simulate(
                    opts,
                    ngspice::tran::Tran {
                        stop: dec!(2e-9),
                        step: dec!(2e-11),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");

            out.x_31.first_x().unwrap()
        }

        let test_name = "test_sim_open_pex";
        let sim_dir = PathBuf::from(TEST_BUILD_PATH).join(test_name).join("sim");
        let ctx = Context::builder().install(Ngspice::default()).build();

        let layout_path = PathBuf::from(COLBUF_LAYOUT_PATH);
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join(test_name).join("pex");

        let output = run(ctx
            .get_sim_controller(
                OpenPexTb {
                    dut: magic_netgen::Pex {
                        schematic: Arc::new(ColBufArray(Sky130Schema::Open)),
                        gds_path: layout_path,
                        layout_cell_name: "test_col_buffer_array".into(),
                        work_dir,
                        magic_tech_file_path: sky130_magic_tech_file(),
                        netgen_setup_file_path: sky130_netgen_setup_file(),
                    },
                },
                &sim_dir,
            )
            .unwrap());

        assert_relative_eq!(output, 1.8, max_relative = 1e-2);
    }
}
