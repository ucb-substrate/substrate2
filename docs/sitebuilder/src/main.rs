use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use config::SiteConfig;
use paths::{BUILD_DIR, CONFIG_PATH, SUBSTRATE_CLONE_URL};

pub mod config;
pub mod paths;

fn main() -> Result<()> {
    create_dir_all(BUILD_DIR)?;
    println!("Hello, world!");
    Ok(())
}

fn read_config() -> Result<SiteConfig> {
    Ok(toml::from_str(&std::fs::read_to_string(CONFIG_PATH)?)?)
}

fn get_latest_release() -> Result<String> {
    Ok(String::from_utf8(
        Command::new("git")
            .args(["describe", "--abbrev=0", "--tags"])
            .output()?
            .stdout,
    )?)
}

fn clone_branch(url: &str, branch: &str, dst: impl AsRef<Path>) -> Result<()> {
    let dst = dst.as_ref();
    let _status = Command::new("git")
        .args(["clone", "--depth=1", url])
        .arg(dst)
        .status()?;
    let _status = Command::new("git")
        .current_dir(dst)
        .args(["fetch", "--all"])
        .status()?;
    let _status = Command::new("git")
        .current_dir(dst)
        .args(["reset", "--hard", "HEAD"])
        .status()
        .expect("failed to execute process");
    let _status = Command::new("git")
        .current_dir(dst)
        .args(["checkout", branch])
        .status()
        .expect("failed to execute process");
    let _status = Command::new("git")
        .current_dir(dst)
        .args(["reset", "--hard", &format!("origin/{}", branch)])
        .status()
        .expect("failed to execute process");

    Ok(())
}

fn reconfigure_branch(branch: &str) -> Result<()> {}

fn build_all(cfg: &SiteConfig) -> Result<()> {
    for (dir_name, branch) in cfg
        .versions
        .iter()
        .map(|v| (v, v))
        .chain(std::iter::once(("__release", &get_latest_release()?)))
    {
        clone_branch(
            SUBSTRATE_CLONE_URL,
            branch,
            PathBuf::from(BUILD_DIR).join(dir_name),
        )?;
    }
    Ok(())
}
