//! Bare-insim race tracker example.
//!
//! Wires [`Presence`], [`Game`], and [`RaceTracker`] together in a plain async
//! packet loop - no kitcar. Prints a results table when the race ends.
//!
//! Run with:
//!     cargo run -p race-tracker -- --addr 127.0.0.1:29999

#![allow(missing_docs)]

use clap::Parser;
use insim::{WithRequestId, insim::TinyType};
use insim_extra::{
    game::Game,
    presence::Presence,
    race::{EntrantState, FinishStatus, RaceEvent, RaceTracker},
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

    // Request current connection, player, and session state from LFS.
    // LFS does not send these automatically on connect.
    conn.write(TinyType::Ncn.with_request_id(1)).await?;
    conn.write(TinyType::Npl.with_request_id(2)).await?;
    conn.write(TinyType::Sst.with_request_id(3)).await?;

    let presence = Presence::new();
    let game = Game::new();
    let race = RaceTracker::new();

    loop {
        let packet = conn.read().await?;

        for event in presence.apply_packet(&packet) {
            for e in race.apply_presence_event(&event) {
                tracing::debug!(?e, "race event");
            }
        }

        for event in game.apply_packet(&packet) {
            let is_race_ended = matches!(event, insim_extra::game::GameEvent::RaceEnded);
            for e in race.apply_game_event(&event) {
                tracing::debug!(?e, "race event");
            }
            if is_race_ended {
                tracing::info!("race ended");
                print_results(&race);
            }
        }

        let race_events = race.apply_packet(&packet);
        for e in &race_events {
            match e {
                RaceEvent::PersonalBest {
                    id,
                    lap,
                    time,
                    previous,
                    ..
                } => {
                    let driver = race
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
                    let driver = race
                        .entrant(*id)
                        .and_then(|e| e.drivers.last().map(|d| d.pname.clone()))
                        .unwrap_or_else(|| "?".to_string());
                    println!("[FL] {:<20}  {}  lap {}", driver, fmt_duration(*time), lap);
                },
                _ => tracing::debug!(?e, "race event"),
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

fn print_results(race: &RaceTracker) {
    let mut entrants = race.entrants();
    entrants.sort_by(|a, b| position_key(a).cmp(&position_key(b)));

    let fl_id = race.fastest_lap().map(|(id, _, _)| id);

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
                    let fl = if fl_id == Some(e.id) { " ★" } else { "" };
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

            ResultRow {
                pos,
                driver,
                laps: e.laps_done,
                best_lap,
                stops: e.pit_stops.len(),
                status,
            }
        })
        .collect();

    println!();
    println!("=== Race Results ===");
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
