//! Explicit Tokio sockets and an InSim session handshake.
use std::{net::SocketAddr, time::Duration};

use clap::{Parser, Subcommand};
use insim::{
    Session,
    identifiers::RequestId,
    insim::{Isi, IsiFlags},
    net::InsimCodec,
};
use tokio::net::{TcpStream, UdpSocket};
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long)]
    isi_interval: Option<u8>,
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

        #[arg(long)]
        /// Bind an additional UDP socket for MCI/NLP replies (use port 0 for a random port).
        udp_bind: Option<SocketAddr>,
    },
}

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

#[tokio::main]
async fn main() -> insim::Result<()> {
    setup_tracing_subscriber();

    let cli = Cli::parse();

    // The handshake requires a nonzero ReqI.
    let mut isi = Isi {
        reqi: RequestId(1),
        flags: IsiFlags::MCI
            | IsiFlags::CON
            | IsiFlags::OBH
            | IsiFlags::AXM_LOAD
            | IsiFlags::AXM_EDIT,
        interval: Duration::from_secs(cli.isi_interval.unwrap_or(1).into()),
        ..Isi::default()
    };

    let session = match cli.command {
        Commands::Udp { bind, addr } => {
            tracing::info!("Connecting via UDP!");
            let socket = UdpSocket::bind(bind.unwrap_or_else(|| ([0, 0, 0, 0], 0).into())).await?;
            socket.connect(addr).await?;
            isi.udpport = socket.local_addr()?.port();
            Session::udp(socket)?
        },
        Commands::Tcp { addr, udp_bind } => {
            tracing::info!("Connecting via TCP!");
            let stream =
                tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(addr)).await??;
            stream.set_nodelay(true)?;
            let session = Session::tcp(stream)?;
            if let Some(bind) = udp_bind {
                let socket = UdpSocket::bind(bind).await?;
                let local_addr = socket.local_addr()?;
                isi.udpport = local_addr.port();
                tracing::info!(%local_addr, "Receiving MCI/NLP over UDP; control uses TCP");
                session.attach_udp(socket, InsimCodec)?
            } else {
                session
            }
        },
    };

    // Ver remains available through read(), alongside subsequent packets.
    let mut connection = session.handshake(isi).await?;
    tracing::info!("Connected!");

    let mut i: usize = 0;

    loop {
        let packet = connection.read().await?;

        tracing::info!("Packet={:?} Index={:?}", packet, i);

        i = i.wrapping_add(1);
    }
}
