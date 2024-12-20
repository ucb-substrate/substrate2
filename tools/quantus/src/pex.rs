use crate::utils::execute_run_script;
use crate::{error::Error, TEMPLATES};
use pegasus::lvs::{run_lvs, LvsParams, LvsStatus};
use serde::Serialize;
use spice::netlist::NetlistOptions;
use spice::Spice;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::pex::PexData;
use substrate::schematic::{HasNestedView, PrimitiveBinding, Schematic};
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

pub type PexContext = substrate::schematic::pex::PexContext<Spice>;

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
                    layout_cell_name: &self.layout_cell_name,
                    source_paths: &[source_path],
                    source_cell_name: &self.schematic.name(),
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
            lvs_run_name: &self.layout_cell_name,
            technology_dir: &self.technology_dir,
            pex_netlist_path: &pex_netlist_path,
        })
        .expect("failed to run PEX");

        let cell_inner = cell
            .ctx()
            .generate_schematic(self.schematic.clone())
            .cell()
            .clone();

        // Read the PEX netlist to determine order of ports in the PEX netlist
        // (which may be different than the order of ports in the schematic netlist).
        let spice =
            spice::parser::Parser::parse_file(spice::parser::Dialect::Spice, &pex_netlist_path)
                .expect("failed to parse pex netlist");
        let subckt = spice
            .ast
            .elems
            .iter()
            .filter_map(|e| {
                if let spice::parser::Elem::Subckt(s) = e {
                    Some(s)
                } else {
                    None
                }
            })
            .find(|s| s.name.as_str() == self.layout_cell_name.as_str())
            .expect("did not find layout cell in pex netlist");

        let primitive = spice::Primitive::RawInstanceWithInclude {
            cell: self.schematic.name(),
            netlist: pex_netlist_path,
            ports: subckt
                .ports
                .iter()
                .map(|p| ArcStr::from(p.as_str()))
                .collect(),
        };

        let ports = self
            .io()
            .kind()
            .flat_names(None)
            .into_iter()
            .map(|n| arcstr::format!("{}", n))
            .collect::<Vec<ArcStr>>();
        let mut binding = PrimitiveBinding::new(primitive);
        for (n, name) in io.flatten_vec().iter().zip(ports.iter()) {
            binding.connect(name, n);
        }
        cell.set_primitive(binding);
        Ok(PexData::new(cell_inner, Arc::new(rawlib), Arc::new(conv)))
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
            technology_dir: Path::new(SKY130_TECHNOLOGY_DIR),
            pex_netlist_path: &pex_path,
        })?;
        Ok(())
    }

    #[test]
    fn test_run_pex_col_inv() -> anyhow::Result<()> {
        let layout_path = PathBuf::from(EXAMPLES_PATH).join("gds/test_col_buffer_array.gds");
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
