use crate::utils::{aggregate_sources, execute_run_script};
use crate::{error::Error, TEMPLATES};
use pegasus::lvs::{run_lvs, LvsParams, LvsStatus};
use regex::Regex;
use serde::{Deserialize, Serialize};
use spice::netlist::NetlistOptions;
use spice::Spice;
use std::fmt::Display;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use substrate::schematic::conv::{ConvertedNodePath, RawLib};
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::{
    Cell, HasNestedView, InstancePath, NestedView, PrimitiveBinding, Schematic,
};
use substrate::scir::{NamedSliceOne, NetlistLibConversion, SliceOnePath};
use substrate::simulation::data::{Save, SaveKey, Saved};
use substrate::simulation::{Analysis, Simulator};
use substrate::types::schematic::{NestedNode, RawNestedNode};
use substrate::types::{Flatten, HasBundleKind, HasNameTree};
use substrate::{
    arcstr::{self, ArcStr},
    block::Block,
};
use tera::Context;

pub struct PexParams<'a> {
    pub work_dir: &'a Path,
    pub lvs_work_dir: &'a Path,
    pub lvs_run_name: &'a str,
    pub technology_dir: &'a Path,
    pub pex_netlist_path: &'a Path,
}

pub struct PexGeneratedPaths {
    pub run_file_path: PathBuf,
}

#[derive(Serialize)]
struct PexTemplateContext<'a> {
    lvs_work_dir: &'a Path,
    lvs_run_name: &'a str,
    technology_dir: &'a Path,
    pex_netlist_path: &'a Path,
}

pub fn write_pex_run_file(params: &PexParams) -> Result<PexGeneratedPaths, Error> {
    fs::create_dir_all(params.work_dir).map_err(Error::Io)?;

    let run_file_path = params.work_dir.join("quantus.ccl");

    let pex_context = PexTemplateContext {
        lvs_work_dir: params.lvs_work_dir,
        lvs_run_name: params.lvs_run_name,
        technology_dir: params.technology_dir,
        pex_netlist_path: params.pex_netlist_path,
    };
    let contents = TEMPLATES
        .render(
            "quantus.ccl",
            &Context::from_serialize(&pex_context).map_err(Error::Tera)?,
        )
        .map_err(Error::Tera)?;

    fs::write(&run_file_path, contents).map_err(Error::Io)?;

    Ok(PexGeneratedPaths { run_file_path })
}

pub fn write_pex_run_script(
    work_dir: impl AsRef<Path>,
    run_file_path: impl AsRef<Path>,
) -> Result<PathBuf, Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    let run_script_path = work_dir.as_ref().join("run_pex.sh");

    let mut context = Context::new();
    context.insert("run_file_path", run_file_path.as_ref());

    let contents = TEMPLATES
        .render("run_pex.sh", &context)
        .map_err(Error::Tera)?;

    fs::write(&run_script_path, contents).map_err(Error::Io)?;

    Ok(run_script_path)
}

pub fn run_quantus_pex(
    work_dir: impl AsRef<Path>,
    run_script_path: impl AsRef<Path>,
) -> Result<(), Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    execute_run_script(run_script_path.as_ref(), &work_dir, "pex")
}

