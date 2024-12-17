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
use substrate::schematic::conv::RawLib;
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::{
    Cell, HasNestedView, InstancePath, NestedView, PrimitiveBinding, Schematic,
};
use substrate::types::schematic::NestedNode;
use substrate::types::{Flatten, HasNameTree};
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Pex<T> {
    pub schematic: Arc<T>,
    pub gds_path: PathBuf,
    pub layout_cell_name: ArcStr,
    pub work_dir: PathBuf,
    pub lvs_rules_dir: PathBuf,
    pub lvs_rules_path: PathBuf,
    pub technology_dir: PathBuf,
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
    path: Option<InstancePath>,
}

impl PexContext {
    pub fn new(lib: RawLib<Spice>) -> Self {
        Self {
            lib: Arc::new(lib),
            path: None,
        }
    }
}

impl HasNestedView<PexContext> for NestedNode {
    type NestedView = NestedNode;
}

pub struct PexData<T: Schematic> {
    cell: Cell<T>,
    ctx: PexContext,
}

impl<T: Schematic> PexData<T>
where
    NestedView<T::NestedData>: HasNestedView<PexContext>,
{
    pub fn data(&self) -> NestedView<NestedView<T::NestedData>, PexContext> {
        self.cell.data().nested_view(&self.ctx)
    }
}

impl<T: Schematic> HasNestedView for PexData<T> {
    type NestedView = Self;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self, InstancePath> {
        Self {
            cell: self.cell.clone(),
            ctx: PexContext {
                lib: self.ctx.lib.clone(),
                path: Some(if let Some(path) = self.ctx.path {
                    path.prepend(parent)
                } else {
                    parent.clone()
                }),
            },
        }
    }
}

impl<T: Schematic<Schema = Spice>> Schematic for Pex<T>
where
    NestedView<T::NestedData>: HasNestedView<PexContext>,
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

        Spice.write_scir_netlist_to_file(&rawlib.scir, &source_path, NetlistOptions::default())?;

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
            ctx: PexContext::new(rawlib),
        })
    }
}

#[cfg(test)]
mod tests {
    use pegasus::lvs::{run_lvs, LvsParams, LvsStatus};

    use crate::pex::{run_pex, write_pex_run_file, PexParams};
    use crate::tests::{
        EXAMPLES_PATH, SKY130_LVS, SKY130_LVS_RULES_PATH, SKY130_TECHNOLOGY_DIR, TEST_BUILD_PATH,
    };
    use std::path::{Path, PathBuf};

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
}
