//! ZINC Is Not CTRA: a small tribute to the community-run LFS network whose
//! points and licence system made clean public racing work. Not affiliated.
//!
//! - Races rotate between BL1 and FE1, and restart 60s after the winner.
//! - A confirmed, clean result (`Res` without a disqualification) earns a point.
//! - Yellow flags caused are counted.
//! - Each point unlocks more cars: UF1 to start, then XFR, XRG, the TBO class
//!   (XRT, RB4, FXO), and LX4. Drivers keep every car they unlock. See
//!   `zinc::LICENCES`.
//!
//! All in memory, so it resets on restart. Two services run side by side: a
//! `RaceDirector` for the rotation, and ZINC's rules as a `Router`.
//!
//! Run with:
//!     cargo run -p kitcar-zinc -- --addr 127.0.0.1:29999 --admin-password hunter2

#![allow(missing_docs)]

mod rank;
mod zinc;

use std::time::Duration;

use clap::Parser;
use insim::{
    Session,
    identifiers::RequestId,
    insim::{Isi, RaceLaps},
};
use insim_extra::kitcar::{
    Host,
    error::HostError,
    services::race_director::{Combo, Cycle, Every, RaceDirector},
};

/// Races per combo before rotating.
const RACES_PER_COMBO: u32 = 2;

#[derive(Parser, Debug)]
#[command(about = "ZINC Is Not CTRA: rotation, points, and cars unlocked by points")]
struct Args {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    admin_password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), HostError> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let stream = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::net::TcpStream::connect(args.addr.as_str()),
    )
    .await
    .map_err(insim::Error::from)?
    .map_err(insim::Error::from)?;
    stream.set_nodelay(true).map_err(insim::Error::from)?;
    let isi = Isi {
        reqi: RequestId(1),
        iname: "zinc".into(),
        prefix: '!',
        admin: args.admin_password.unwrap_or_default(),
        ..Isi::default()
    };
    let session = Session::tcp(stream)?.handshake(isi).await?;

    let mut host = Host::spawn(session);

    // No car restrictions here: the ZINC service licenses each driver's cars.
    let combo = |track: &str| {
        Combo::untimed(track.parse().expect("a valid track code")).laps(RaceLaps::Laps(5))
    };
    let director = RaceDirector::new(Cycle::new([combo("BL1"), combo("FE1")]))
        .every(Every::Races(RACES_PER_COMBO))
        .auto_restart(Duration::from_secs(60));
    let _director = host.start(director).await?;
    let _zinc = host.start(zinc::service()).await?;

    host.closed().await
}
