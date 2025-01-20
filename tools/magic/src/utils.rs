use std::fs;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::Error;

pub(crate) struct OutputFiles {
    pub(crate) stdout: PathBuf,
    #[allow(dead_code)]
    pub(crate) stderr: PathBuf,
}

pub(crate) fn execute_run_script(
    path: impl AsRef<Path>,
    work_dir: impl AsRef<Path>,
    output_prefix: &str,
) -> Result<OutputFiles, Error> {
    let path = path.as_ref();
    let work_dir = work_dir.as_ref();

    let stdout_path = work_dir.join(format!("{output_prefix}.out"));
    let stderr_path = work_dir.join(format!("{output_prefix}.err"));

    let out_file = fs::File::create(&stdout_path)?;
    let err_file = fs::File::create(&stderr_path)?;

    // Make the run script executable
    let mut perms = std::fs::metadata(path).map_err(Error::Io)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(Error::Io)?;

    let status = Command::new("/usr/bin/bash")
        .arg(path)
        .current_dir(work_dir)
        .stdin(Stdio::null())
        .stdout(out_file)
        .stderr(err_file)
        .status()
        .map_err(Error::Io)?;

    if !status.success() {
        return Err(Error::Magic(status));
    }

    Ok(OutputFiles {
        stdout: stdout_path,
        stderr: stderr_path,
    })
}
