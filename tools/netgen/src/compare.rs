use crate::utils::execute_run_script;
use crate::{error::Error, TEMPLATES};
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
    pub node1_mappings: &'a [&'a str],
}

#[derive(Serialize)]
struct CompareRunsetContext<'a> {
    netlist1_path: &'a Path,
    cell1: &'a str,
    netlist2_path: &'a Path,
    cell2: &'a str,
    work_dir: &'a Path,
    setup_file_path: &'a Path,
    node1_mappings: &'a [&'a str],
    compare_results_path: &'a Path,
    run_script_path: &'a Path,
    nxf_path: &'a Path,
    tcl_path: &'a Path,
    no_matching_node: &'a str,
}

struct CompareGeneratedPaths {
    run_script_path: PathBuf,
    compare_results_path: PathBuf,
    nxf_path: PathBuf,
}

pub struct CompareOutput<'a> {
    /// Indicates whether or not the two input netlists matched.
    pub matches: bool,
    /// Mapping from netlist 1 node name to netlist 2 node name.
    pub node1_mappings: HashMap<&'a str, String>,
}

fn write_compare_files(params: &CompareParams) -> Result<CompareGeneratedPaths, Error> {
    fs::create_dir_all(params.work_dir).map_err(Error::Io)?;

    let compare_results_path = params.work_dir.join("compare_results.rpt");
    let run_script_path = params.work_dir.join("run_netgen.sh");
    let nxf_path = params.work_dir.join("mappings.nxf");
    let tcl_path = params.work_dir.join("compare.tcl");

    let context = CompareRunsetContext {
        netlist1_path: params.netlist1_path,
        cell1: params.cell1,
        netlist2_path: params.netlist2_path,
        cell2: params.cell2,
        work_dir: params.work_dir,
        setup_file_path: params.setup_file_path,
        node1_mappings: params.node1_mappings,
        compare_results_path: &compare_results_path,
        run_script_path: &run_script_path,
        nxf_path: &nxf_path,
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
    })
}

fn run_compare_inner(
    work_dir: impl AsRef<Path>,
    run_script_path: impl AsRef<Path>,
) -> Result<(), Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    execute_run_script(run_script_path.as_ref(), &work_dir, "compare")
}

pub fn compare<'a>(params: &'a CompareParams) -> Result<CompareOutput<'a>, Error> {
    let CompareGeneratedPaths {
        run_script_path,
        compare_results_path,
        nxf_path,
    } = write_compare_files(params)?;
    run_compare_inner(params.work_dir, run_script_path)?;

    let nxf = File::open(compare_results_path)?;
    let reader = BufReader::new(nxf);
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
    let mut node1_mappings = HashMap::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let node = line.trim();
        if node == NO_MATCHING_NODE {
            continue;
        } else {
            node1_mappings.insert(params.node1_mappings[i], node.to_string());
        }
    }

    Ok(CompareOutput {
        matches,
        node1_mappings,
    })
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
        let nodes = ["din_1", "din_b_2", "vdd", "vss"];

        let params = CompareParams {
            netlist1_path: &netlist1_path,
            cell1: "col_inv_array",
            netlist2_path: &netlist2_path,
            cell2: "col_inv_array",
            work_dir: &work_dir,
            node1_mappings: &nodes,
            setup_file_path: &setup_file_path,
        };
        let output = compare(&params)?;
        assert!(output.matches);
        for node in nodes {
            assert_eq!(output.node1_mappings[node], node);
        }

        Ok(())
    }

    #[test]
    fn test_compare_mismatched() -> anyhow::Result<()> {
        let netlist1_path = PathBuf::from(EXAMPLES_PATH).join("col_inv_array.spice");
        let netlist2_path = PathBuf::from(EXAMPLES_PATH).join("col_inv_array.bad.spice");
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_compare_mismatched");
        let setup_file_path = PathBuf::from(SKY130_SETUP_FILE);
        let nodes = [];

        let params = CompareParams {
            netlist1_path: &netlist1_path,
            cell1: "col_inv_array",
            netlist2_path: &netlist2_path,
            cell2: "col_inv_array",
            work_dir: &work_dir,
            node1_mappings: &nodes,
            setup_file_path: &setup_file_path,
        };
        let output = compare(&params)?;
        assert!(!output.matches);

        Ok(())
    }
}
