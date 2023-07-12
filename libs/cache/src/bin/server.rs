use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use cache::{error::Result, persistent::server::CacheServer};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short = 'H', long, default_value = "0.0.0.0")]
    host: IpAddr,
    #[clap(short, long)]
    remote: Option<u16>,
    #[clap(short, long)]
    local: Option<u16>,
    #[clap(value_parser)]
    root: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut server = CacheServer::new(args.root);

    if let Some(remote) = args.remote {
        server = server.with_remote(SocketAddr::new(args.host, remote));
    }

    if let Some(local) = args.local {
        server = server.with_remote(SocketAddr::new(args.host, local));
    }

    server.start().await?;

    Ok(())
}
