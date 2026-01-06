use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    pub addr: SocketAddr,

    #[arg(short, long)]
    pub password: Option<String>,
}
