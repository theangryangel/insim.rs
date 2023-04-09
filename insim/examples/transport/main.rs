use insim::prelude::*;

fn setup() {
    // setup tracing with some defaults if nothing is set
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
pub async fn main() -> Result<(), insim::error::Error> {
    setup();

    tracing::info!("connecting!");

    let mut client = ClientBuilder::default()
        // .connect_udp("0.0.0.0:29991", "localhost:29999")
        .connect_relay("Nubbins AU Demo")
        .await?;

    tracing::info!("Connected!");

    let mut i = 0;

    while let Some(m) = client.next().await {
        i += 1;

        tracing::info!("Event: {:?} {:?}", m, i);
    }

    Ok(())
}
