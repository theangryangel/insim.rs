//! High level example
//! This example showcases the shortcut methods
use clap::{Parser, Subcommand};
use if_chain::if_chain;
use insim::{insim::IsiFlags, relay::Hlr, Packet, Result};
use std::{net::SocketAddr, time::Duration};

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

    /// Connect via LFS World Relay
    Relay {
        #[arg(long)]
        /// Optional host to automatically select after successful connection to relay
        select_host: Option<String>,

        #[arg(long)]
        /// List hosts on the relay and then quit
        list_hosts: bool,

        #[arg(long)]
        websocket: bool,

        #[arg(long)]
        spectator_password: Option<String>,
    },
}

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

#[tokio::main]
pub async fn main() -> Result<()> {
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
        Commands::Relay {
            select_host,
            websocket,
            spectator_password,
            ..
        } => {
            tracing::info!("Connecting via LFS World Relay!");

            // use insim relay
            insim::relay()
                .relay_websocket(*websocket)
                .relay_spectator_password(spectator_password.clone())
                .relay_select_host(select_host.clone())
        },
    };

    // set our IsiFlags
    builder = builder.isi_flags(IsiFlags::MCI | IsiFlags::CON | IsiFlags::OBH);
    if let Some(interval) = &cli.isi_interval {
        builder = builder.isi_interval(Duration::from_secs((*interval).into()));
    }

    // Establish a connection
    let mut connection = builder.connect().await?;
    tracing::info!("Connected!");

    // If we're connected via the relay, and asked to list the hosts, request the host list
    if let Commands::Relay {
        list_hosts: true, ..
    } = &cli.command
    {
        connection.write(Hlr::default()).await?;
    }

    let mut i: usize = 0;

    loop {
        let packet = connection.read().await?;

        tracing::info!("Packet={:?} Index={:?}", packet, i);

        // if we were connected via the relay and only asked for the list of hosts, and we have the
        // last hostinfo, break the loop
        if_chain! {
            if let Commands::Relay{ list_hosts: true, .. } = &cli.command;
            if let Packet::RelayHos(hostinfo) = &packet;
            if hostinfo.is_last();
            then {
                break;
            }
        }

        i = i.wrapping_add(1);
    }

    Ok(())
}
