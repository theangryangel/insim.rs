//! High level example
//! This example showcases the shortcut methods
use std::{net::SocketAddr, time::Duration};

use clap::{Parser, Subcommand};

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
        .init();
}

pub fn main() -> insim::Result<()> {
    // Setup tracing_subcriber with some sane defaults
    setup_tracing_subscriber();

    // Parse our command line arguments, using clap
    let cli = Cli::parse();

    // Create an insim connection builder
    let mut builder = match &cli.command {
        Commands::Udp { bind, addr } => {
            tracing::info!("Connecting via UDP!");
            // use udp
            insim::udp(*addr, *bind)
        },
        Commands::Tcp { addr } => {
            tracing::info!("Connecting via TCP!");
            // use udp
            insim::tcp(*addr)
        },
    };

    // set our IsiFlags
    builder = builder
        .isi_flag_mso_cols(true)
        .isi_flag_mci(true)
        .isi_flag_con(true)
        .isi_flag_obh(true)
        .isi_flag_hlv(true);

    if let Some(interval) = &cli.isi_interval {
        builder = builder.isi_interval(Duration::from_secs((*interval).into()));
    }

    // Establish a connection
    let mut connection = builder.connect_blocking()?;
    tracing::info!("Connected!");

    let mut i: usize = 0;

    loop {
        let packet = connection.read()?;
        tracing::info!("Packet={:?} Index={:?}", packet, i);
        i = i.wrapping_add(1);
    }
}
