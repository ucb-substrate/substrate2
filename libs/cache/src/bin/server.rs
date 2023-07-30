//! The cache server binary.
//!
//! Can be configured to expose a remote or local API, or both.
#![warn(missing_docs)]

use std::{net::SocketAddr, path::PathBuf};

use cache::{error::Result, persistent::server::Server};
use clap::Parser;

/// The arguments to the cache server binary.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The socket address that the local API gRPC server should listen on.
    #[clap(short, long, value_name = "ENDPOINT")]
    pub local: Option<SocketAddr>,
    /// The socket address that the remote API gRPC server should listen on.
    #[clap(short, long, value_name = "ENDPOINT")]
    pub remote: Option<SocketAddr>,
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

    builder = builder.root(args.root);

    if let Some(remote) = args.remote {
        builder = builder.remote(remote).await?;
    }

    if let Some(local) = args.local {
        builder = builder.local(local).await?;
    }

    let server = builder.build();

    server.start().await?;

    Ok(())
}
