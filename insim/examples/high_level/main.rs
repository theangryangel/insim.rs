//! High level example
//! In this example you do not have to maintain the connection,
//! manage sending keepalives, or reconnection.
//! Just keep calling `poll` on your connection!
use clap::{Parser, Subcommand};
use if_chain::if_chain;
use insim::{
    codec::{Frame, Mode},
    connection::{Connection, Event},
    relay::HostListRequest,
    result::Result,
    v9::{IsiFlags, Packet},
};
use std::{net::SocketAddr, time::Duration};

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

    let mut isi = Packet::isi_default();
    isi.flags = IsiFlags::MCI | IsiFlags::CON | IsiFlags::OBH;
    isi.iname = "insim.rs".into();
    isi.interval = Duration::from_millis(1000);

    let mut client: Connection<Packet> = match &cli.command {
        Commands::Udp { bind, addr } => {
            // if the local binding address is not provided, we let the OS decide a port to use
            let local = bind.unwrap_or("0.0.0.0:0".parse()?);
            tracing::info!("Connecting via UDP!");
            Connection::udp(local, *addr, Mode::Compressed, true, isi)
        }
        Commands::Tcp { addr } => {
            tracing::info!("Connecting via TCP!");
            Connection::tcp(Mode::Compressed, *addr, true, isi)
        }
        Commands::Relay {
            select_host,
            websocket,
            spectator_password,
            ..
        } => {
            tracing::info!("Connecting via LFS World Relay!");
            Connection::relay(
                select_host.clone(),
                *websocket,
                spectator_password.clone(),
                None,
                isi,
            )
        }
    };

    let mut i: usize = 0;

    loop {
        let event = client.poll().await?;

        if matches!(event, Event::Connected(_)) {
            if let Commands::Relay {
                list_hosts: true, ..
            } = &cli.command
            {
                client.send(HostListRequest::default()).await?;
            }

            tracing::info!("Connected!");
        }

        tracing::info!("Evt={:?} Index={:?}", event, i);

        if_chain! {
            if let Commands::Relay{ list_hosts: true, .. } = &cli.command;
            if let Event::Data(packet, _) = &event;
            if let Packet::RelayHostList(hostinfo) = &packet;
            if hostinfo.is_last();
            then {
                break;
            }
        }

        i = i.wrapping_add(1);
    }

    Ok(())
}
