use std::path::{Path, PathBuf};

pub const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");

#[inline]
pub fn get_path(test_name: impl AsRef<Path>, file_name: impl AsRef<Path>) -> PathBuf {
    let mut buf = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    buf.push("build");
    buf.push(test_name);
    buf.push(file_name);
    buf
}

#[inline]
pub fn test_data(file_name: impl AsRef<Path>) -> PathBuf {
    let mut buf = PathBuf::from(TEST_DATA_DIR);
    buf.push(file_name);
    buf
}
