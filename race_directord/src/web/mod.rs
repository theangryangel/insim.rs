use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::net::SocketAddr;

use crate::connections::ConnectionManager;

async fn root(State(manager): State<ConnectionManager>) -> impl IntoResponse {
    Json(manager.list().await)
}

async fn subscribe(
    State(manager): State<ConnectionManager>,
    Path(peer): Path<String>,
) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    // XXX: Proof of concept. Needs tidying.
    let mut receiver = manager.subscribe(&peer).await.unwrap();

    Sse::new(async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok(insim::connection::Event::Data(i, _)) => {
                    let event = Event::default()
                        .json_data(i).unwrap();

                    yield Ok(event);
                },

                e => {
                    tracing::error!(error = ?e, "Failed to get");
                }
            }
        }
    })
    .keep_alive(KeepAlive::default())
}

pub(crate) fn run(addr: &SocketAddr, manager: ConnectionManager) {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/:peer/subscribe", get(subscribe))
        .with_state(manager);

    tracing::info!("Web listening on {}", addr);

    tokio::task::spawn(axum::Server::bind(addr).serve(app.into_make_service()));
}
