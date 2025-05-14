//! A simple demo of the outgauge library
use std::net::UdpSocket;

use bytes::{Bytes, BytesMut};
use outsim::{core::Decode, OutsimPack};

/// Setup tracing output
fn setup_tracing_subscriber() {
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

/// Main
pub fn main() {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();

    loop {
        let mut raw = [0; 68];
        let (amt, src) = socket.recv_from(&mut raw).unwrap();

        let mut buf: Bytes = BytesMut::from(&raw[..amt]).freeze();

        let packet = OutsimPack::decode(&mut buf).unwrap();
        tracing::info!("from={:?}, len={:?}, data={:?}", src, amt, packet);
    }
}
