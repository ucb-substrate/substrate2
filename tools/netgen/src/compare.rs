use crate::utils::execute_run_script;
use crate::{error::Error, TEMPLATES};
use arcstr::ArcStr;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tera::Context;

const NO_MATCHING_NODE: &str = "<__no_matching_node__>";

#[derive(Serialize)]
pub struct CompareParams<'a> {
    pub netlist1_path: &'a Path,
    pub cell1: &'a str,
    pub netlist2_path: &'a Path,
    pub cell2: &'a str,
    pub work_dir: &'a Path,
    pub setup_file_path: &'a Path,
}

#[derive(Serialize)]
struct CompareRunsetContext<'a> {
    netlist1_path: &'a Path,
    cell1: &'a str,
    netlist2_path: &'a Path,
    cell2: &'a str,
    work_dir: &'a Path,
    setup_file_path: &'a Path,
    compare_results_path: &'a Path,
    run_script_path: &'a Path,
    nxf_path: &'a Path,
    ixf_path: &'a Path,
    tcl_path: &'a Path,
    no_matching_node: &'a str,
}

struct CompareGeneratedPaths {
    run_script_path: PathBuf,
    compare_results_path: PathBuf,
    nxf_path: PathBuf,
    ixf_path: PathBuf,
}

pub struct CompareOutput {
    /// Indicates whether or not the two input netlists matched.
    pub matches: bool,
    /// Mapping from netlist 1 node name to netlist 2 node name.
    pub node_map: HashMap<ArcStr, ArcStr>,
}

fn write_compare_files(params: &CompareParams) -> Result<CompareGeneratedPaths, Error> {
    fs::create_dir_all(params.work_dir).map_err(Error::Io)?;

    let compare_results_path = params.work_dir.join("compare_results.rpt");
    let run_script_path = params.work_dir.join("run_netgen.sh");
    let nxf_path = params.work_dir.join("mappings.nxf");
    let ixf_path = params.work_dir.join("mappings.ixf");
    let tcl_path = params.work_dir.join("compare.tcl");

    let context = CompareRunsetContext {
        netlist1_path: params.netlist1_path,
        cell1: params.cell1,
        netlist2_path: params.netlist2_path,
        cell2: params.cell2,
        work_dir: params.work_dir,
        setup_file_path: params.setup_file_path,
        compare_results_path: &compare_results_path,
        run_script_path: &run_script_path,
        nxf_path: &nxf_path,
        ixf_path: &ixf_path,
        tcl_path: &tcl_path,
        no_matching_node: NO_MATCHING_NODE,
    };

    let context = Context::from_serialize(context).map_err(Error::Tera)?;

    let contents = TEMPLATES
        .render("compare.tcl", &context)
        .map_err(Error::Tera)?;
    fs::write(&tcl_path, contents).map_err(Error::Io)?;

    let contents = TEMPLATES
        .render("run_netgen.sh", &context)
        .map_err(Error::Tera)?;
    fs::write(&run_script_path, contents).map_err(Error::Io)?;
    let mut perms = fs::metadata(&run_script_path)
        .map_err(Error::Io)?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&run_script_path, perms).map_err(Error::Io)?;

    Ok(CompareGeneratedPaths {
        run_script_path,
        compare_results_path,
        nxf_path,
        ixf_path,
    })
}

fn run_compare_inner(
    work_dir: impl AsRef<Path>,
    run_script_path: impl AsRef<Path>,
) -> Result<(), Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    execute_run_script(run_script_path.as_ref(), &work_dir, "compare")
}

pub fn compare(params: &CompareParams) -> Result<CompareOutput, Error> {
    let CompareGeneratedPaths {
        run_script_path,
        compare_results_path,
        nxf_path,
        ..
    } = write_compare_files(params)?;
    run_compare_inner(params.work_dir, run_script_path)?;

    let f = File::open(compare_results_path)?;
    let reader = BufReader::new(f);
    let mut matches = false;
    for line in reader.lines() {
        let line = line?;
        if line.contains("Final result: Circuits match uniquely.") {
            matches = true;
            break;
        }
    }

    let nxf = File::open(nxf_path)?;
    let reader = BufReader::new(nxf);
    let mut node_map = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let mut iter = line.split_whitespace();
        let n1 = iter.next().unwrap().trim();
        let n2 = iter.next().unwrap().trim();
        node_map.insert(n1.into(), n2.into());
    }

    Ok(CompareOutput { matches, node_map })
}

#[cfg(test)]
mod tests {
    use crate::compare::*;
    use crate::tests::{EXAMPLES_PATH, SKY130_SETUP_FILE, TEST_BUILD_PATH};
    use std::path::PathBuf;

    #[test]
    fn test_compare_with_node_matching() -> anyhow::Result<()> {
        let netlist1_path = PathBuf::from(EXAMPLES_PATH).join("col_inv_array.spice");
        let netlist2_path = PathBuf::from(EXAMPLES_PATH).join("col_inv_array.layout.spice");
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_compare_with_node_matching");
        let setup_file_path = PathBuf::from(SKY130_SETUP_FILE);
        let nodes = [
            "din_1", "din_b_2", "din_24", "din_21", "din_b_21", "vdd", "vss",
        ];

        let params = CompareParams {
            netlist1_path: &netlist1_path,
            cell1: "col_inv_array",
            netlist2_path: &netlist2_path,
            cell2: "col_inv_array",
            work_dir: &work_dir,
            setup_file_path: &setup_file_path,
        };
        let output = compare(&params)?;
        assert!(output.matches);
        for node in nodes {
            assert_eq!(output.node_map[node], node);
        }

        Ok(())
    }

    #[test]
    fn test_compare_mismatched() -> anyhow::Result<()> {
        let netlist1_path = PathBuf::from(EXAMPLES_PATH).join("col_inv_array.spice");
        let netlist2_path = PathBuf::from(EXAMPLES_PATH).join("col_inv_array.bad.spice");
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_compare_mismatched");
        let setup_file_path = PathBuf::from(SKY130_SETUP_FILE);

        let params = CompareParams {
            netlist1_path: &netlist1_path,
            cell1: "col_inv_array",
            netlist2_path: &netlist2_path,
            cell2: "col_inv_array",
            work_dir: &work_dir,
            setup_file_path: &setup_file_path,
        };
        let output = compare(&params)?;
        assert!(!output.matches);

        Ok(())
    }
}
