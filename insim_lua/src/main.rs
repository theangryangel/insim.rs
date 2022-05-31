use clap::Parser;
use std::path;

/// insim_lua does stuff
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    config: path::PathBuf,
}

use tracing_subscriber;

fn setup() {
    // setup tracing with some defaults if nothing is set
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

use insim::client::prelude::*;

#[tokio::main]
pub async fn main() {
    setup();

    let mut client = Config::default()
        .relay(Some("Nubbins AU Demo".into()))
        .try_reconnect(true)
        .try_reconnect_attempts(2)
        .into_client();

    while let Some(d) = client.next().await {
        println!("{:?}", d);
    }
}
