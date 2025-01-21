use crate::utils::execute_run_script;
use crate::{error::Error, TEMPLATES};
use serde::Serialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tera::Context;

#[derive(Serialize)]
pub struct ExtractParams<'a> {
    pub cell_name: &'a str,
    pub work_dir: &'a Path,
    pub gds_path: &'a Path,
    pub tech_file_path: &'a Path,
    pub netlist_path: &'a Path,
}

#[derive(Serialize)]
struct ExtractRunsetContext<'a> {
    cell_name: &'a str,
    work_dir: &'a Path,
    gds_path: &'a Path,
    tech_file_path: &'a Path,
    netlist_path: &'a Path,
    tcl_path: &'a Path,
    run_script_path: &'a Path,
}

pub struct ExtractGeneratedPaths {
    run_script_path: PathBuf,
}

fn write_extract_files(params: &ExtractParams) -> Result<ExtractGeneratedPaths, Error> {
    fs::create_dir_all(params.work_dir).map_err(Error::Io)?;

    let tcl_path = params.work_dir.join("lvs_extract.tcl");
    let run_script_path = params.work_dir.join("run_magic.sh");

    let context = ExtractRunsetContext {
        cell_name: params.cell_name,
        work_dir: params.work_dir,
        gds_path: params.gds_path,
        tech_file_path: params.tech_file_path,
        netlist_path: params.netlist_path,
        tcl_path: &tcl_path,
        run_script_path: &run_script_path,
    };

    let context = Context::from_serialize(context).map_err(Error::Tera)?;

    let contents = TEMPLATES
        .render("lvs_extract.tcl", &context)
        .map_err(Error::Tera)?;
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

    Ok(ExtractGeneratedPaths { run_script_path })
}

fn run_extract_inner(
    work_dir: impl AsRef<Path>,
    run_script_path: impl AsRef<Path>,
) -> Result<(), Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    execute_run_script(run_script_path.as_ref(), &work_dir, "extract")?;
    Ok(())
}

pub fn run_extract(params: &ExtractParams) -> Result<(), Error> {
    let ExtractGeneratedPaths {
        run_script_path, ..
    } = write_extract_files(params)?;
    run_extract_inner(params.work_dir, run_script_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::extract::*;
    use crate::tests::{COLBUF_LAYOUT_PATH, SKY130_TECH_FILE, TEST_BUILD_PATH};
    use std::path::PathBuf;

    #[test]
    fn test_run_magic_extract() -> anyhow::Result<()> {
        let gds_path = PathBuf::from(COLBUF_LAYOUT_PATH);
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_run_magic_extract");
        let netlist_path = work_dir.join("test_col_inv_array.lvs.spice");
        let _ = std::fs::remove_file(&netlist_path);
        assert!(!netlist_path.exists());

        run_extract(&ExtractParams {
            work_dir: &work_dir,
            gds_path: &gds_path,
            cell_name: "test_col_inv_array",
            tech_file_path: &PathBuf::from(SKY130_TECH_FILE),
            netlist_path: &netlist_path,
        })?;

        assert!(netlist_path.exists());
        let meta = std::fs::metadata(netlist_path).unwrap();
        assert!(meta.is_file());
        assert!(meta.len() > 0);

        Ok(())
    }
}
