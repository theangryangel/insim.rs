use crate::state::{State, connection::Connection};
use axum::{
    extract::{ws::{WebSocketUpgrade, Message}, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Response, sse},
    routing::get,
    Extension, Json, Router,
};

use futures::{sink::SinkExt, stream::StreamExt};

use crate::state::player::Player;
use std::{sync::Arc, task::Poll};
use std::{collections::HashMap, path::PathBuf};

use miette::{IntoDiagnostic, Result};

use minijinja::context;
use minijinja::{Environment, Source};
use minijinja_autoreload::AutoReloader;

pub(crate) fn spawn(tasks: HashMap<String, Arc<State>>) -> tokio::task::JoinHandle<Result<()>> {
    let mut env = Environment::new();
    env.set_source(Source::from_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")));

    tokio::task::spawn(async move {
        let addr: std::net::SocketAddr = ([0, 0, 0, 0], 3000).into();

        let app = Router::new()
            .route("/", get(servers_index))
            .route("/s/:server/live", get(servers_live))
            .route("/s/:server", get(servers_show))
            .layer(Extension(Arc::new(env)))
            .layer(Extension(tasks));

        axum::Server::bind(&addr.to_string().parse().unwrap())
            .serve(app.into_make_service())
            .await
            .into_diagnostic()
    })
}

async fn servers_index(
    env: Extension<Arc<Environment<'static>>>,
    state: Extension<HashMap<String, Arc<State>>>,
) -> impl IntoResponse {
    let servers = state.keys().map(|e| e.clone()).collect::<Vec<String>>();

    let tmpl = env.get_template("hello.html").unwrap();
    let res = tmpl
        .render(context! {
            name => servers
        })
        .unwrap(); // FIXME
    Html(res)
}

async fn servers_show(
    Path(server): Path<String>,
    env: Extension<Arc<Environment<'static>>>,
    state: Extension<HashMap<String, Arc<State>>>,
) -> impl IntoResponse {
    let servers = state.get(&server).unwrap();

    let tmpl = env.get_template("servers_show.html").unwrap();
    let res = tmpl
        .render(context! {
            players => (*servers.get_players()).clone(),
            connections => (*servers.get_connections()).clone(),
            name => &server,
        })
        .unwrap(); // FIXME
    Html(res)
}


async fn servers_live(
    Path(server): Path<String>,
    env: Extension<Arc<Environment<'static>>>,
    state: Extension<HashMap<String, Arc<State>>>
) -> sse::Sse<impl futures::stream::Stream<Item = Result<sse::Event, std::convert::Infallible>>> {
    println!("server = {:?}", server);

    let s = state.get(&server).unwrap().clone();


    let stream = async_stream::stream! {

            loop {

                s.notify_on_player().notified().await;
                let tmpl = env.get_template("servers_info.html").unwrap();

                let res = tmpl
                .render(context! {
                    players => (*s.get_players()).clone(),
                    connections => (*s.get_connections()).clone(),
                    name => "",
                }).unwrap();

                yield Ok(sse::Event::default().event("message").data(res))
            }
    };

    sse::Sse::new(stream)
}
