use std::path::PathBuf;

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "../tests/data");

#[inline]
pub(crate) fn get_path(test_name: &str, file_name: &str) -> PathBuf {
    PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
}

#[inline]
pub(crate) fn test_data(file_name: &str) -> PathBuf {
    PathBuf::from(TEST_DATA_DIR).join(file_name)
}
