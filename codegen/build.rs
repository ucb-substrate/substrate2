use snippets::build_snippets;

fn main() {
    println!("cargo::rerun-if-changed=examples/example.rs");
    build_snippets(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../substrate/examples/substrate.rs"
        ),
        "substrate",
    );
}
