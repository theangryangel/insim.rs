extern crate insim;
use tracing::{error, info};
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

    let mut client = insim::Config::default().relay().build().await;

    // This is going to get awful to work with.
    // Is it better to have some kind of "Sink" or "Handler" thats passed to client?
    while let Some(event) = client.recv().await {
        match event {
            Ok(insim::client::Event::Connected) => {
                info!("Connected");
                let hlr = insim::packets::Insim::RelayHostListRequest(
                    insim::packets::relay::HostListRequest { reqi: 0 },
                );

                client.send(hlr);

                let hs =
                    insim::packets::Insim::RelayHostSelect(insim::packets::relay::HostSelect {
                        reqi: 0,

                        hname: "^0[^7MR^0c] ^7Beginner ^0BMW".into(),
                        admin: "".into(),
                        spec: "".into(),
                    });

                client.send(hs);
            }
            Ok(data) => {
                info!("{:?}", data);
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }
}
