use axum::{routing::get, Extension, Router};
use clap::Parser;
use miette::{IntoDiagnostic, Result};
use std::{path, sync::Arc};

mod config;
mod insim;
mod state;
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

    // FIXME implement config reloading

    let mut manager = crate::insim::InsimManager::new();
    manager.update_from_config(&config);

    let templates = web::templating::Engine::new(config.web.templates_to_path_buf(), false);

    let app = Router::new()
        .route("/", get(web::servers_index))
        .route("/s/:server/live", get(web::servers_live))
        .route("/s/:server/map", get(web::track_map))
        .route("/s/:server", get(web::servers_show))
        .layer(Extension(templates))
        .layer(Extension(Arc::new(manager)));

    axum::Server::bind(&config.web.listen)
        .serve(app.into_make_service())
        .await
        .into_diagnostic()?;

    Ok(())
}
