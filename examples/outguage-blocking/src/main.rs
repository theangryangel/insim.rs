use std::{
    io::Cursor,
    net::{SocketAddr, UdpSocket},
};

use outgauge::{
    core::binrw::{BinRead, BinWrite},
    outgauge::OutgaugePack,
};

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

pub fn main() {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();

    loop {
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 1024];
        let (amt, _src) = socket.recv_from(&mut buf).unwrap();

        let packet = OutgaugePack::read(&mut Cursor::new(&buf[..amt])).unwrap();

        tracing::debug!("{:?}", &packet);
    }
}