pub fn run_pex(params: &PexParams) -> Result<(), Error> {
    let PexGeneratedPaths { run_file_path } = write_pex_run_file(params)?;
    let run_script_path = write_pex_run_script(params.work_dir, run_file_path)?;
    run_quantus_pex(params.work_dir, run_script_path)?;

    Ok(())
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Pex<T> {
    pub schematic: Arc<T>,
    pub gds_path: PathBuf,
    pub layout_cell_name: ArcStr,
    pub work_dir: PathBuf,
    pub lvs_rules_dir: PathBuf,
    pub lvs_rules_path: PathBuf,
    pub technology_dir: PathBuf,
}

impl<T> Clone for Pex<T> {
    fn clone(&self) -> Self {
        Self {
            schematic: self.schematic.clone(),
            gds_path: self.gds_path.clone(),
            layout_cell_name: self.layout_cell_name.clone(),
            work_dir: self.work_dir.clone(),
            lvs_rules_dir: self.lvs_rules_dir.clone(),
            lvs_rules_path: self.lvs_rules_path.clone(),
            technology_dir: self.technology_dir.clone(),
        }
    }
}

impl<T: Block> Block for Pex<T> {
    type Io = <T as Block>::Io;

    fn name(&self) -> ArcStr {
        self.schematic.name()
    }

    fn io(&self) -> Self::Io {
        self.schematic.io()
    }
}

pub struct PexContext {
    /// The source spice file for this DSPF extracted view.
    lib: Arc<RawLib<Spice>>,
    conv: Arc<NetlistLibConversion>,
    path: InstancePath,
}

impl HasNestedView<PexContext> for NestedNode {
    type NestedView = RawNestedNode;

    fn nested_view(&self, parent: &PexContext) -> NestedView<Self, PexContext> {
        let n = self;
        let path = parent.lib.convert_node_path(&n.path()).unwrap();
        let path = match path {
            ConvertedNodePath::Cell(path) => path,
            ConvertedNodePath::Primitive {
                instances, port, ..
            } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
        };
        let path = parent.lib.scir.simplify_path(path);
        RawNestedNode::new(
            parent.path.clone(),
            Spice::node_path_with_prefix_and_separator(
                &parent.lib.scir,
                &parent.conv,
                &path,
                "X",
                "/",
            )
            .to_uppercase(),
        )
    }
}

pub struct PexData<T: Schematic> {
    cell: Cell<Arc<T>>,
    lib: Arc<RawLib<Spice>>,
    conv: Arc<NetlistLibConversion>,
}

pub struct NestedPexData<T: Schematic> {
    cell: Cell<Arc<T>>,
    ctx: PexContext,
}

impl<T: Schematic> NestedPexData<T>
where
    T::NestedData: HasNestedView<PexContext>,
{
    pub fn data(&self) -> NestedView<T::NestedData, PexContext> {
        self.cell.custom_data(&self.ctx)
    }
}

impl<T: Schematic> HasNestedView for PexData<T> {
    type NestedView = NestedPexData<T>;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self, InstancePath> {
        NestedPexData {
            cell: self.cell.clone(),
            ctx: PexContext {
                lib: self.lib.clone(),
                conv: self.conv.clone(),
                path: parent.clone(),
            },
        }
    }
}

impl<T: Schematic> HasNestedView for NestedPexData<T> {
    type NestedView = NestedPexData<T>;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self, InstancePath> {
        NestedPexData {
            cell: self.cell.clone(),
            ctx: PexContext {
                lib: self.ctx.lib.clone(),
                conv: self.ctx.conv.clone(),
                path: self.ctx.path.prepend(parent),
            },
        }
    }
}

impl<S: Simulator, A: Analysis, T: Schematic> Save<S, A> for NestedPexData<T>
where
    T::NestedData: HasNestedView<PexContext>,
    NestedView<T::NestedData, PexContext>: Save<S, A>,
{
    type SaveKey = SaveKey<NestedView<T::NestedData, PexContext>, S, A>;
    type Saved = Saved<NestedView<T::NestedData, PexContext>, S, A>;

    fn save(
        &self,
        ctx: &substrate::simulation::SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
        self.data().save(ctx, opts)
    }

    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
        <NestedView<T::NestedData, PexContext> as Save<S, A>>::from_saved(output, key)
    }
}

