//! High level example of using a combination of TCP and UDP.
use std::{net::SocketAddr, time::Duration};

use bytes::{Buf, Bytes, BytesMut};
use clap::Parser;
use insim::{Packet, core::Decode, insim::SmallType};
use outgauge::Outgauge;
use outsim::OutsimPack;
use tokio::net::UdpSocket;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long)]
    /// host:port of LFS to connect to
    addr: SocketAddr,

    #[arg(long)]
    /// host:port to bind a udp listener to.
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
        .init();
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    // Parse our command line arguments, using clap
    let cli = Cli::parse();

    let udp = UdpSocket::bind(cli.udp_listen_addr).await?;
    // Make a buffer for the UdpSocket, we'll reuse this across recv_from calls
    let mut buf = [0; 1024];

    // Make our TCP connection
    let mut connection = insim::tcp(cli.addr)
        .isi_flag_mci(true)
        .isi_flag_con(true)
        .isi_flag_obh(true)
        .isi_flag_axm_load(true)
        .isi_flag_axm_edit(true)
        .isi_interval(Duration::from_secs(1))
        // Force the UDP port, so that it's sent to our socket
        .isi_udpport(cli.udp_listen_addr.port())
        .connect_async()
        .await?;

    // Establish a connection
    tracing::info!("Connected!");

    // Start sending gauges (outgauge)
    connection
        .write(SmallType::Ssg(Duration::from_secs(1)))
        .await?;

    // Start sending positions (outsim)
    connection
        .write(SmallType::Ssp(Duration::from_secs(1)))
        .await?;

    loop {
        tokio::select! {
            packet = connection.read() => {
                let packet = packet?;
                tracing::info!("{:?}", packet);
            },
            res = udp.recv_from(&mut buf) => {
                let (amt, src) = res?;
                let mut buf: Bytes = BytesMut::from(&buf[..amt]).freeze();

                if amt == 92 {
                    // Conventionally it's outgauge
                    let packet = Outgauge::decode(&mut buf)?;
                    tracing::info!("outgauge: from={:?}, data={:?}", src, packet);
                } else if amt == 64 {
                    // Conventionally it's outsim
                    let packet = OutsimPack::decode(&mut buf)?;
                    tracing::info!("outsim: from={:?}, data={:?}", src, packet);
                } else if amt > 1 {
                    // Otherwise it's probably a Mci or Nlp packet
                    //
                    // XXX: we need to chop off the first byte, which is the length of the packet.
                    // That's normally handled internally
                    // by the insim crate.
                    buf.advance(1);
                    let packet = Packet::decode(&mut buf)?;
                    tracing::info!("insim: from={:?}, data={:?}", src, packet);
                }
            }
        }
    }
}
