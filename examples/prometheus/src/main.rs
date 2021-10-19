use axum;
use insim;
use tracing::info;
use tracing_subscriber;
use std::net::SocketAddr;
use lazy_static::lazy_static;
use prometheus;
use prometheus::Encoder;

lazy_static! {
    static ref PACKET_COUNTER: prometheus::IntCounter =
        prometheus::register_int_counter!("insim_packets", "Total number of Insim Packets received").unwrap();
}

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
struct SelectBeginnerBmw {}
impl insim::client::EventHandler for SelectBeginnerBmw {
    fn on_connect(&self, ctx: insim::client::Ctx) {
        ctx.send(
            insim::protocol::relay::HostSelect {
                hname: "^0[^7MR^0c] ^7Beginner ^0BMW".into(),
                ..Default::default()
            }
            .into(),
        );
    }
}

struct PromHandler {}
impl insim::client::EventHandler for PromHandler {
    #[allow(unused)]
    fn on_raw(&self, ctx: insim::client::Ctx, data: &insim::protocol::Packet) {
        PACKET_COUNTER.inc();
    }
}

async fn axum_prom_route() -> String {
    let mut buffer = Vec::new();
    let encoder = prometheus::TextEncoder::new();

    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}

#[tokio::main]
pub async fn main() {
    setup();

    let routes = axum::Router::new()
        .route("/metrics", axum::handler::get(axum_prom_route));

    let web = axum::Server::bind(
        &SocketAddr::from(([0, 0, 0, 0], 3000))
    );

    let client = insim::client::Config::default()
        .relay()
        .using_event_handler(PromHandler {})
        .using_event_handler(SelectBeginnerBmw{})
        .build();

    // run until insim client or web server completes
    tokio::select! {
        res = client.run() => {
            info!("insim client result {:?}", res);
        },

        res = web.serve(routes.into_make_service()) => {
            info!("result {:?}", res);
        }
    }
}
