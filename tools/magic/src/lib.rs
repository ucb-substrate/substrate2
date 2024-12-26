//! Magic plugin for Substrate.

use lazy_static::lazy_static;
use tera::Tera;

pub mod error;
pub mod extract;
pub mod pex;
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
    pub const COLBUF_LAYOUT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "../../examples2/colbuf/test_col_buffer_array.gds"
    );
    pub const SKY130_TECH_FILE: &str = concat!(env!("OPEN_PDKS_ROOT"), "/sky130/magic/sky130.tech");
}
