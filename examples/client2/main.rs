extern crate insim;
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

    let mut i = 0;

    let config = insim::client::Config::default()
        .relay()
        .try_reconnect(true)
        .try_reconnect_attempts(2000);

    let client = insim::client2::Client2::from_config(config);

    while let Some(m) = client.recv().await {
        i += 1;

        match m {
            insim::client::Event::Connected => {
                let _ = client.send(insim::client::Event::Packet(
                    insim::protocol::relay::HostSelect {
                        hname: "Nubbins AU Demo".into(),
                        ..Default::default()
                    }
                    .into(),
                ));
            }
            _ => {}
        }

        tracing::debug!("Event: {:?} {:?}", m, i);

        // if i >= 10 {
        //     client.shutdown();
        // }
    }
}
