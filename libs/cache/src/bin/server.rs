//! The cache server binary.
//!
//! Can be configured to expose a remote or local API, or both.
#![warn(missing_docs)]

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use cache::{error::Result, persistent::server::Server};
use clap::Parser;

/// The arguments to the cache server binary.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The host IP address that the configured gRPC servers should listen on.
    #[clap(short = 'H', long, default_value = "0.0.0.0", value_hint = clap::ValueHint::Hostname)]
    pub host: IpAddr,
    /// The port that the local API gRPC server should listen on.
    #[clap(short, long, value_name = "PORT")]
    pub local: Option<u16>,
    /// The port that the remote API gRPC server should listen on.
    #[clap(short, long, value_name = "PORT")]
    pub remote: Option<u16>,
    /// The root directory of the cache server.
    ///
    /// All cached data and metadata will be stored in this directory.
    #[clap(value_parser, value_hint = clap::ValueHint::DirPath)]
    pub root: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut builder = Server::builder();

    builder.root(args.root);

    if let Some(remote) = args.remote {
        builder.remote(SocketAddr::new(args.host, remote));
    }

    if let Some(local) = args.local {
        builder.local(SocketAddr::new(args.host, local));
    }

    let server = builder.build();

    server.start().await?;

    Ok(())
}
