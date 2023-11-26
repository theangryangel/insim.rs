//! Low level example of directly using connection::udp, tcp and relay directly.
//! In this example you are 100% responsible for managing the state of the connection,
//! providing the initial stream/udpsocket, sending keepalive packets, etc.
use clap::{Parser, Subcommand};
use if_chain::if_chain;
use insim::{
    codec::{Codec, Mode},
    insim::{Isi, IsiFlags},
    network::{Framed, FramedInner},
    packet::Packet,
    relay,
    result::Result,
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

    let cli = Cli::parse();

    let mut isi = Isi::default();
    isi.flags = IsiFlags::MCI | IsiFlags::CON | IsiFlags::OBH;
    isi.iname = "insim.rs".into();
    isi.interval = Duration::from_millis(1000);

    let mut client: Framed = match &cli.command {
        Commands::Udp { bind, addr } => {
            let local = bind.unwrap_or("0.0.0.0:0".parse()?);
            let stream = UdpSocket::bind(local).await.unwrap();
            isi.udpport = stream.local_addr().unwrap().port().into();
            stream.connect(addr).await.unwrap();

            Framed::Udp(FramedInner::new(stream, Codec::new(Mode::Compressed)))
        }
        Commands::Tcp { addr } => {
            let stream = TcpStream::connect(addr).await?;

            tracing::info!("Connected to server. Creating client");

            Framed::Tcp(FramedInner::new(stream, Codec::new(Mode::Compressed)))
        }
        Commands::Relay { .. } => {
            let stream = TcpStream::connect("isrelay.lfs.net:47474").await?;

            tracing::info!("Connected to LFSW Relay. Creating client");

            Framed::Tcp(FramedInner::new(stream, Codec::new(Mode::Uncompressed)))
        }
    };

    tracing::info!("Sending ISI");

    client.write(isi).await?;

    if let Commands::Relay {
        list_hosts: true, ..
    } = &cli.command
    {
        tracing::info!("Sending HLR");
        let hlr = relay::HostListRequest::default();
        client.write(hlr).await?;
    }

    if let Commands::Relay {
        select_host: Some(hname),
        ..
    } = &cli.command
    {
        tracing::info!("Sending HS");
        let hs = relay::HostSelect {
            hname: hname.into(),
            ..Default::default()
        };
        client.write(hs).await?;
    }

    tracing::info!("Connected!");

    let mut i = 0;

    loop {
        let m = client.read().await?;

        i += 1;

        tracing::info!("Packet={:?} Index={:?}", m, i);

        if_chain! {
            if let Packet::RelayHostList(i) = &m;
            if i.is_last();
            then {
                break;
            }
        }
    }

    Ok(())
}
