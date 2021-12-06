extern crate insim;
use tracing_subscriber;

use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::net::TcpStream;

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
#[allow(unused_must_use)]
pub async fn main() {
    setup();

    let tcp: TcpStream = TcpStream::connect("isrelay.lfs.net:47474").await.unwrap();

    let mut t =
        insim::protocol::transport::Transport::new(tcp, insim::protocol::codec::Mode::Uncompressed);
    let isi = insim::protocol::insim::Init {
        name: "insim.rs".into(),
        password: "".into(),
        prefix: b'!',
        version: insim::protocol::insim::VERSION,
        interval: 1000,
        flags: insim::protocol::insim::InitFlags::MCI,
        reqi: 1,
    };

    t.send(isi).await;

    t.send(insim::protocol::relay::HostSelect {
        hname: "Nubbins AU Demo".into(),
        ..Default::default()
    })
    .await;

    while let Some(m) = t.next().await {
        tracing::debug!("{:?}", m);
    }
}
