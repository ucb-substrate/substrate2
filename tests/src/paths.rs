use std::path::PathBuf;

pub const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

pub fn get_path(test_name: &str, file_name: &str) -> PathBuf {
    PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
}
