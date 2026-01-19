use std::net::SocketAddr;

use clap::Parser;
use insim::core::track::Track;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    pub addr: SocketAddr,

    #[arg(short, long)]
    pub password: Option<String>,

    #[arg(short, long)]
    pub rounds: Option<usize>,

    #[arg(short, long)]
    pub max_scorers: Option<usize>,

    #[arg(short, long)]
    pub track: Option<Track>,

    #[arg(short, long)]
    pub layout: Option<String>,
}
