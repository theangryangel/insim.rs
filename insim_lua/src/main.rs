use clap::Parser;
use miette::Result;
use std::path;

mod config;
mod script;
mod task;

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

    let mut handles = Vec::new();

    for server in config.servers.iter() {
        // TODO lets be more specific about what we want to do here
        let (task_insim, task_lua) = task::spawn(server)?;

        handles.push(task_insim);
        handles.push(task_lua);
    }

    futures::future::join_all(handles).await;
    Ok(())
}
