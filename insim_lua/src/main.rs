use clap::Parser;
use futures::stream::{FuturesUnordered, StreamExt};
use miette::Result;
use std::collections::HashMap;
use std::path;

mod config;
mod script;
mod state;
mod task;
mod web;

/// insim_lua does stuff
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    config: path::PathBuf,
}

fn setup_tracing() {
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

#[tokio::main]
pub async fn main() -> Result<()> {
    miette::set_panic_hook();
    setup_tracing();

    let args = Args::parse();
    let config = config::read(&args.config)?;

    let mut tasks = HashMap::new();

    let mut fut = FuturesUnordered::new();

    for server in config.servers.iter() {
        // TODO lets be more specific about what we want to do here
        let (insim_future, lua_future, state) = task::spawn(server)?;

        fut.push(insim_future);
        fut.push(lua_future);

        tasks.insert(server.name.clone(), state);
    }

    fut.push(web::spawn(tasks));

    // FIXME
    while let Some(res) = fut.next().await {
        panic!("{:?}", res);
        //res.into_diagnostic()?;
    }

    Ok(())
}
