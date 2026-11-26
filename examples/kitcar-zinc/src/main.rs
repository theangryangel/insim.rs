//! ZINC Is Not CTRA: a small tribute to the community-run LFS network whose
//! points and licence system made clean public racing work. Not affiliated.
//!
//! - Races rotate, reshuffled each pass, through KY1X with the "Mini Oval"
//!   layout (10 laps, UF1 only), SO6X with the "Park Life Circuit V1" layout, FE1 (3 laps with a pit stop
//!   and a reversed grid, at midnight, midday, or an evening race into the
//!   dark, picked each visit), AS1 on live time with a random grid, BL1 at
//!   sunrise, and BL3 rallycross with a random grid. The rest are one lap, and
//!   races restart 60s after the winner.
//! - FE1 allows the UF1, XFG and 1DFF72; BL3 the UF1 and 1DFF72; AS1 and BL1
//!   every car on the licence ladder.
//! - A confirmed, clean result (`Res` without a disqualification) earns a point.
//! - Yellow flags and collisions are counted, but only while you're racing:
//!   not in the lobby, and not once you've finished.
//! - A topbar pinned to every screen shows the race, points, rank and the next
//!   car to unlock.
//! - A message of the day greets everyone as they connect, until they close it.
//! - `!top` opens the leaderboard, a page at a time. `!clock` puts a ticking
//!   clock mid-screen until it's clicked.
//! - Each point unlocks more cars: UF1 and XFG to start, then the 1DFF72 mod,
//!   XFR, XRG, the TBO class (XRT, RB4, FXO), and LX4. Drivers keep every car
//!   they unlock. See `LICENCES`, which the track limits share cars with.
//!
//! All in memory, so it resets on restart. The host's `RaceDirector` runs the
//! rotation, and ZINC's rules run as a `Router` that reads it for the topbar.
//!
//! Run with:
//!     cargo run -p kitcar-zinc -- --addr 127.0.0.1:29999 --admin-password hunter2

#![allow(missing_docs)]

mod clock;
mod rank;
mod top;
mod topbar;
mod zinc;

use std::time::Duration;

use clap::Parser;
use insim::{
    Session,
    core::vehicle::Vehicle,
    identifiers::RequestId,
    insim::{Isi, IsiFlags, RaceLaps, VtnAction},
};
use insim_extra::{
    commands::time::{Month, TimeOfDay},
    kitcar::{
        Host,
        director::{Combo, Cycle, Layout, RaceDirector, StartOrder},
        error::HostError,
        services::{motd::Motd, no_vote::NoVote},
    },
};

/// The 1DFF72 mod, by its id. The server must allow mods for it to be
/// drivable; the director turns them on where a combo lists one.
const MOD_1DFF72: Vehicle = Vehicle::Mod(0x1D_FF72);

/// Where every driver starts.
const STARTER: &[Vehicle] = &[Vehicle::Uf1, Vehicle::Xfg];

/// The licence ladder: rung `n` needs `n` points, and rungs stack, so 3 points
/// unlocks everything from the starter cars to the XRG. Add rungs to extend it.
const LICENCES: zinc::Ladder = &[
    STARTER,
    &[MOD_1DFF72],
    &[Vehicle::Xfr],
    &[Vehicle::Xrg],
    // The TBO class.
    &[Vehicle::Xrt, Vehicle::Rb4, Vehicle::Fxo],
    &[Vehicle::Lx4],
];

/// Every car on the ladder: what an ordinary track allows. Anything else (an
/// FZR, a BF1) is greyed out in the garage.
fn ladder_cars() -> Vec<Vehicle> {
    LICENCES
        .iter()
        .flat_map(|rung| rung.iter().copied())
        .collect()
}

/// FE1: the starter cars, and the first car a point earns.
const FERN_BAY: &[Vehicle] = &[Vehicle::Uf1, Vehicle::Xfg, MOD_1DFF72];

