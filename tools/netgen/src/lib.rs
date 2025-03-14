//! Netgen plugin for Substrate.

use lazy_static::lazy_static;
use tera::Tera;

pub mod compare;
pub mod error;
pub mod utils;

pub const TEMPLATES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates");

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        match Tera::new(&format!("{TEMPLATES_PATH}/*")) {
            Ok(t) => t,
            Err(e) => {
                panic!("Encountered errors while parsing Tera templates: {e}");
            }
        }
    };
}

#[cfg(test)]
mod tests {
    pub const TEST_BUILD_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    pub const EXAMPLES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
    pub const SKY130_SETUP_FILE: &str =
        concat!(env!("OPEN_PDKS_ROOT"), "/sky130/netgen/sky130_setup.tcl");
}
