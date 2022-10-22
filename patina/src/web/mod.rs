use crate::state::State;
use axum::{
    extract::{ws::WebSocketUpgrade, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};

use crate::state::player::Player;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use miette::{IntoDiagnostic, Result};

use minijinja::context;
use minijinja::{Environment, Source};
use minijinja_autoreload::AutoReloader;

pub(crate) fn spawn(tasks: HashMap<String, Arc<State>>) -> tokio::task::JoinHandle<Result<()>> {
    let reloader = Arc::new(AutoReloader::new(|notifier| {
        let mut env = Environment::new();
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
        notifier.watch_path(&template_path, true);
        env.set_source(Source::from_path(&template_path));
        Ok(env)
    }));

    tokio::task::spawn(async move {
        let addr: std::net::SocketAddr = ([0, 0, 0, 0], 3000).into();

        let app = Router::new()
            .route("/", get(servers_index))
            .layer(Extension(reloader))
            .layer(Extension(tasks));

        axum::Server::bind(&addr.to_string().parse().unwrap())
            .serve(app.into_make_service())
            .await
            .into_diagnostic()
    })
}

async fn servers_index(
    reloader: Extension<Arc<AutoReloader>>,
    state: Extension<HashMap<String, Arc<State>>>,
) -> impl IntoResponse {
    let servers = state.keys().map(|e| e.clone()).collect::<Vec<String>>();

    let env = reloader.acquire_env().unwrap();
    let tmpl = env.get_template("hello.html").unwrap();
    let res = tmpl
        .render(context! {
            name => servers
        })
        .unwrap(); // FIXME
    Html(res)
}

// async fn upgrade_ws(
//     ws: WebSocketUpgrade,
//     Extension(tasks): Extension<HashMap<String, Arc<State>>>,
// ) -> Response {
//     ws.on_upgrade(move |ws| async move { view.upgrade_with_props(ws, app, tasks).await })
// }
