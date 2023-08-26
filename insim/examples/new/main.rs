use clap::{Parser, Subcommand};
use if_chain::if_chain;
use insim::{
    packets,
    result::Result,
    connection::traits,
};
use tokio::net::{TcpStream, UdpSocket};
use std::{net::SocketAddr, time::Duration};

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
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    let cli = Cli::parse();

    let mut isi = packets::insim::Isi {
        iname: "insim.rs".into(),
        version: packets::VERSION,
        flags: packets::insim::IsiFlags::MCI | packets::insim::IsiFlags::CON | packets::insim::IsiFlags::OBH,
        interval: Duration::from_millis(1000),
        ..Default::default()
    };

    let mut client = match &cli.command {
        Commands::Udp { bind, addr } => {
            let local = bind.unwrap_or("0.0.0.0:0".parse()?);
            let stream = UdpSocket::bind(local).await.unwrap();
            isi.udpport = stream.local_addr().unwrap().port().into();
            stream.connect(addr).await.unwrap();

            traits::ReadWritePacket::boxed(
                traits::Udp::new(
                    stream, 
                    insim::codec::Mode::Compressed
                )
            )
        },
        Commands::Tcp { addr } => {
            let stream = TcpStream::connect(addr).await?;

            tracing::info!("Connected to server. Creating client");

            traits::ReadWritePacket::boxed(
                traits::Tcp::new(stream, insim::codec::Mode::Compressed)
            )
        }
    };

    tracing::info!("Sending ISI");

    client.write(isi.into()).await?;

    // tracing::info!("Sending HLR");
    // let hlr = packets::relay::HostListRequest::default();
    // traits::WritePacket::write(&mut client, hlr).await?;

    tracing::info!("Connected!");

    let mut i = 0;

    while let Some(m) = client.read().await? {
        i += 1;

        tracing::info!("Packet={:?} Index={:?}", m, i);

        if_chain! {
            if let packets::Packet::Tiny(i) = &m;
            if i.is_keepalive();
            then {
                let pong = packets::insim::Tiny{
                    subt: packets::insim::TinyType::None,
                    ..Default::default()
                };

                client.write(pong.into()).await?;

                println!("ping? pong!");
            }

        }

        if_chain! {
            if let packets::Packet::RelayHostList(i) = &m;
            if i.is_last();
            then {
                break;
            }
        }
    }

    Ok(())
}
