use std::path::Path;

use crate::config::Config;
use anyhow::Result;

pub fn parse_document(toml: &str, _file: &Path, _config: &Config) -> Result<toml::Table> {
    // At the moment, no compatibility checks are needed.
    toml.parse()
        .map_err(|e| anyhow::Error::from(e).context("could not parse input as TOML"))
}
