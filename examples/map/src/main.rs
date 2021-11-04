use axum::{
    error_handling::HandleErrorExt,
    extract::Extension,
    http::StatusCode,
    response::sse::{Event, Sse},
    routing::{get, service_method_routing as service},
    AddExtensionLayer, Router,
};
use flume;
use futures::stream::Stream;
use futures::StreamExt;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{services::ServeDir, trace::TraceLayer};

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

// TODO this is probably common enough that we should turn it into a library feature.
// it should include error handling.
struct SelectBeginnerBmw {
    tx: flume::Sender<insim::protocol::Packet>,
}
impl insim::client::EventHandler for SelectBeginnerBmw {
    fn on_connect(&self, ctx: insim::client::Ctx) {
        ctx.send(
            insim::protocol::relay::HostSelect {
                hname: "Nubbins AU Demo".into(),
                ..Default::default()
            }
            .into(),
        );
    }

    fn on_raw(&self, ctx: insim::client::Ctx, data: &insim::protocol::Packet) {
        let res = self.tx.send(data.clone());
        if let Err(e) = res {
            tracing::error!("{:?}", e);
            ctx.shutdown();
        };
    }
}

struct AppState {
    rx: flume::Receiver<insim::protocol::Packet>,
}

#[tokio::main]
async fn main() {
    setup();

    let static_files_service = service::get(
        ServeDir::new("assets").append_index_html_on_directories(true),
    )
    .handle_error(|error: std::io::Error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", error),
        )
    });

    let (tx, rx) = flume::unbounded::<insim::protocol::Packet>();
    let app_state = Arc::new(AppState { rx });

    // build our application with a route
    let app = Router::new()
        .fallback(static_files_service)
        .route("/sse", get(sse_handler))
        .layer(AddExtensionLayer::new(app_state))
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    let web = axum::Server::bind(&addr);

    let client = insim::client::Config::default()
        .relay()
        .using_event_handler(SelectBeginnerBmw { tx })
        .build();

    // run until insim client or web server completes
    tokio::select! {
        res = client.run() => {
            panic!("insim client result {:?}", res);
        },

        res = web.serve(app.into_make_service()) => {
            panic!("result {:?}", res);
        }
    }
}

async fn sse_handler(
    Extension(state): Extension<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, serde_json::error::Error>>> {
    let rx = state.rx.clone();
    Sse::new(
        rx.into_stream()
            .map(|packet| Event::default().json_data(packet)),
    )
}
