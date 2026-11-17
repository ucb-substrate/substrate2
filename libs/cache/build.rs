//! Generates the gRPC services for the cache servers.

use tonic_build::manual::{Builder, Method, Service};

/// Builds a unary RPC whose messages are encoded with `crate::rpc::PostcardCodec`.
fn method(name: &str, route_name: &str, input_type: &str, output_type: &str) -> Method {
    Method::builder()
        .name(name)
        .route_name(route_name)
        .input_type(input_type)
        .output_type(output_type)
        .codec_path("crate::rpc::PostcardCodec")
        .build()
}

fn main() {
    let local = Service::builder()
        .name("LocalCache")
        .package("local")
        .method(method("get", "Get", "super::GetRequest", "super::GetReply"))
        .method(method(
            "drop",
            "Drop",
            "super::DropRequest",
            "crate::rpc::Empty",
        ))
        .method(method(
            "heartbeat",
            "Heartbeat",
            "super::HeartbeatRequest",
            "crate::rpc::Empty",
        ))
        .method(method(
            "done",
            "Done",
            "super::DoneRequest",
            "crate::rpc::Empty",
        ))
        .build();
    let remote = Service::builder()
        .name("RemoteCache")
        .package("remote")
        .method(method("get", "Get", "super::GetRequest", "super::GetReply"))
        .method(method(
            "heartbeat",
            "Heartbeat",
            "super::HeartbeatRequest",
            "crate::rpc::Empty",
        ))
        .method(method(
            "set",
            "Set",
            "super::SetRequest",
            "crate::rpc::Empty",
        ))
        .build();
    Builder::new().compile(&[local, remote]);
    println!("cargo:rerun-if-changed=build.rs");
}
