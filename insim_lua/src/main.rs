use clap::Parser;
use futures::stream::FuturesUnordered;
use miette::{IntoDiagnostic, Result};
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

use axum::{extract::Extension, routing::get, Router};
use tower::ServiceBuilder;

#[tokio::main]
pub async fn main() -> Result<()> {
    miette::set_panic_hook();
    setup_tracing();

    let args = Args::parse();
    let config = config::read(&args.config)?;

    let mut tasks = HashMap::new();

    let fut = FuturesUnordered::new();

    for server in config.servers.iter() {
        // TODO lets be more specific about what we want to do here
        let (insim_future, lua_future, state) = task::spawn(server)?;

        fut.push(insim_future);
        fut.push(lua_future);

        tasks.insert(server.name.clone(), state);
    }

    let app = Router::new()
        .route("/", get(web::index))
        .route("/s/:server", get(web::server_index))
        // Use a precompiled and minified build of axum-live-view's JavaScript.
        // This is the easiest way to get started. Integration with bundlers
        // is of course also possible.
        .route("/bundle.js", axum_live_view::precompiled_js())
        .layer(ServiceBuilder::new().layer(Extension(tasks)));

    // ...that we run like any other axum app
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .into_diagnostic()?;

    // FIXME
    // while let Some(res) = fut.next().await {
    //     res.into_diagnostic()?;
    // }

    Ok(())
}
