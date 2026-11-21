//! A simple demo of the outgauge library
use std::net::UdpSocket;

use tracing_subscriber::fmt::format::FmtSpan;

/// Setup tracing output
fn setup_tracing_subscriber() {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .init();
}

/// Main
pub fn main() {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();

    let mut listener = outsim::Listener::udp(socket).unwrap();
    loop {
        let packet = listener.read().unwrap();
        tracing::info!(?packet);
    }
}
