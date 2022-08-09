use crate::state::State;
use axum::{
    extract::ws::WebSocketUpgrade,
    response::{Html, Response},
    routing::get,
    Extension, Router,
};
use dioxus_core::{Element, LazyNodes, Scope};
use dioxus_liveview::Liveview;

use std::collections::HashMap;
use std::sync::Arc;

mod components;
mod hooks;

use miette::{IntoDiagnostic, Result};
use tower::ServiceBuilder;

use dioxus::{
    prelude::*,
    router::{Link, Route, Router},
};

pub(crate) fn spawn(tasks: HashMap<String, Arc<State>>) -> tokio::task::JoinHandle<Result<()>> {
    tokio::task::spawn(async move {
        let addr: std::net::SocketAddr = ([0, 0, 0, 0], 3000).into();

        let view = dioxus_liveview::new(addr);
        let body = view
            .body("<title>insim_lua</title><script src=\"https://cdn.tailwindcss.com\"></script>");

        let app = Router::new()
            .route("/", get(move || async { Html(body) }))
            .route("/app", get(upgrade_ws))
            .layer(ServiceBuilder::new().layer(Extension(tasks)))
            .layer(ServiceBuilder::new().layer(Extension(view)));

        // ...that we run like any other axum app
        axum::Server::bind(&addr.to_string().parse().unwrap())
            .serve(app.into_make_service())
            .await
            .into_diagnostic()
    })
}

async fn upgrade_ws(
    ws: WebSocketUpgrade,
    Extension(tasks): Extension<HashMap<String, Arc<State>>>,
    Extension(view): Extension<Liveview>,
) -> Response {
    ws.on_upgrade(move |ws| async move { view.upgrade_with_props(ws, app, tasks).await })
}

fn app(cx: Scope<HashMap<String, Arc<State>>>) -> Element {
    cx.use_hook(|_| cx.provide_context(cx.props.clone()));

    cx.render(rsx! {
            Router {
                Route {
                    to: "/", {
                        cx.props.iter().map(|(key, _)| {
                            rsx! {
                                div {
                                    key: "{key}",
                                    Link {
                                        to: "/s/{key}",
                                        "{key}"
                                    }
                                }
                            }
                        })
                    }
                },
                Route {
                    to: "/s/:id",
                    components::server::show {}
                },
            }
    })
}
