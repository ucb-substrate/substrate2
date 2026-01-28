//! Pegasus plugin for Substrate.

use lazy_static::lazy_static;
use tera::Tera;

pub mod error;
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

#[derive(Debug)]
pub struct RuleCheck {
    pub name: String,
    pub num_results: u32,
}

#[cfg(test)]
pub mod tests {
    pub const TEST_BUILD_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
    pub const EXAMPLES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../pegasus/examples");
    pub const COLBUF_LAYOUT_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/latest/colbuf/test_col_buffer_array.gds"
    );
}
