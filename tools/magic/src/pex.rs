use crate::utils::execute_run_script;
use crate::{TEMPLATES, error::Error};
use anyhow::anyhow;
use serde::Serialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tera::Context;

#[derive(Serialize)]
pub struct PexParams<'a> {
    pub cell_name: &'a str,
    pub work_dir: &'a Path,
    pub gds_path: &'a Path,
    pub tech_file_path: &'a Path,
    pub pex_netlist_path: &'a Path,
}

#[derive(Serialize)]
struct PexRunsetContext<'a> {
    cell_name: &'a str,
    work_dir: &'a Path,
    gds_path: &'a Path,
    tech_file_path: &'a Path,
    pex_netlist_path: &'a Path,
    tcl_path: &'a Path,
    run_script_path: &'a Path,
}

struct PexGeneratedPaths {
    run_script_path: PathBuf,
}

fn write_pex_files(params: &PexParams) -> Result<PexGeneratedPaths, Error> {
    fs::create_dir_all(params.work_dir).map_err(Error::Io)?;

    let tcl_path = params.work_dir.join("pex.tcl");
    let run_script_path = params.work_dir.join("run_magic.sh");

    let context = PexRunsetContext {
        cell_name: params.cell_name,
        work_dir: params.work_dir,
        gds_path: params.gds_path,
        tech_file_path: params.tech_file_path,
        pex_netlist_path: params.pex_netlist_path,
        tcl_path: &tcl_path,
        run_script_path: &run_script_path,
    };

    let context = Context::from_serialize(context).map_err(Error::Tera)?;

    let contents = TEMPLATES.render("pex.tcl", &context).map_err(Error::Tera)?;
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

    Ok(PexGeneratedPaths { run_script_path })
}

fn run_pex_inner(
    work_dir: impl AsRef<Path>,
    run_script_path: impl AsRef<Path>,
) -> Result<(), Error> {
    fs::create_dir_all(&work_dir).map_err(Error::Io)?;

    execute_run_script(run_script_path.as_ref(), &work_dir, "pex")?;
    Ok(())
}

pub fn run_pex(params: &PexParams) -> Result<(), Error> {
    let _ = std::fs::remove_file(params.pex_netlist_path);
    let PexGeneratedPaths {
        run_script_path, ..
    } = write_pex_files(params)?;
    run_pex_inner(params.work_dir, run_script_path)?;
    if !params.pex_netlist_path.exists() {
        return Err(anyhow!("magic failed to write output PEX netlist").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::pex::*;
    use crate::tests::{COLBUF_LAYOUT_PATH, SKY130_TECH_FILE, TEST_BUILD_PATH};
    use std::path::PathBuf;

    #[test]
    fn test_run_magic_pex() -> anyhow::Result<()> {
        let gds_path = PathBuf::from(COLBUF_LAYOUT_PATH);
        let work_dir = PathBuf::from(TEST_BUILD_PATH).join("test_run_magic_pex");
        let pex_netlist_path = work_dir.join("test_col_buffer_array.pex.spice");
        let _ = std::fs::remove_file(&pex_netlist_path);
        assert!(!pex_netlist_path.exists());

        run_pex(&PexParams {
            work_dir: &work_dir,
            gds_path: &gds_path,
            cell_name: "test_col_buffer_array",
            tech_file_path: &PathBuf::from(SKY130_TECH_FILE),
            pex_netlist_path: &pex_netlist_path,
        })?;

        assert!(pex_netlist_path.exists());
        let meta = std::fs::metadata(pex_netlist_path).unwrap();
        assert!(meta.is_file());
        assert!(meta.len() > 0);

        Ok(())
    }
}
