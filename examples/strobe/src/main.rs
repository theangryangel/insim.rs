#![doc = include_str!("../README.md")]
use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use insim::{
    Packet, Result, WithRequestId,
    identifiers::PlayerId,
    insim::{LclFlags, TinyType},
};
use serde::Deserialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// host:port of LFS to connect to
    addr: SocketAddr,

    /// path to YAML sequence file
    file: PathBuf,
}

/// One step in the light sequence.
///
/// `flags` is a `|`-separated list of [`LclFlags`] names, e.g.
/// `SIGNAL_LEFT | LIGHT_OFF | FOG_FRONT_OFF | FOG_REAR_OFF`.
#[derive(Deserialize)]
struct StepConfig {
    duration_ms: u64,
    flags: LclFlags,
}

/// The top-level structure of the YAML file.
#[derive(Deserialize)]
struct Config {
    steps: Vec<StepConfig>,
}

/// Extension trait to extract the `plid` from packets that carry one,
/// letting us handle [`Packet::Pll`] and [`Packet::Plp`] in a single arm.
trait PacketExt {
    fn plid(&self) -> Option<PlayerId>;
}

impl PacketExt for Packet {
    fn plid(&self) -> Option<PlayerId> {
        match self {
            Self::Plp(p) => Some(p.plid),
            Self::Pll(p) => Some(p.plid),
            _ => None,
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Read and parse the YAML sequence file. LclFlags deserialises from a
    // `|`-separated string of flag names via the bitflags serde integration.
    let content = std::fs::read_to_string(&cli.file)?;
    let config: Config = serde_norway::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let step_vec: Vec<(Duration, LclFlags)> = config
        .steps
        .iter()
        .map(|s| (Duration::from_millis(s.duration_ms), s.flags))
        .collect();

    let mut steps = step_vec.iter().cycle();

    // Connect to LFS. `isi_flag_local` tells LFS that we are local insim application.
    // `isi_iname` is the application name.
    let mut connection = insim::tcp(cli.addr)
        .isi_flag_local(true)
        .isi_iname(Some("insim.rs/strobe".to_string()))
        .connect_async()
        .await?;

    tracing::info!("Connected!");

    // Request the current player list so we receive an Npl packet for any
    // player already on track when we connect, not just future joiners.
    connection.write(TinyType::Npl.with_request_id(1)).await?;

    // We only send light commands when a local (human, non-AI) player is on
    // track. `plid` holds their player ID once we see their Npl packet.
    let mut plid: Option<PlayerId> = None;

    // Pre-fetch the first step so we always have a valid (duration, flags)
    // pair to refer to, even before the first tick fires.
    let (mut current_duration, mut current_flags) = *steps.next().unwrap();

    // `next_tick` tracks when the current step expires. Initialising to
    // `Instant::now()` means the first tick fires immediately on join.
    let mut next_tick = tokio::time::Instant::now();

    loop {
        tokio::select! {
            // The `if plid.is_some()` guard disables this branch entirely
            // while no local player is on track, effectively pausing the timer.
            _ = tokio::time::sleep_until(next_tick), if plid.is_some() => {
                connection.write(current_flags).await?;
                // Advance the deadline by this step's duration before moving
                // on, so any processing delay doesn't cause drift over time.
                next_tick = next_tick + current_duration;
                (current_duration, current_flags) = *steps.next().unwrap();
            },

            packet = connection.read() => {
                match packet? {
                    // Npl fires when a player joins the track (including when
                    // returning from the pits). We only care about the local
                    // human player — remote and AI players are ignored.
                    Packet::Npl(npl) => {
                        if !npl.ptype.is_remote() && !npl.ptype.is_ai() {
                            plid = Some(npl.plid);
                            // Reset the deadline so the sequence starts
                            // immediately rather than mid-sleep.
                            next_tick = tokio::time::Instant::now();
                            tracing::info!("Local player joined! {:?}", plid);
                        }
                    },

                    // Pll fires when a player leaves the session entirely;
                    // Plp fires when they drive into the pits. Both should
                    // pause the strobe and reset the sequence to the start.
                    p @ (Packet::Pll(_) | Packet::Plp(_)) => {
                        if p.plid() == plid {
                            tracing::info!("Local player left! {:?}", plid);
                            plid = None;
                            steps = step_vec.iter().cycle();
                            (current_duration, current_flags) = *steps.next().unwrap();
                        }
                    },

                    _ => {}
                }
            },
        }
    }
}
