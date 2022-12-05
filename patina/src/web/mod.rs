use crate::state::{chat::Chat, State};
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
            chat => servers.chat().iter().map(|c| c.clone()).collect::<Vec<Chat>>(),
            player_count => servers.get_player_count(),
            connection_count => servers.get_connection_count(),
            game => servers.game(),
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

            let notify_on_player = s.notify_on_player();
            let notify_on_chat = s.notify_on_chat();

            tokio::select! {
                _ = notify_on_player.notified() => {
                    let tmpl = env.get_template("servers_info.html").unwrap();

                    let res = tmpl
                    .render(context! {
                        players => (s.get_players()),
                        connections => (s.get_connections()),
                        name => "",
                    }).unwrap();

                    yield Ok(sse::Event::default().event("players").data(res))
                },

                _ = notify_on_chat.notified() => {
                    let tmpl = env.get_template("servers_chat.html").unwrap();

                    let res = tmpl
                    .render(context! {
                        chat => s.chat().iter().map(|c| c.clone()).collect::<Vec<Chat>>(),
                    }).unwrap();

                    yield Ok(sse::Event::default().event("message").data(res))
                }

            }

        }
    };

    sse::Sse::new(stream)
}
