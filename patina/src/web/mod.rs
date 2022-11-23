use crate::state::State;
use axum::{
    extract::Path,
    response::{sse, Html, IntoResponse},
    routing::get,
    Extension, Router,
};

use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use miette::{IntoDiagnostic, Result};

use minijinja::context;
use minijinja::{Environment, Source};

fn get_minijinja_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_source(Source::from_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates"),
    ));
    env
}

pub(crate) fn spawn(tasks: HashMap<String, Arc<State>>) -> tokio::task::JoinHandle<Result<()>> {
    let env = get_minijinja_env();

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
    #[allow(clippy::map_clone)]
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
            players => servers.get_players(),
            connections => servers.get_connections(),
            name => &server,
        })
        .unwrap(); // FIXME
    Html(res)
}

async fn servers_live(
    Path(server): Path<String>,
    env: Extension<Arc<Environment<'static>>>,
    state: Extension<HashMap<String, Arc<State>>>,
) -> sse::Sse<impl futures::stream::Stream<Item = Result<sse::Event, std::convert::Infallible>>> {
    println!("server = {:?}", server);

    let s = state.get(&server).unwrap().clone();

    let stream = async_stream::stream! {

            loop {

                s.notify_on_player().notified().await;
                let tmpl = env.get_template("servers_info.html").unwrap();

                let res = tmpl
                .render(context! {
                    players => (s.get_players()),
                    connections => (s.get_connections()),
                    name => "",
                }).unwrap();

                yield Ok(sse::Event::default().event("message").data(res))
            }
    };

    sse::Sse::new(stream)
}
