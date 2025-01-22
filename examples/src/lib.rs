pub fn get_path(rel_path: &str) -> String {
    format!("{}/{rel_path}", env!("CARGO_MANIFEST_DIR"))
}