/// BL3 rallycross: the starter UF1, and the first car a point earns.
const RALLYCROSS: &[Vehicle] = &[Vehicle::Uf1, MOD_1DFF72];

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
    // Info by default, so the director and ZINC say what they're doing.
    // `RUST_LOG` overrides it.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
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
        // LFS asks ZINC before anyone takes the track, so it can check their
        // licence. LFS waits for the answer, so joins stall if ZINC stops
        // answering.
        // CON reports contact between cars, for the collision count.
        flags: IsiFlags::REQ_JOIN | IsiFlags::CON,
        ..Isi::default()
    };
    let session = Session::tcp(stream)?.handshake(isi).await?;

    let mut host = Host::spawn(session);

    // No car restrictions here: the ZINC service licenses each driver's cars.
    // Each visit picks one of a combo's times at random.
    let midnight = TimeOfDay::fixed(0, 0);
    let midday = TimeOfDay::fixed(12, 0);
    // LFS has no sunrise preset, and the real sunrise moves with the date.
    let sunrise = TimeOfDay::fixed(6, 0);
    // 17:00 at the equinox, at 60x: sunset about a minute in, fully dark by
    // about 2.5, so FE1's 3 laps finish at night.
    let into_the_night = TimeOfDay::Set {
        date: Some((21, Month::Mar)),
        at: Some((17, 0)),
        utc: None,
        speed: 60,
    };
    // Each combo names its cars: the director greys out the rest in the
    // garage, and ZINC's licences narrow it per driver at the join. One race
    // each, then the next track.
    let combo = |track: &str, times: &[TimeOfDay], cars: &[Vehicle]| {
        Combo::untimed(track.parse().expect("a valid track code"))
            .laps(RaceLaps::Laps(1))
            .times(times.iter().copied())
            .vehicles(cars.iter().copied())
            .races(1)
    };
    let ladder = ladder_cars();
    // Each start order and the pit rule come up at least once a pass, so a
    // test session exercises them all.
    let director = RaceDirector::new(
        Cycle::new([
            // First, so they come up straight away: layouts loaded by name.
            // The Mini Oval is tiny, so it runs 10 laps, in UF1s for the lols.
            combo("KY1X", &[midday], &[Vehicle::Uf1])
                .layout(Layout::Axload("Mini Oval".into()))
                .laps(RaceLaps::Laps(10))
                .name("Kyoto Mini Oval"),
            combo("SO6X", &[midday], &ladder)
                .layout(Layout::Axload("Park Life Circuit V1".into()))
                .name("Park Life Circuit"),
            combo("FE1", &[midnight, midday, into_the_night], FERN_BAY)
                .laps(RaceLaps::Laps(3))
                .must_pit(true)
                .start(StartOrder::Reverse)
                .name("Fern Bay Club"),
            combo("AS1", &[TimeOfDay::live()], &ladder)
                .start(StartOrder::Random)
                .name("Aston Cadet"),
            combo("BL1", &[sunrise], &ladder).name("Blackwood GP"),
            combo("BL3", &[midday], RALLYCROSS)
                .start(StartOrder::Random)
                .name("Blackwood Rallycross"),
        ])
        .shuffle(),
    )
    .auto_restart(Duration::from_secs(60));
    host.start_director(director).await?;
    let _zinc = host.start(zinc::service(LICENCES)).await?;
    // A restart or qualifying vote would cut across the director's rotation.
    // Voting to end the race is fine: the director moves on from there.
    let _no_vote = host
        .start(
            NoVote::new([VtnAction::Restart, VtnAction::Qualify]).message(
                "^3Sorry, no restart or qualifying votes here - races restart on their own",
            ),
        )
        .await?;
    let _motd = host
        .start(
            Motd::new("ZINC Is Not CTRA")
                .line("A tribute to clean community racing.")
                .line("This test is about stability, not points or licences.")
                .line("Your progress resets whenever the server restarts.")
                .line("Only participation points are currently awarded.")
                .line("Type ^3!help^7 for commands.")
                .line("Thanks for testing with us!")
                .button("LET'S RACE"),
        )
        .await?;

    host.closed().await
}
