use clap::Parser;
use std::path;

mod config;
mod manager;
mod script_path;

/// insim_lua does stuff
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    config: path::PathBuf,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();

    let config = config::read(&args.config);

    let mut instances = manager::Manager::new(config);
    instances.run().await;
}
