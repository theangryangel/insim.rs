//! Low level example of directly using connection::udp, tcp and relay directly.
//! In this example you are 100% responsible for managing the state of the connection,
//! providing the initial stream/udpsocket, sending keepalive packets, etc.
use clap::{Parser, Subcommand};
use if_chain::if_chain;
use insim::{
    packets,
    result::Result,
    traits::{ReadPacket, ReadWritePacket, WritePacket}, framed::Framed,
};
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpStream, UdpSocket};

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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect via UDP
    Udp {
        #[arg(long)]
        /// Local address to bind to. If not provided a random port will be used.
        bind: Option<SocketAddr>,

        #[arg(long)]
        /// host:port of LFS to connect to
        addr: SocketAddr,
    },

    /// Connect via TCP
    Tcp {
        #[arg(long)]
        /// host:port of LFS to connect to
        addr: SocketAddr,
    },

    /// Connect via LFS World Relay
    Relay {
        #[arg(long)]
        /// Optional host to automatically select after successful connection to relay
        select_host: Option<String>,

        #[arg(long)]
        /// List hosts on the relay and then quit
        list_hosts: bool,
    },
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    // let stream = TcpStream::connect("isrelay.lfs.net:47474").await?;
    let stream = insim::framed::websocket::connect_to_relay().await?;

    tracing::info!("Connected to LFSW Relay. Creating client");

    use insim::framed::codec::v9;

    let codec = v9::Codec { 
        mode: insim::codec::Mode::Uncompressed 
    };

    let mut client = Framed::new(stream, codec);

    let isi = v9::insim::Isi {
        iname: "insim.rs".into(),
        version: client.version(),
        flags: v9::insim::IsiFlags::MCI
            | v9::insim::IsiFlags::CON
            | v9::insim::IsiFlags::OBH,
        interval: Duration::from_millis(1000),
        ..Default::default()
    };

    tracing::info!("Sending ISI {:?}", &isi);

    client.write(isi).await?;

    tracing::info!("Sending HLR");
    let hlr = v9::relay::HostListRequest::default();
    client.write(hlr).await?;

    tracing::info!("Connected!");

    let mut i = 0;

    loop {
        let m = client.read().await?;
        tracing::info!("{:?}", m);


    }
}
