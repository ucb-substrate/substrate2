use approx::assert_relative_eq;
use rust_decimal_macros::dec;
use spice::Spice;
use substrate::arcstr;
use substrate::block::Block;
use substrate::context::Context;
use substrate::schematic::{NestedData, Schematic};
use substrate::scir::{Cell, Direction, Instance, LibraryBuilder};
use substrate::simulation::SimController;
use substrate::types::schematic::{NestedNode, Node, NodeBundle};
use substrate::types::{Array, InOut, Input, Io, Output, Signal, TestbenchIo};

#[derive(Clone, Debug, Default, Io)]
struct ColInvIo {
    din: Input<Signal>,
    din_b: Output<Signal>,
    vdd: InOut<Signal>,
    vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
#[substrate(io = "ColInvIo")]
struct ColInv;

impl Schematic for ColInv {
    type Schema = Spice;
    type NestedData = ();

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut scir = Spice::scir_cell_from_str(
            r#"
                .subckt col_data_inv din din_b vdd vss
                M0 din_b din vss vss nfet_01v8 w=1.4u l=0.15u
                M1 din_b din vdd vdd pfet_01v8 w=2.6u l=0.15u
                .ends
            "#,
            "col_data_inv",
        );

        scir.connect("din", io.din);
        scir.connect("din_b", io.din_b);
        scir.connect("vss", io.vss);
        scir.connect("vdd", io.vdd);

        cell.set_scir(scir);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Io)]
struct ColBufIo {
    din: Input<Signal>,
    dout: Output<Signal>,
    vdd: InOut<Signal>,
    vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
#[substrate(io = "ColBufIo")]
struct ColBuf;

#[derive(NestedData)]
struct ColBufData {
    x: Node,
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

        let inv1 = cell.instantiate_connected(
            ColInv,
            NodeBundle::<ColInvIo> {
                din: io.din,
                din_b: x,
                vdd: io.vdd,
                vss: io.vss,
            },
        );

        let inv2 = cell.instantiate_connected(
            ColInv,
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
struct ColBufArrayIo {
    din: Input<Array<Signal>>,
    dout: Output<Array<Signal>>,
    vdd: InOut<Signal>,
    vss: InOut<Signal>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct ColBufArray;

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
struct ColBufArrayData {
    x_31: NestedNode,
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
                    ColBuf,
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
struct PexTb;

impl Schematic for PexTb {
    type Schema = Spectre;
    type NestedData = NestedPexData<ColBufArray>;
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let layout_path = PathBuf::from(EXAMPLES_PATH).join("gds/test_col_buffer_array.gds");
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_sim_pex/pex");
        let vdd = cell.signal("vdd", Signal);
        let mut spice_builder = cell.sub_builder::<Spice>();
        let dut = spice_builder.instantiate(Pex {
            schematic: Arc::new(ColBufArray),
            gds_path: layout_path,
            layout_cell_name: "test_col_buffer_array".into(),
            work_dir,
            lvs_rules_path: PathBuf::from(SKY130_LVS_RULES_PATH),
            lvs_rules_dir: PathBuf::from(SKY130_LVS),
            technology_dir: PathBuf::from(SKY130_TECHNOLOGY_DIR),
        });

        cell.connect(dut.io().vdd, vdd);
        cell.connect(dut.io().vss, io.vss);
        cell.connect(dut.io().din[31], io.vss);

        let vsource = cell.instantiate(Vsource::dc(dec!(1.8)));
        cell.connect(vsource.io().p, vdd);
        cell.connect(vsource.io().n, io.vss);

        Ok(dut.data())
    }
}

fn run(sim: SimController<Spectre, PexTb>) -> f64 {
    let mut opts = Options::default();
    opts.include(PathBuf::from(SKY130_TT_MODEL_PATH));
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

    *out.x_31.first().unwrap()
}