impl<T: Schematic<Schema = Spice>> Schematic for Pex<T>
where
    T::NestedData: HasNestedView<PexContext>,
{
    type Schema = Spice;
    type NestedData = PexData<T>;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let source_path = self.work_dir.join("source.sp");
        let pex_netlist_path = self
            .work_dir
            .join(format!("{}.pex.spf", self.schematic.name()));
        let rawlib = cell.ctx().export_scir(self.schematic.clone()).unwrap();

        let conv = Spice.write_scir_netlist_to_file(
            &rawlib.scir,
            &source_path,
            NetlistOptions::default(),
        )?;

        let lvs_dir = self.work_dir.join("lvs");
        let pex_dir = self.work_dir.join("pex");

        assert!(
            matches!(
                run_lvs(&LvsParams {
                    work_dir: &lvs_dir,
                    layout_path: &self.gds_path,
                    layout_cell_name: "test_col_inv_array",
                    source_paths: &[source_path],
                    source_cell_name: "col_inv_array",
                    rules_dir: &self.lvs_rules_dir,
                    rules_path: &self.lvs_rules_path,
                })
                .expect("failed to run LVS")
                .status,
                LvsStatus::Correct
            ),
            "LVS failed"
        );

        run_pex(&PexParams {
            work_dir: &pex_dir,
            lvs_work_dir: &lvs_dir,
            lvs_run_name: "test_col_inv_array",
            technology_dir: &self.technology_dir,
            pex_netlist_path: &pex_netlist_path,
        })
        .expect("failed to run PEX");

        let cell_inner = cell
            .ctx()
            .generate_schematic(self.schematic.clone())
            .cell()
            .clone();

        let ports = self
            .io()
            .kind()
            .flat_names(None)
            .into_iter()
            .map(|n| arcstr::format!("{}", n))
            .collect::<Vec<ArcStr>>();

        let primitive = spice::Primitive::RawInstanceWithInclude {
            cell: self.schematic.name(),
            netlist: pex_netlist_path,
            ports: ports.clone(),
        };
        let mut binding = PrimitiveBinding::new(primitive);
        for (n, name) in io.flatten_vec().iter().zip(ports.iter()) {
            binding.connect(name, n);
        }
        cell.set_primitive(binding);
        Ok(PexData {
            cell: cell_inner,
            lib: Arc::new(rawlib),
            conv: Arc::new(conv),
        })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use pegasus::lvs::{run_lvs, LvsParams, LvsStatus};
    use rust_decimal_macros::dec;
    use spectre::analysis::tran::Tran;
    use spectre::blocks::Vsource;
    use spectre::{ErrPreset, Options, Spectre};
    use spice::{Primitive, Spice};
    use substrate::arcstr;
    use substrate::block::Block;
    use substrate::context::Context;
    use substrate::schematic::{NestedData, Schematic};
    use substrate::scir::{Cell, Direction, Instance, LibraryBuilder};
    use substrate::simulation::SimController;
    use substrate::types::schematic::{NestedNode, Node, NodeBundle};
    use substrate::types::{Array, InOut, Input, Io, Output, Signal, TestbenchIo};

    use crate::pex::{run_pex, write_pex_run_file, PexParams};
    use crate::tests::{
        EXAMPLES_PATH, SKY130_LVS, SKY130_LVS_RULES_PATH, SKY130_TECHNOLOGY_DIR, TEST_BUILD_PATH,
    };
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use super::{NestedPexData, Pex};

    #[test]
    fn test_write_pex_run_file() -> anyhow::Result<()> {
        let test_dir = PathBuf::from(TEST_BUILD_PATH).join("test_write_pex_run_file");
        let lvs_work_dir = test_dir.join("lvs");
        let pex_work_dir = test_dir.join("pex");
        let pex_path = pex_work_dir.join("test_col_inv_array.pex.netlist");

        write_pex_run_file(&PexParams {
            work_dir: &pex_work_dir,
            lvs_work_dir: &lvs_work_dir,
            lvs_run_name: "test_col_inv_array",
            technology_dir: &Path::new(SKY130_TECHNOLOGY_DIR),
            pex_netlist_path: &pex_path,
        })?;
        Ok(())
    }

    #[test]
    fn test_run_pex_col_inv() -> anyhow::Result<()> {
        let layout_path = PathBuf::from(EXAMPLES_PATH).join("gds/test_col_inv_array.gds");
        let source_path = PathBuf::from(EXAMPLES_PATH).join("spice/col_inv_array.spice");
        let test_dir = PathBuf::from(TEST_BUILD_PATH).join("test_run_pex_col_inv");
        let lvs_dir = test_dir.join("lvs");
        let pex_dir = test_dir.join("pex");
        let pex_path = pex_dir.join("test_col_inv_array.pex.netlist");

        assert!(
            matches!(
                run_lvs(&LvsParams {
                    work_dir: &lvs_dir,
                    layout_path: &layout_path,
                    layout_cell_name: "test_col_inv_array",
                    source_paths: &[source_path],
                    source_cell_name: "col_inv_array",
                    rules_dir: &PathBuf::from(SKY130_LVS),
                    rules_path: &PathBuf::from(SKY130_LVS_RULES_PATH),
                })?
                .status,
                LvsStatus::Correct
            ),
            "LVS failed"
        );

        run_pex(&PexParams {
            work_dir: &pex_dir,
            lvs_work_dir: &lvs_dir,
            lvs_run_name: "test_col_inv_array",
            technology_dir: &Path::new(SKY130_TECHNOLOGY_DIR),
            pex_netlist_path: &pex_path,
        })?;
        Ok(())
    }

    #[test]
    fn test_sim_pex() {
        #[derive(Clone, Debug, Default, Io)]
        struct ColDataInvIo {
            din: Input<Signal>,
            din_b: Output<Signal>,
            vdd: InOut<Signal>,
            vss: InOut<Signal>,
        }

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
        #[substrate(io = "ColDataInvIo")]
        struct ColDataInv;

        #[derive(NestedData)]
        struct ColDataInvData {
            din_b: Node,
        }

        impl Schematic for ColDataInv {
            type Schema = Spice;
            type NestedData = ColDataInvData;

            fn schematic(
                &self,
                io: &substrate::types::schematic::IoNodeBundle<Self>,
                cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
            ) -> substrate::error::Result<Self::NestedData> {
                let mut scir = Spice::scir_cell_from_str(
                    r#"
                .subckt col_data_inv din din_b vdd vss
                M0 dout din vss vss nfet_01v8 w=2.6u l=0.15u
                M1 dout din vdd vdd pfet_01v8 w=4u l=0.15u
                .ends
            "#,
                    "col_data_inv",
                );

                scir.connect("din", io.din);
                scir.connect("din_b", io.din_b);
                scir.connect("vss", io.vss);
                scir.connect("vdd", io.vdd);

                cell.set_scir(scir);
                Ok(ColDataInvData { din_b: io.din_b })
            }
        }

        #[derive(Clone, Debug, Io)]
        struct ColInvArrayIo {
            din: Input<Array<Signal>>,
            din_b: Output<Array<Signal>>,
            vdd: InOut<Signal>,
            vss: InOut<Signal>,
        }

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        struct ColInvArray;

        impl Block for ColInvArray {
            type Io = ColInvArrayIo;

            fn name(&self) -> arcstr::ArcStr {
                arcstr::literal!("col_inv_array")
            }

            fn io(&self) -> Self::Io {
                ColInvArrayIo {
                    din: Input(Array::new(32, Signal)),
                    din_b: Output(Array::new(32, Signal)),
                    vdd: InOut(Signal),
                    vss: InOut(Signal),
                }
            }
        }

        #[derive(NestedData)]
        struct ColInvArrayData {
            din_b_31: NestedNode,
        }

        impl Schematic for ColInvArray {
            type Schema = Spice;
            type NestedData = ColInvArrayData;

            fn schematic(
                &self,
                io: &substrate::types::schematic::IoNodeBundle<Self>,
                cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
            ) -> substrate::error::Result<Self::NestedData> {
                let invs = (0..32)
                    .map(|i| {
                        cell.instantiate_connected(
                            ColDataInv,
                            NodeBundle::<ColDataInvIo> {
                                din: io.din[i],
                                din_b: io.din_b[i],
                                vdd: io.vdd,
                                vss: io.vss,
                            },
                        )
                    })
                    .collect::<Vec<_>>();

                Ok(ColInvArrayData {
                    din_b_31: invs[31].din_b.clone(),
                })
            }
        }

        #[derive(Debug, Clone, Hash, Eq, PartialEq, Block)]
        #[substrate(io = "TestbenchIo")]
        struct PexTb;

        impl Schematic for PexTb {
            type Schema = Spectre;
            type NestedData = NestedPexData<ColInvArray>;
            fn schematic(
                &self,
                io: &substrate::types::schematic::IoNodeBundle<Self>,
                cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
            ) -> substrate::error::Result<Self::NestedData> {
                let layout_path = PathBuf::from(EXAMPLES_PATH).join("gds/test_col_inv_array.gds");
                let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_sim_pex/pex");
                let vdd = cell.signal("vdd", Signal);
                let mut spice_builder = cell.sub_builder::<Spice>();
                let dut = spice_builder.instantiate(Pex {
                    schematic: Arc::new(ColInvArray),
                    gds_path: layout_path,
                    layout_cell_name: "test_col_inv_array".into(),
                    work_dir: work_dir,
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
            let out = sim
                .simulate(
                    Options::default(),
                    Tran {
                        stop: dec!(2e-9),
                        errpreset: Some(ErrPreset::Conservative),
                        ..Default::default()
                    },
                )
                .expect("failed to run simulation");

            *out.din_b_31.first().unwrap()
        }

        let test_name = "spectre_can_include_sections";
        let sim_dir = PathBuf::from(TEST_BUILD_PATH)
            .join(test_name)
            .join("test_sim_pex/sim");
        let ctx = Context::builder().install(Spectre::default()).build();

        let output = run(ctx.get_sim_controller(PexTb, &sim_dir).unwrap());

        assert_relative_eq!(output, 1.8);
    }
}
