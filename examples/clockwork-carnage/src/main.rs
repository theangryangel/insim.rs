//! Clockwork carnage. PoC to experiment with the "scene" wrapping idea. go look at
//! twenty_second_league for where this is "going".

use std::{collections::HashMap, fmt::Debug, time::Duration};

use clap::Parser;
use insim::{core::{object::insim::{InsimCheckpoint, InsimCheckpointKind}, track::Track}, identifiers::PlayerId, insim::{ObjectInfo, RaceLaps, TinyType}, WithRequestId};
use kitcar::{chat::Parse, game::track_rotation::TrackRotation, time::countdown::Countdown};
use tokio::time::sleep;

mod cli;
mod wait_for_players;
mod scene;
mod chat;

use scene::Scene;

#[derive(Debug, Default, Clone)]
struct ClockworkCarnage;

impl Scene<Context> for ClockworkCarnage {
    type Output = anyhow::Result<()>;

    async fn poll(&mut self, ctx: Context) -> Self::Output {
        let mut packets = ctx.insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => match packet? {
                    insim::Packet::Mso(mso) => {
                        if_chain::if_chain! {
                            if let Ok(chat::Chat::Start) = chat::Chat::parse(mso.msg_from_textstart());
                            if let Some(conn_info) = ctx.presence.connection(&mso.ucid).await;
                            if conn_info.admin;
                            then {
                                break;
                            }
                        }
                    },
                    _ => {},
                }
            }
        }

        let mut rotation = TrackRotation::request(
            ctx.game.clone(), ctx.insim.clone(), Track::Bl1, None, RaceLaps::Practice, None
        );
        rotation.poll().await??;

        tracing::info!("Starting lobby...");
        Lobby.poll(ctx).await?;
        Ok(())
    }
}

struct Lobby;
impl Scene<Context> for Lobby {
    type Output = anyhow::Result<()>;

    async fn poll(&mut self, ctx: Context) -> Self::Output {
        tracing::info!("Lobby started 5 minute warm up");

        let mut countdown = Countdown::new(
            Duration::from_secs(1),
            300,
        );

        loop {
            match countdown.tick().await {
                Some(_) => {
                    let remaining_duration = countdown.remaining_duration();
                    tracing::info!("Waiting for lobby to complete! {:?}", remaining_duration);
                },
                None => {
                    break;
                }
            }
        }

        tracing::info!("Lobby done");

        Event.poll(ctx.clone()).await?;
        Ok(())
    }
}

struct Event;
impl Scene<Context> for Event {
    type Output = anyhow::Result<()>;

    async fn poll(&mut self, ctx: Context) -> Self::Output {
        for round in 1..=5 {
            let mut active_runs: HashMap<PlayerId, Duration> = HashMap::new();
            let mut round_scores: HashMap<PlayerId, Duration> = HashMap::new();

            ctx.insim.send_command("/restart").await?;
            ctx.game.wait_for_racing().await;
            sleep(Duration::from_secs(1)).await;

            tracing::info!("Round {round}/5");

            let mut countdown = Countdown::new(Duration::from_secs(1), 60);
            let mut packets = ctx.insim.subscribe();
            let target = Duration::from_secs(30);

            loop {
                tokio::select! {
                    remaining = countdown.tick() => match remaining {
                        Some(_) => {
                            let remaining_duration = countdown.remaining_duration();
                            tracing::debug!("{:?} remaining!", &remaining_duration);
                        },
                        None => {
                            break;
                        }
                    },
                    packet = packets.recv() => match packet? {
                        insim::Packet::Ncn(ncn) => {
                            ctx.insim
                                .send_message(
                                    "Welcome to the Clockwork Carnage! Game in currently in progress!",
                                    ncn.ucid,
                                )
                                .await?;
                        },
                        insim::Packet::Uco(uco) => match uco.info {
                            ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind: InsimCheckpointKind::Checkpoint1, .. }) => {
                                let _ = active_runs.insert(uco.plid, uco.time);
                            },
                            ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind: InsimCheckpointKind::Finish, .. }) => {
                                if_chain::if_chain! {
                                    if let Some(start) = active_runs.remove(&uco.plid);
                                    let delta = uco.time.saturating_sub(start);
                                    if !delta.is_zero();
                                    if let Some(_conn) = ctx.presence.connection_by_player(&uco.plid).await;
                                    then {
                                        let new_diff = target.abs_diff(delta);
                                        let _ = round_scores
                                            .entry(uco.plid)
                                            .and_modify(|existing| {
                                                let existing_diff = target.abs_diff(*existing);
                                                if new_diff < existing_diff {
                                                    *existing = delta;
                                                }
                                            })
                                            .or_insert(delta);
                                    }
                                }
                            },

                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            let max_scorers = 10;

            // score round
            let mut ordered = round_scores
                .drain()
                .map(|(k, v)| (k, target.abs_diff(v)))
                .collect::<Vec<(PlayerId, Duration)>>();
            ordered.sort_by(|a, b| a.1.cmp(&b.1));
            let top: Vec<(PlayerId, i32, usize, Duration)> = ordered
                .into_iter()
                .take(max_scorers)
                .enumerate()
                .map(|(i, (uname, delta))| {
                    let points = (max_scorers - i) as i32;
                    (uname, points, i, delta)
                })
                .collect();

            // TODO: announce scores
            tracing::info!("Scorers: {:?}", top);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Context {
    pub insim: insim::builder::SpawnedHandle,
    pub presence: kitcar::presence::PresenceHandle,
    pub game: kitcar::game::GameHandle,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = cli::Args::parse();

    let (insim, _insim_join_handle) = insim::tcp(args.addr.clone())
        .isi_admin_password(args.password.clone())
        .isi_iname("clockwork".to_owned())
        .isi_prefix('!')
        .spawn(100)
        .await?;

    tracing::info!("Starting clockwork-carnage");
    let presence = kitcar::presence::Presence::spawn(insim.clone(), 32); 

    let ctx = Context {
        insim: insim.clone(),
        presence: presence.clone(),
        game: kitcar::game::GameInfo::spawn(insim.clone(), 32),
    };

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;

    wait_for_players::WaitForPlayers::new(
        insim,
        presence,
        30
    ).poll(ClockworkCarnage, ctx).await?;

    Ok(())
}
