use std::fs;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::error::Error;

pub fn execute_run_script(
    path: impl AsRef<Path>,
    work_dir: impl AsRef<Path>,
    output_prefix: &str,
) -> Result<(), Error> {
    let path = path.as_ref();
    let work_dir = work_dir.as_ref();

    let out_file =
        fs::File::create(work_dir.join(format!("{output_prefix}.out"))).map_err(Error::Io)?;
    let err_file =
        fs::File::create(work_dir.join(format!("{output_prefix}.err"))).map_err(Error::Io)?;

    // Make the run script executable
    let mut perms = std::fs::metadata(path).map_err(Error::Io)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(Error::Io)?;

    let status = Command::new("/usr/bin/bash")
        .arg(path)
        .current_dir(work_dir)
        .stdout(out_file)
        .stderr(err_file)
        .status()
        .map_err(Error::Io)?;

    if !status.success() {
        return Err(Error::Pegasus(status));
    }

    Ok(())
}
