extern crate insim;
use tracing_subscriber;

use futures::{StreamExt, SinkExt};

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

    let mut client = insim::client::Config::default()
        .relay(None)
        .try_reconnect(true)
        .try_reconnect_attempts(2000)
        .into_client();

    while let Some(m) = client.next().await {
        i += 1;

        match m {
            insim::client::Event::Connected => {
                let _ = client
                    .send(
                        insim::protocol::Packet::RelayHostSelect(
                            insim::protocol::relay::HostSelect {
                                hname: "Nubbins AU Demo".into(),
                                ..Default::default()
                            }
                        ).into()
                    )
                    .await;
            }

            insim::client::Event::Data(insim::protocol::Packet::MultiCarInfo(mci)) => {
                tracing::debug!("MultiCarInfo: {:?}", mci);

                for car in mci.info.iter() {
                    tracing::info!("{} = {:?}", car.plid, car);
                }
            }

            _ => {
                tracing::info!("Event: {:?} {:?}", m, i);
            }
        }

        // if i >= 10 {
        //     client.shutdown();
        // }
    }
}
