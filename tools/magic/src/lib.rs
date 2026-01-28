//! Magic plugin for Substrate.

use lazy_static::lazy_static;
use tera::Tera;

pub mod drc;
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
    use std::path::PathBuf;

    pub const TEST_BUILD_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    pub const COLBUF_LAYOUT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/latest/colbuf/test_col_buffer_array.gds"
    );
    pub const INVERTER_LICON8_LAYOUT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/gds/inverter_licon8.gds"
    );
    pub fn sky130_tech_file() -> PathBuf {
        PathBuf::from(
            std::env::var("OPEN_PDKS_ROOT")
                .expect("OPEN_PDKS_ROOT environment variable must be defined"),
        )
        .join("sky130/magic/sky130.tech")
    }
}
