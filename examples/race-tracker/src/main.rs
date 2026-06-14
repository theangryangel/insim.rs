//! Bare-insim race tracker example using [`World`].
//!
//! - On `SessionStarted` (an `Rst`) the tracker clears, so it re-requests the
//!   player list and grid order to repopulate for the new session.
//! - On each confirmed result (`Res` -> `ResultConfirmed`) it reprints the
//!   table, so the classification builds up live as drivers cross the line.
//! - On `SessionEnded` (back to the lobby screen) it prints the final results
//!   table, including each entrant's starting grid position.
//!
//! Run with:
//!     cargo run -p race-tracker -- --addr 127.0.0.1:29999

#![allow(missing_docs)]

use clap::Parser;
use insim::WithRequestId;
use insim_extra::{
    race::{EntrantState, FinishStatus, RaceEvent},
    world::{World, WorldEvent},
};
use tabled::{Table, Tabled, settings::Style};

#[derive(Parser, Debug)]
#[command(about = "Race tracker example - prints results when the race ends")]
struct Args {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    admin_password: Option<String>,
}

#[tokio::main]
async fn main() -> insim::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();

    let mut builder = insim::tcp(args.addr.as_str()).isi_iname("race-tracker".to_string());
    if let Some(pw) = args.admin_password {
        builder = builder.isi_admin_password(pw);
    }

    let mut conn = builder.connect_async().await?;
    tracing::info!("connected");

    // Send startup requests once to sync state from LFS.
    for t in World::STARTUP_REQUESTS {
        conn.write(t.clone().with_request_id(1)).await?;
    }

    let world = World::new();

    loop {
        let packet = conn.read().await?;

        for event in world.apply_packet(&packet) {
            match event {
                WorldEvent::SessionStarted { kind } => {
                    tracing::info!(?kind, "session started");
                    // The tracker just cleared - re-request its packets so it
                    // repopulates for the new session.
                    for t in World::SESSION_REQUESTS {
                        conn.write(t.clone().with_request_id(1)).await?;
                    }
                },
                WorldEvent::SessionEnded => {
                    tracing::info!("session ended");
                    print_results(&world, "Final Race Results");
                },
                WorldEvent::Race(ref e) => {
                    match e {
                        RaceEvent::PersonalBest {
                            id,
                            lap,
                            time,
                            previous,
                            ..
                        } => {
                            let driver = world
                                .entrant(*id)
                                .and_then(|e| e.drivers.last().map(|d| d.pname.clone()))
                                .unwrap_or_else(|| "?".to_string());
                            let prev = previous
                                .map(|p| format!(" (was {})", fmt_duration(p)))
                                .unwrap_or_default();
                            println!(
                                "[PB] {:<20}  {}{}  lap {}",
                                driver,
                                fmt_duration(*time),
                                prev,
                                lap
                            );
                        },
                        RaceEvent::FastestLap { id, lap, time, .. } => {
                            let driver = world
                                .entrant(*id)
                                .and_then(|e| e.drivers.last().map(|d| d.pname.clone()))
                                .unwrap_or_else(|| "?".to_string());
                            println!("[FL] {:<20}  {}  lap {}", driver, fmt_duration(*time), lap);
                        },
                        // Each confirmed result classifies one more driver. Reprint the
                        // table so it builds up live as drivers cross the line; the
                        // final copy is printed on session end.
                        RaceEvent::ResultConfirmed {
                            result_num,
                            num_results,
                            ..
                        } => {
                            print_results(
                                &world,
                                &format!("Results ({}/{} classified)", result_num + 1, num_results),
                            );
                        },
                        _ => tracing::debug!(?e, "race event"),
                    }
                },
                _ => {},
            }
        }
    }
}

fn fmt_duration(d: std::time::Duration) -> String {
    let total = d.as_millis();
    format!("{}:{:06.3}", total / 60000, (total % 60000) as f64 / 1000.0)
}

#[derive(Tabled)]
struct ResultRow {
    #[tabled(rename = "#")]
    pos: String,
    #[tabled(rename = "Grid")]
    grid: String,
    #[tabled(rename = "Driver")]
    driver: String,
    #[tabled(rename = "Laps")]
    laps: u16,
    #[tabled(rename = "Best Lap")]
    best_lap: String,
    #[tabled(rename = "Stops")]
    stops: usize,
    #[tabled(rename = "Status")]
    status: String,
}

fn print_results(world: &World, title: &str) {
    let mut entrants = world.entrants();
    entrants.sort_by_key(position_key);

    let fl_id = world.fastest_lap().map(|(id, _, _)| id);

    let rows: Vec<ResultRow> = entrants
        .iter()
        .map(|e| {
            let driver = e
                .drivers
                .last()
                .map(|d| d.pname.clone())
                .unwrap_or_else(|| "?".to_string());
            let best_lap = e
                .best_lap
                .map(|d| {
                    let fl = if fl_id == Some(e.id) { " *" } else { "" };
                    format!("{}{}", fmt_duration(d), fl)
                })
                .unwrap_or_else(|| "-".to_string());

            let (pos, status) = match &e.status {
                FinishStatus::Finished { result_num, .. } => (
                    result_num
                        .map(|n| format!("{}", n + 1))
                        .unwrap_or_else(|| "-".to_string()),
                    "Finished".to_string(),
                ),
                FinishStatus::Racing => ("-".to_string(), "Racing".to_string()),
                FinishStatus::Dnf => ("-".to_string(), "DNF".to_string()),
            };

            let grid = e
                .grid_position
                .map(|g| g.to_string())
                .unwrap_or_else(|| "-".to_string());

            ResultRow {
                pos,
                grid,
                driver,
                laps: e.laps_done,
                best_lap,
                stops: e.pit_stops.len(),
                status,
            }
        })
        .collect();

    println!();
    println!("=== {title} ===");
    println!("{}", Table::new(rows).with(Style::sharp()));
    println!();
}

fn position_key(e: &EntrantState) -> (u8, i32) {
    match &e.status {
        FinishStatus::Finished { result_num, .. } => {
            (0, result_num.map(|n| n as i32).unwrap_or(999))
        },
        FinishStatus::Racing => (1, -(e.laps_done as i32)),
        FinishStatus::Dnf => (2, -(e.laps_done as i32)),
    }
}
