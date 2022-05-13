use clap::Parser;
use std::path;
use std::sync::RwLock;
//
// mod config;
// mod manager;
// mod script_path;

use axum::extract::Extension;
use axum::{async_trait, response::IntoResponse, routing::get, Router};
use axum_live_view::{
    event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower::ServiceBuilder;

/// insim_lua does stuff
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    config: path::PathBuf,
}

// async fn root(
//     Extension(state): Extension<Arc<RwLock<HashMap<String, manager::State>>>>,
// ) -> String {
//
//     let state = state.read().unwrap();
//
//     state.values()
//         .map(|c|
//             c.connections.lock().unwrap().iter().map(|c| c.uname.clone()).collect::<Vec<String>>()
//         )
//         .flatten()
//         .collect::<Vec<String>>().join("\n")
// }

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

use futures::SinkExt;
use futures::StreamExt;
use tokio::net::TcpStream;

#[tokio::main]
pub async fn main() {
    setup();

    let tcp: TcpStream = TcpStream::connect("isrelay.lfs.net:47474").await.unwrap();

    let mut t =
        insim::protocol::transport::Transport::new(tcp, insim::protocol::codec::Mode::Uncompressed);
    let isi = insim::protocol::insim::Init {
        name: "insim.rs".into(),
        password: "".into(),
        prefix: b'!',
        version: insim::protocol::VERSION,
        interval: 1000,
        flags: insim::protocol::insim::InitFlags::MCI,
        reqi: 1,
    };

    t.send(isi.into()).await;

    t.send(
        insim::protocol::relay::HostSelect {
            hname: "Nubbins AU Demo".into(),
            ..Default::default()
        }
        .into(),
    )
    .await;

    insim::client::Server::new(t, insim::client::Echo).await;

    // let args = Args::parse();
    //
    // let config = config::read(&args.config);
    //
    // let state : Arc<RwLock<HashMap<String, manager::State>>> = Arc::new(RwLock::new(HashMap::new()));
    //
    // let mut handles = futures::stream::FuturesUnordered::new();
    //
    // {
    //     let mut inner = state.write().unwrap();
    //
    //     for server in config.servers.iter() {
    //         let (state, fut) = manager::spawn(&server).unwrap();
    //         inner.insert(
    //             server.name.clone(),
    //             state,
    //         );
    //         handles.push(fut);
    //     }
    //
    // }
    //
    // let app = Router::new()
    //     .route("/", get(root))
    //     .route("/bundle.js", axum_live_view::precompiled_js())
    //     .layer(
    //         ServiceBuilder::new().layer(Extension(state))
    //     );
    //
    // let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    // let server = axum::Server::bind(&addr)
    //     .serve(app.into_make_service());
    //
    // tokio::select! {
    //     e = server => {
    //         println!("Server stopped: {:?}", e);
    //     },
    //     e = handles.next() => {
    //         println!("Instances stopped: {:?}", e);
    //     },
    // }
}
