//! High level example
//! This example showcases the shortcut methods
use std::{net::SocketAddr, time::Duration};

use clap::Parser;
use insim::{
    identifiers::{PlayerId, RequestId},
    insim::{IsiFlags, LclFlags, Tiny, TinyType},
    Packet, Result,
};
use tokio::time::interval;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// host:port of LFS to connect to
    addr: SocketAddr,
}

struct ReversibleSequence<T> {
    sequence: Vec<T>,
    index: usize,
    reverse: bool,
}

impl<T: Copy> ReversibleSequence<T> {
    fn new(sequence: Vec<T>) -> Self {
        ReversibleSequence {
            sequence,
            index: 0,
            reverse: false,
        }
    }

    fn next(&mut self) -> &T {
        if self.index >= self.sequence.len() {
            self.reverse = !self.reverse;
            self.index = 0;
        }

        let result = if self.reverse {
            self.sequence.get(self.sequence.len() - 1 - self.index)
        } else {
            self.sequence.get(self.index)
        };

        self.index += 1;
        result.unwrap()
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Setup tracing_subcriber with some sane defaults
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse our command line arguments, using clap
    let cli = Cli::parse();

    let mut builder = insim::tcp(cli.addr);

    // set our IsiFlags
    builder = builder.isi_flags(IsiFlags::LOCAL);

    // Establish a connection
    let mut connection = builder.connect_async().await?;
    tracing::info!("Connected!");

    connection
        .write(Tiny {
            subt: TinyType::Npl,
            reqi: RequestId(1),
        })
        .await?;

    let mut plid: Option<PlayerId> = None;

    let mut interval = interval(Duration::from_millis(250));

    let mut sequence = ReversibleSequence::new(vec![
        LclFlags::SIGNAL_LEFT
            | LclFlags::LIGHT_OFF
            | LclFlags::FOG_REAR_OFF
            | LclFlags::FOG_FRONT_OFF,
        LclFlags::SIGNAL_OFF | LclFlags::LIGHT_HIGH | LclFlags::FOG_REAR | LclFlags::FOG_FRONT,
        LclFlags::SIGNAL_RIGHT
            | LclFlags::LIGHT_OFF
            | LclFlags::FOG_REAR_OFF
            | LclFlags::FOG_FRONT_OFF,
    ]);

    loop {
        tokio::select! {

            // TODO: We should probably find a way to pause the ticker, but whatever.
            _ = interval.tick() => {
                if plid.is_none() {
                    continue;
                }

                // insim implements From<LclFlags> for Packet, so we can shortcut, we dont need to
                // create a Small or a Packet::Small by hand if we're happy to use the default
                // values
                connection.write(*sequence.next()).await?;
            },

            packet = connection.read() => {

                match packet? {
                    Packet::Npl(npl) => {

                        if !npl.ptype.is_remote() && !npl.ptype.is_ai() {
                            plid = Some(npl.plid);
                            tracing::info!("Woot! local player joined! {:?}", plid);
                        }
                    },

                    Packet::Pll(pll) => {
                        if plid.map_or(false, |p| p == pll.plid) {
                            plid = None;

                            tracing::info!("Local player left!");
                        }
                    },

                    _ => {}

                }
            },
        }
    }
}
