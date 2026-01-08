//! Clockwork carnage. PoC to experiment with the "scene" wrapping idea.
//! 60s. 5 Rounds. Complete the point to point in as close to the target time as possible.

use std::{collections::HashMap, fmt::Debug, time::Duration};

use clap::Parser;
use insim::{
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        track::Track,
    }, identifiers::ConnectionId, insim::{ObjectInfo, RaceLaps, TinyType}, WithRequestId
};
use kitcar::time::countdown::Countdown;
use tokio::{sync::broadcast, time::{sleep, timeout}};

mod chat;
mod cli;
mod scene;

use scene::Scene;

#[derive(Debug, thiserror::Error)]
/// ClockworkCarnage Error
enum ClockworkCarnageError {
    #[error("Chat bus closed!")]
    ChatClosed,

    /// SendError
    #[error("Chat Broadcast Failed: {0}")]
    ChatBroadcastFailed(#[from] tokio::sync::broadcast::error::SendError<(chat::Chat, ConnectionId)>),

    /// RecvError
    #[error("Receive Failed: {0}")]
    RecvFailed(#[from] tokio::sync::broadcast::error::RecvError),

    /// Task failed
    #[error("Join failed: {0}")]
    JoinFailed(#[from] tokio::task::JoinError),

    /// Some sort of insim error
    #[error("insim error: {0}")]
    Insim(#[from] insim::Error),
}

#[derive(Debug)]
enum ClockworkCarnageExit {
    Continue,
    Exit,
}

#[derive(Debug, Default, Clone)]
struct ClockworkCarnage {
    min_players: usize,
}

impl ClockworkCarnage {
    async fn take_over(&self, mut ctx: Context) -> Result<(), ClockworkCarnageError> {
        let mut packets = ctx.insim.subscribe();
        loop {
            tokio::select! {
                packet = packets.recv() => {
                    // FIXME expect
                    if let insim::Packet::Ncn(ncn) = packet? {
                        tracing::info!("Waiting for players...");
                        let _ = ctx.insim.send_message("Waiting for players", ncn.ucid).await.expect("Unhandled error");
                    }
                },
                _ = ctx.presence.wait_for_connection_count(|val| *val >= self.min_players) => {
                    tracing::info!("Got minimum player count!");
                    break;
                }
            }
        }

        let _ = ctx
            .insim
            .send_message("Ready for admin start command...", ConnectionId::ALL)
            .await?;

        let mut chat = ctx.chat.subscribe();

        loop {
            match chat.recv().await {
                Ok((cmd, ucid)) => {
                    if_chain::if_chain! {
                        if matches!(cmd, chat::Chat::Start);
                        if let Some(conn_info) = ctx.presence.connection(&ucid).await;
                        if conn_info.admin;
                        then {
                            tracing::info!("Starting..");
                            break;
                        }
                    }
                },
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Chat commands lost due to lag");
                },
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(ClockworkCarnageError::ChatClosed);
                }
            }
        }

        Ok(())
    }
}

impl Scene<Context> for ClockworkCarnage {
    type Output = ClockworkCarnageExit;
    type Error = ClockworkCarnageError;

    async fn poll(&mut self, mut ctx: Context) -> Result<Self::Output, Self::Error> {
        self.take_over(ctx.clone()).await?;

        tokio::select! {
            res = timeout(Duration::from_secs(60), ctx.game.track_rotation(
                ctx.insim.clone(),
                Track::Fe1x,
                RaceLaps::Practice,
                0,
                None,
            )) => {
                if res.is_err() {
                    // timeout occured
                    tracing::error!("Timeout changing track and waiting for players");
                    return Ok(ClockworkCarnageExit::Continue);
                }
            },
            _ = ctx.presence.wait_for_connection_count(|val| *val < self.min_players) => {
                tracing::info!("less than minimum player count!");
                return Ok(ClockworkCarnageExit::Continue);
            }
        }

        tracing::info!("Starting lobby...");
        let mut event = Event::new(5, 10);
        let event_fut = event.poll(ctx.clone());
        tokio::pin!(event_fut);
        let mut chat = ctx.chat.subscribe();

        loop {
            tokio::select! {
                chat = chat.recv() => match chat {
                    Ok((cmd, ucid)) => {
                        if_chain::if_chain! {
                            if matches!(cmd, chat::Chat::End);
                            if let Some(conn_info) = ctx.presence.connection(&ucid).await;
                            if conn_info.admin;
                            then {
                                tracing::info!("Ending..");
                                break;
                            }
                        }
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        tracing::warn!("Chat commands lost due to lag");
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        return Err(ClockworkCarnageError::ChatClosed);
                    }
                },
                _ = ctx.game.wait_for_end() => {
                    // players all voted to end.. probably shouldn't be here, but eeh if we put it
                    // into wait_for_players we end up with a loop.. so this'll do for now.
                    break;
                },
                res = &mut event_fut => {
                    res?;
                    return Ok(ClockworkCarnageExit::Continue)
                }
            }
        }

        Ok(ClockworkCarnageExit::Exit)
    }
}

#[derive(Debug)]
struct Event {
    /// scored by LFS uname (for now)
    scores: HashMap<String, u32>,
    rounds: usize,
    max_scorers: usize,
}

impl Event {
    pub fn new(rounds: usize, max_scorers: usize) -> Self {
        Self {
            scores: HashMap::new(),
            rounds,
            max_scorers,
        }
    }
}

impl Event {
    async fn lobby(&mut self, ctx: Context) -> Result<(), ClockworkCarnageError> {
        tracing::info!("Lobby started 5 minute warm up");

        let mut countdown = Countdown::new(Duration::from_secs(1), 20);

        loop {
            match countdown.tick().await {
                Some(_) => {
                    let remaining_duration = countdown.remaining_duration();
                    tracing::info!("Waiting for lobby to complete! {:?}", remaining_duration);
                    ctx.insim
                        .send_message(
                            format!("Waiting for lobby to complete! {:?}", remaining_duration),
                            ConnectionId::ALL,
                        )
                        .await?;
                },
                None => {
                    break;
                },
            }
        }

        tracing::info!("Lobby done");
        Ok(())
    }

    async fn rounds(&mut self, mut ctx: Context) -> Result<(), ClockworkCarnageError> {
        for round in 1..=self.rounds {
            let mut active_runs: HashMap<String, Duration> = HashMap::new();
            let mut round_scores: HashMap<String, Duration> = HashMap::new();

            ctx.insim.send_command("/restart").await?;
            ctx.game.wait_for_racing().await;
            sleep(Duration::from_secs(1)).await;

            tracing::info!("Round {round}/{}", self.rounds);

            let mut countdown = Countdown::new(Duration::from_secs(1), 60);
            let mut packets = ctx.insim.subscribe();
            let target = Duration::from_secs(20);

            loop {
                tokio::select! {
                    remaining = countdown.tick() => match remaining {
                        Some(_) => {
                            let remaining_duration = countdown.remaining_duration();
                            ctx.insim.send_message(format!("{:?} remaining, round {}/{}", remaining_duration, round, self.rounds), ConnectionId::ALL).await?;

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
                                if_chain::if_chain! {
                                    if let Some(player) = ctx.presence.player(&uco.plid).await;
                                    if !player.ptype.is_ai();
                                    if let Some(conn) = ctx.presence.connection_by_player(&uco.plid).await;
                                    then {
                                        let _ = active_runs.insert(conn.uname, uco.time);
                                    }
                                }
                            },
                            ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind: InsimCheckpointKind::Finish, .. }) => {
                                if_chain::if_chain! {
                                    // FIXME: we need a way to fetch connection and player at the
                                    // same time
                                    if let Some(player) = ctx.presence.player(&uco.plid).await;
                                    if let Some(conn) = ctx.presence.connection_by_player(&uco.plid).await;
                                    if !player.ptype.is_ai();
                                    if let Some(start) = active_runs.remove(&conn.uname);
                                    let delta = uco.time.saturating_sub(start);
                                    if !delta.is_zero();
                                    then {
                                        let new_diff = target.abs_diff(delta);
                                        let best = round_scores
                                            .entry(conn.uname.clone())
                                            .and_modify(|existing| {
                                                if new_diff < *existing {
                                                    *existing = new_diff;
                                                }
                                            })
                                            .or_insert(new_diff);

                                        ctx.insim.send_command(format!("/spec {}", &conn.uname)).await?;
                                        ctx.insim.send_message(format!("You finished {:?} off the target..", new_diff), conn.ucid).await?;
                                        ctx.insim.send_message(format!("Your best time was {:?}", best), conn.ucid).await?;
                                        ctx.insim.send_message("You can rejoin to retry...", conn.ucid).await?;
                                    }
                                }
                            },

                            _ => {}
                        },
                        _ => {}
                    }
                }
            }

            // score round by LFS uname
            let mut ordered = round_scores
                .drain()
                .map(|(k, v)| (k, target.abs_diff(v)))
                .collect::<Vec<(String, Duration)>>();
            ordered.sort_by(|a, b| a.1.cmp(&b.1));
            let top: Vec<(String, u32, usize, Duration)> = ordered
                .into_iter()
                .take(self.max_scorers)
                .enumerate()
                .map(|(i, (uname, delta))| {
                    let points = (self.max_scorers - i) as u32;
                    // update global scores
                    let _ = self
                        .scores
                        .entry(uname.clone())
                        .and_modify(|existing| {
                            *existing = existing.saturating_add(points);
                        })
                        .or_insert(points);
                    (uname, points, i, delta)
                })
                .collect();

            // TODO: announce scorers this round
            tracing::info!("Round scorers: {:?}", top);
        }

