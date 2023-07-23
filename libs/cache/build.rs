/// Compiles protocol buffer code using [`tonic_build`].
fn main() {
    let protos = &["proto/local.proto", "proto/remote.proto"];
    let dirs = &["proto/"];
    tonic_build::configure()
        .compile(protos, dirs)
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));
    // recompile protobufs only if any of the proto files changes.
    for file in protos {
        println!("cargo:rerun-if-changed={}", file);
    }
}
