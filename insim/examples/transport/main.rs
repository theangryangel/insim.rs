use clap::{Parser, Subcommand};
use if_chain::if_chain;
use insim::{
    packets::{relay::HostListRequest, Packet},
    prelude::*,
    result::Result,
};
use std::net::SocketAddr;

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

    // Use ConnectionBuilder to create a Connection
    let mut builder = ConnectionBuilder::default();

    // We need to box the output of the builder as connect_udp, connect_tcp, etc. all return
    // different types.
    // If you only call one of these, then you can skip the box'ing.
    let mut client = match &cli.command {
        Commands::Udp { bind, addr } => {
            // if the local binding address is not provided, we let the OS decide a port to use
            let local = bind.unwrap_or("0.0.0.0:0".parse()?);
            tracing::info!("Connecting via UDP!");
            let res = builder.connect_udp(local, addr).await?;
            res.boxed()
        }
        Commands::Tcp { addr } => {
            tracing::info!("Connecting via TCP!");
            let res = builder.connect_tcp(addr).await?;
            res.boxed()
        }
        Commands::Relay {
            select_host,
            list_hosts,
        } => {
            let host = match (select_host, list_hosts) {
                (Some(host), false) => Some(host.as_str()),
                (_, _) => None,
            };
            tracing::info!("Connecting via LFS World Relay!");
            let mut res = builder.connect_relay(host).await?;
            if *list_hosts {
                res.send(HostListRequest::default()).await?;
            }
            res.boxed()
        }
    };

    tracing::info!("Connected!");

    let mut i = 0;

    while let Some(m) = client.next().await {
        i += 1;

        let m = m?;

        tracing::info!("Packet={:?} Index={:?}", m, i);

        if_chain! {
            if let Commands::Relay{ list_hosts: true, .. } = &cli.command;
            if let Packet::RelayHostList(i) = &m;
            if i.is_last();
            then {
                break;
            }
        }
    }

    Ok(())
}
