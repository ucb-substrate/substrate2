use snippets::build_snippets;

fn main() {
    let example_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/latest/substrate_api_examples/src/lib.rs"
    );
    println!("cargo::rerun-if-changed={example_path}");
    build_snippets(example_path, "substrate");
}
