extern crate insim;
use futures::{SinkExt, StreamExt};
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

#[tokio::main]
pub async fn main() {
    setup();

    let mut client = insim::framework::Config::default()
        .relay()
        .try_reconnect(false)
        .build();

    while let Some(m) = client.next().await {
        match m {
            insim::framework::Event::Connected => {
                let _ = client
                    .send(insim::framework::Event::Packet(
                        insim::protocol::relay::HostSelect {
                            hname: "Nubbins AU Demo".into(),
                            ..Default::default()
                        }
                        .into(),
                    ))
                    .await;
            }
            _ => {}
        }

        tracing::debug!("Event: {:?}", m);
    }
}