        tracing::info!("Event scorers: {:?}", self.scores);

        Ok(())
    }
}

impl Scene<Context> for Event {
    type Output = ();
    type Error = ClockworkCarnageError;

    async fn poll(&mut self, ctx: Context) -> Result<Self::Output, Self::Error> {
        self.lobby(ctx.clone()).await?;
        self.rounds(ctx).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Context {
    pub insim: insim::builder::SpawnedHandle,
    pub presence: kitcar::presence::PresenceHandle,
    pub game: kitcar::game::GameHandle,
    pub chat: chat::ChatHandle,
}

#[tokio::main]
async fn main() -> Result<(), ClockworkCarnageError> {
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
    let game = kitcar::game::GameInfo::spawn(insim.clone(), 32);
    let (chat, chat_cancel_token) = chat::Chat::spawn(insim.clone(), presence.clone());

    let ctx = Context {
        insim: insim.clone(),
        presence: presence.clone(),
        game,
        chat,
    };

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;
    let mut cc = ClockworkCarnage{ min_players: 2 };

    loop {
        tokio::select! {
            _ = chat_cancel_token.cancelled() => {
                break;
            },
            res = cc.poll(ctx.clone()) => {
                if matches!(res?, ClockworkCarnageExit::Exit) {
                    break;
                }
            }
        }
    }

    Ok(())
}
