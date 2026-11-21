//! TCP control with InSim, OutGauge and OutSim sharing an attached UDP socket.
use std::{net::SocketAddr, time::Duration};

use clap::Parser;
use insim::{
    Session,
    identifiers::RequestId,
    insim::{Isi, IsiFlags, SmallType},
    net::{Demux, InsimCodec, OutGaugeCodec, OutSimCodec},
};
use tokio::net::{TcpStream, UdpSocket};
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long)]
    /// host:port of LFS to connect to
    addr: SocketAddr,

    #[arg(long)]
    /// host:port to bind a UDP listener to (port 0 selects an available port).
    udp_listen_addr: SocketAddr,
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
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    // Parse our command line arguments, using clap
    let cli = Cli::parse();

    let udp = UdpSocket::bind(cli.udp_listen_addr).await?;
    let udp_addr = udp.local_addr()?;
    let stream =
        tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(cli.addr)).await??;
    stream.set_nodelay(true)?;

    let isi = Isi {
        reqi: RequestId(1),
        udpport: udp_addr.port(),
        flags: IsiFlags::MCI
            | IsiFlags::CON
            | IsiFlags::OBH
            | IsiFlags::AXM_LOAD
            | IsiFlags::AXM_EDIT,
        interval: Duration::from_secs(1),
        ..Isi::default()
    };

    // SSG and SSP send OutGauge and OutSim to Isi.udpport, alongside MCI and NLP.
    // Telemetry is recognized by size, so InsimCodec goes last. Known issue: an
    // NLP for 10 or 15 players has a telemetry size and is misread (NLP is not
    // enabled here).
    let decoder = Demux::new()
        .with(OutGaugeCodec)
        .with(OutSimCodec::default())
        .with(InsimCodec);
    let mut connection = Session::tcp(stream)?
        .attach_udp(udp, decoder)?
        .handshake(isi)
        .await?;

    tracing::info!(%udp_addr, "Connected; receiving InSim, OutGauge and OutSim over UDP");

    // Start sending gauges (outgauge)
    connection
        .write(SmallType::Ssg(Duration::from_secs(1)))
        .await?;

    // Start sending positions (outsim)
    connection
        .write(SmallType::Ssp(Duration::from_secs(1)))
        .await?;

    loop {
        let packet = connection.read().await?;
        tracing::info!(?packet);
    }
}
