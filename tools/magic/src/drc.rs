//! Run design rule checking (DRC) using Magic.

use crate::utils::{OutputFiles, execute_run_script};
use crate::{TEMPLATES, error::Error};
use anyhow::anyhow;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufRead;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tera::Context;

/// Parameters for running DRC using Magic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrcParams<'a> {
    /// The name of the cell to DRC check.
    pub cell_name: &'a str,
    /// The working directory.
    pub work_dir: &'a Path,
    /// The path to the GDS layout file, which must contain a cell named `cell_name`.
    pub gds_path: &'a Path,
    /// The path to the Magic tech file.
    ///
    /// Contains process-specific information, such as available layers and DRC rules.
    pub tech_file_path: &'a Path,
    /// The path to which the DRC report should be written.
    pub drc_report_path: &'a Path,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DrcRunsetContext<'a> {
    cell_name: &'a str,
    work_dir: &'a Path,
    gds_path: &'a Path,
    tech_file_path: &'a Path,
    drc_report_path: &'a Path,
    tcl_path: &'a Path,
    run_script_path: &'a Path,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrcGeneratedPaths {
    run_script_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrcData {
    pub rule_checks: Vec<RuleCheck>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleCheck {
    pub reason: String,
    pub num_results: u32,
}

fn write_drc_files(params: &DrcParams) -> Result<DrcGeneratedPaths, Error> {
    fs::create_dir_all(params.work_dir).map_err(Error::Io)?;

    let tcl_path = params.work_dir.join("drc.tcl");
    let run_script_path = params.work_dir.join("run_magic.sh");

    let context = DrcRunsetContext {
        cell_name: params.cell_name,
        work_dir: params.work_dir,
        gds_path: params.gds_path,
        tech_file_path: params.tech_file_path,
        drc_report_path: params.drc_report_path,
        tcl_path: &tcl_path,
        run_script_path: &run_script_path,
    };

    let context = Context::from_serialize(context).map_err(Error::Tera)?;

    let contents = TEMPLATES.render("drc.tcl", &context).map_err(Error::Tera)?;
    fs::write(&tcl_path, contents).map_err(Error::Io)?;

    let contents = TEMPLATES
        .render("run_magic.sh", &context)
        .map_err(Error::Tera)?;
    fs::write(&run_script_path, contents).map_err(Error::Io)?;
    let mut perms = fs::metadata(&run_script_path)
        .map_err(Error::Io)?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&run_script_path, perms).map_err(Error::Io)?;

    Ok(DrcGeneratedPaths { run_script_path })
}

pub fn parse_drc_results(report_path: impl AsRef<Path>) -> Result<DrcData, Error> {
    let file = fs::File::open(&report_path).map_err(Error::Io)?;
    let rule_checks: Result<Vec<RuleCheck>, Error> = std::io::BufReader::new(file)
        .lines()
        .tuples()
        .map(|(reason, count)| -> Result<RuleCheck, Error> {
            let reason = reason?;
            let count = count?;
            let count = count
                .trim()
                .parse()
                .map_err(|e| anyhow!("failed to parse error count `{count}`: {e:?}"))?;
            Ok(RuleCheck {
                reason,
                num_results: count,
            })
        })
        .filter(|rc| rc.as_ref().map(|rc| rc.num_results > 0).unwrap_or(true))
        .collect();
    Ok(DrcData {
        rule_checks: rule_checks?,
    })
}

fn run_drc_inner(
    work_dir: impl AsRef<Path>,
    run_script_path: impl AsRef<Path>,
) -> Result<OutputFiles, Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    execute_run_script(run_script_path.as_ref(), &work_dir, "drc")
}

/// Run DRC using Magic.
pub fn run_drc(params: &DrcParams) -> Result<DrcData, Error> {
    let DrcGeneratedPaths {
        run_script_path, ..
    } = write_drc_files(params)?;
    let output_files = run_drc_inner(params.work_dir, run_script_path)?;
    // Magic sometimes exits with exit code 0 even if one of the TCL commands had an error.
    // This checks that Magic reached and executed the final TCL command.
    if !std::fs::read_to_string(output_files.stdout)?.contains("__substrate_magic_drc_complete_0") {
        return Err(anyhow!("magic did not complete successfully").into());
    }
    parse_drc_results(params.drc_report_path)
}

#[cfg(test)]
mod tests {
    use crate::drc::*;
    use crate::tests::{
        COLBUF_LAYOUT_PATH, INVERTER_LICON8_LAYOUT_PATH, SKY130_TECH_FILE, TEST_BUILD_PATH,
    };
    use std::path::PathBuf;

    #[test]
    fn test_run_magic_drc_clean() -> anyhow::Result<()> {
        let gds_path = PathBuf::from(COLBUF_LAYOUT_PATH);
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_run_magic_drc_clean");
        let drc_report_path = work_dir.join("drc_results.rpt");
        let _ = std::fs::remove_file(&drc_report_path);
        assert!(!drc_report_path.exists());

        let checks = run_drc(&DrcParams {
            work_dir: &work_dir,
            gds_path: &gds_path,
            cell_name: "test_col_inv_array",
            tech_file_path: &PathBuf::from(SKY130_TECH_FILE),
            drc_report_path: &drc_report_path,
        })?;
        assert!(drc_report_path.exists());
        assert!(
            checks.rule_checks.is_empty(),
            "expected layout to be DRC clean"
        );

        Ok(())
    }

    #[test]
    fn test_run_magic_drc_licon8() -> anyhow::Result<()> {
        let gds_path = PathBuf::from(INVERTER_LICON8_LAYOUT_PATH);
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_run_magic_drc_licon8");
        let drc_report_path = work_dir.join("drc_results.rpt");
        let _ = std::fs::remove_file(&drc_report_path);
        assert!(!drc_report_path.exists());

        let checks = run_drc(&DrcParams {
            work_dir: &work_dir,
            gds_path: &gds_path,
            cell_name: "inverter",
            tech_file_path: &PathBuf::from(SKY130_TECH_FILE),
            drc_report_path: &drc_report_path,
        })?;
        assert!(drc_report_path.exists());
        assert_eq!(checks.rule_checks.len(), 1,);
        assert!(checks.rule_checks[0].reason.contains("licon.8"));
        assert_eq!(checks.rule_checks[0].num_results, 2);

        Ok(())
    }
}
