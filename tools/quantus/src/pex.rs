use crate::utils::{aggregate_sources, execute_run_script};
use crate::{error::Error, TEMPLATES};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
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
            lvs_run_name: "col_inv_array",
            technology_dir: &Path::new(SKY130_TECHNOLOGY_DIR),
            pex_netlist_path: &pex_path,
        })?;
        Ok(())
    }
}
