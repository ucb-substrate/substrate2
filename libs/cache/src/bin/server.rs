use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use cache::{error::Result, server::CacheServer};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short = 'H', long, default_value = "0.0.0.0")]
    host: IpAddr,
    #[clap(short, long, default_value_t = 28055)]
    port: u16,
    #[clap(value_parser)]
    root: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let server = CacheServer::new(args.root, SocketAddr::new(args.host, args.port));
    server.start().await?;

    Ok(())
}
