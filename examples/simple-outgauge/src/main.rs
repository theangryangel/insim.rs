//! A simple demo of the outgauge library
use std::net::UdpSocket;

use bytes::{Bytes, BytesMut};
use outgauge::{Outgauge, core::Decode};

/// Setup tracing output
fn setup_tracing_subscriber() {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}

/// Main
pub fn main() {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();

    loop {
        let mut raw = [0; 96];
        let (amt, src) = socket.recv_from(&mut raw).unwrap();

        let mut buf: Bytes = BytesMut::from(&raw[..amt]).freeze();

        let packet = Outgauge::decode(&mut buf).unwrap();
        tracing::info!("from={:?}, data={:?}", src, packet);
    }
}
