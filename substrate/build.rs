use snippets::build_snippets;

fn main() {
    println!("cargo::rerun-if-changed=examples/substrate.rs");
    build_snippets(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/substrate.rs"),
        "substrate",
    );
}
