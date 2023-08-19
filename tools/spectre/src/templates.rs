use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use serde::Serialize;
use tera::{Context, Tera};

pub(crate) const TEMPLATES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates");

lazy_static! {
    pub(crate) static ref TEMPLATES: Tera = {
        match Tera::new(&format!("{TEMPLATES_PATH}/*")) {
            Ok(t) => t,
            Err(e) => {
                panic!("Encountered errors while parsing Tera templates: {e}");
            }
        }
    };
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct RunScriptContext<'a> {
    pub(crate) netlist: &'a PathBuf,
    pub(crate) raw_output_path: &'a PathBuf,
    pub(crate) log_path: &'a PathBuf,
    pub(crate) bashrc: Option<&'a PathBuf>,
    pub(crate) format: &'a str,
    pub(crate) flags: &'a str,
}

pub(crate) fn write_run_script(
    ctx: RunScriptContext,
    path: impl AsRef<Path>,
) -> crate::error::Result<()> {
    let ctx = Context::from_serialize(ctx)?;
    let mut f = std::fs::File::create(path.as_ref())?;
    TEMPLATES.render_to("simulate.sh", &ctx, &mut f)?;

    Ok(())
}
