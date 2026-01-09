//! Clockwork carnage with generic, reusable scene system
//! Scenes can be shared across different game server projects

use std::{collections::HashMap, time::Duration};

use clap::Parser;
use insim::{
    WithRequestId,
    builder::SpawnedHandle,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colourify,
        track::Track,
    },
    identifiers::ConnectionId,
    insim::{ObjectInfo, RaceLaps, TinyType, Uco},
};
use kitcar::{game::GameHandle, presence::PresenceHandle, time::countdown::Countdown};
use tokio::{sync::broadcast, time::sleep};

mod chat;
mod cli;
mod scene;

use scene::{Scene, SceneExt, SceneResult};

use crate::chat::ChatHandle;

// FIXME: sort out these errors so they're more useful
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Chat bus closed")]
    ChatLost,
    #[error("Chat broadcast failed: {0}")]
    ChatBroadcastFailed(
        #[from] tokio::sync::broadcast::error::SendError<(chat::Chat, ConnectionId)>,
    ),
    #[error("Receive failed: {0}")]
    RecvFailed(#[from] tokio::sync::broadcast::error::RecvError),
    #[error("Join failed: {0}")]
    JoinFailed(#[from] tokio::task::JoinError),
    #[error("insim error: {0}")]
    Insim(#[from] insim::Error),
}

/// Wait for minimum players to connect
#[derive(Clone)]
struct WaitForPlayers {
    insim: SpawnedHandle,
    presence: PresenceHandle,
    min_players: usize,
}

impl Scene for WaitForPlayers {
    type Output = ();
    type Error = Error;  // FIXME: WaitForPlayersError

    async fn run(mut self) -> Result<SceneResult<()>, Error> {
        tracing::info!("Waiting for {} players...", self.min_players);
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    if let insim::Packet::Ncn(ncn) = packet? {
                        self.insim.send_message("Waiting for players", ncn.ucid).await?;
                    }
                }
                _ = self.presence.wait_for_connection_count(|val| *val >= self.min_players) => {
                    tracing::info!("Got minimum player count!");
                    return Ok(SceneResult::Continue(()));
                }
            }
        }
    }
}

/// Wait for admin to start
#[derive(Clone)]
struct WaitForAdminStart {
    insim: SpawnedHandle,
    presence: PresenceHandle,
    chat: ChatHandle,
}

impl Scene for WaitForAdminStart {
    type Output = ();
    type Error = Error;  // FIXME: WaitForAdminStartError

    async fn run(self) -> Result<SceneResult<()>, Error> {
        self.insim
            .send_message("Ready for admin !start command", ConnectionId::ALL)
            .await?;
        let mut chat = self.chat.subscribe();

        loop {
            match chat.recv().await {
                Ok((chat::Chat::Start, ucid)) => {
                    if let Some(conn) = self.presence.connection(&ucid).await {
                        if conn.admin {
                            tracing::info!("Admin started game");
                            return Ok(SceneResult::Continue(()));
                        }
                    }
                },
                Ok(_) => {},
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Chat commands lost due to lag");
                },
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(Error::ChatLost);
                },
            }
        }
    }
}

/// Setup track
#[derive(Clone)]
struct SetupTrack {
    game: GameHandle,
    presence: PresenceHandle,
    insim: SpawnedHandle,
    min_players: usize,
    track: Track,
}

impl Scene for SetupTrack {
    type Output = ();
    type Error = Error; // FIXME: SetupTrackError

    async fn run(mut self) -> Result<SceneResult<()>, Error> {
        tokio::select! {
            _ = self.game.track_rotation(
                self.insim.clone(),
                self.track,
                RaceLaps::Practice,
                0,
                None,
            ) => {
                Ok(SceneResult::Continue(()))
            },
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                tracing::error!("Track change timeout");
                Ok(SceneResult::Bail)
            },
            _ = self.presence.wait_for_connection_count(|val| *val < self.min_players) => {
                tracing::info!("Lost players during track setup");
                Ok(SceneResult::Bail)
            }
        }
    }
}

/// Clockwork Carnage event
#[derive(Clone)]
struct Clockwork {
    game: GameHandle,
    presence: PresenceHandle,
    chat: ChatHandle,
    insim: SpawnedHandle,

    rounds: usize,
    target: Duration,
    max_scorers: usize,
    min_players: usize,
}

impl Scene for Clockwork {
    type Output = ();
    type Error = Error;

    async fn run(mut self) -> Result<SceneResult<()>, Error> {
        let mut event = ClockworkInner {
            max_scorers: self.max_scorers,
            scores: Default::default(),
            rounds: self.rounds,
            target: self.target,
            insim: self.insim.clone(),
            game: self.game.clone(),
            presence: self.presence.clone(),
        };
        let mut chat = self.chat.subscribe();

        tokio::select! {
            res = event.run() => {
                res?;
                Ok(SceneResult::Continue(()))
            }
            _ = wait_for_admin_end(&mut chat, self.presence.clone()) => {
                tracing::info!("Admin ended event");
                Ok(SceneResult::Bail)
            }
            _ = self.game.wait_for_end() => {
                tracing::info!("Players voted to end");
                Ok(SceneResult::Continue(()))
            }
            _ = self.presence.wait_for_connection_count(|val| *val < self.min_players) => {
                tracing::info!("Lost players during event");
                Ok(SceneResult::Bail)
            }
        }
    }
}

async fn wait_for_admin_end(
    chat: &mut broadcast::Receiver<(chat::Chat, ConnectionId)>,
    presence: PresenceHandle,
) -> Result<(), Error> {
    loop {
        match chat.recv().await {
            Ok((chat::Chat::End, ucid)) => {
                if let Some(conn) = presence.connection(&ucid).await {
                    if conn.admin {
                        return Ok(());
                    }
                }
            },
            Ok(_) => {},
            Err(broadcast::error::RecvError::Lagged(_)) => {
                tracing::warn!("Chat commands lost due to lag");
            },
            Err(broadcast::error::RecvError::Closed) => {
                return Err(Error::ChatLost);
            },
        }
    }
}

struct ClockworkInner {
    scores: HashMap<String, u32>,
    rounds: usize,
    max_scorers: usize,
    target: Duration,

    insim: SpawnedHandle,
    game: GameHandle,
    presence: PresenceHandle,
}

impl ClockworkInner {
    async fn run(&mut self) -> Result<(), Error> {
        self.lobby().await?;
        for round in 1..=self.rounds {
            self.round(round).await?;
        }
        self.announce_results().await?;
        Ok(())
    }

    async fn lobby(&self) -> Result<(), Error> {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 20);

        while let Some(_) = countdown.tick().await {
            let remaining = countdown.remaining_duration();
            self.insim
                .send_message(format!("Warm up: {:?}", remaining), ConnectionId::ALL)
                .await?;
        }

        Ok(())
    }

    async fn round(&mut self, round: usize) -> Result<(), Error> {
        let mut active_runs: HashMap<String, Duration> = HashMap::new();
        let mut round_scores: HashMap<String, Duration> = HashMap::new();

        self.insim.send_command("/restart").await?;
        sleep(Duration::from_secs(5)).await;
        self.game.wait_for_racing().await;
        sleep(Duration::from_secs(1)).await;

        tracing::info!("Round {}/{}", round, self.rounds);

        let mut countdown = Countdown::new(Duration::from_secs(1), 60);
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                remaining = countdown.tick() => {
                    match remaining {
                        Some(_) => {
                            let dur = countdown.remaining_duration();
                            self.insim
                                .send_message(
                                    format!("{:?} left, round {}/{}", dur, round, self.rounds),
                                    ConnectionId::ALL
                                )
                                .await?;
                        }
                        None => break,
                    }
                },
                packet = packets.recv() => {
                    self.handle_packet(packet?, &mut active_runs, &mut round_scores).await?;
                }
            }
        }

        self.score_round(round_scores);
        Ok(())
    }

    async fn handle_packet(
        &self,
        packet: insim::Packet,
        active_runs: &mut HashMap<String, Duration>,
        round_scores: &mut HashMap<String, Duration>,
    ) -> Result<(), Error> {
        match packet {
            insim::Packet::Ncn(ncn) => {
                self.insim
                    .send_message("Welcome! Game in progress", ncn.ucid)
                    .await?;
            },
            insim::Packet::Uco(Uco {
                info:
                    ObjectInfo::InsimCheckpoint(InsimCheckpoint {
                        kind: InsimCheckpointKind::Checkpoint1,
                        ..
                    }),
                plid,
                time,
                ..
            }) => {
                if_chain::if_chain! {
                    if let Some(player) = self.presence.player(&plid).await;
                    if !player.ptype.is_ai();
                    if let Some(conn) = self.presence.connection_by_player(&plid).await;
                    then {
                        let _ = active_runs.insert(conn.uname, time);
                    }
                }
            },
            insim::Packet::Uco(Uco {
                info:
                    ObjectInfo::InsimCheckpoint(InsimCheckpoint {
                        kind: InsimCheckpointKind::Finish,
                        ..
                    }),
                plid,
                time,
                ..
            }) => {
                if_chain::if_chain! {
                    if let Some(player) = self.presence.player(&plid).await;
                    if !player.ptype.is_ai();
                    if let Some(conn) = self.presence.connection_by_player(&plid).await;
                    if let Some(start) = active_runs.remove(&conn.uname);
                    then {
                        let delta = time.saturating_sub(start);
                        let diff = self.target.abs_diff(delta);
                        let best = round_scores
                            .entry(conn.uname.clone())
                            .and_modify(|e| {
                                if diff < *e {
                                    *e = diff;
                                }
                            })
                            .or_insert(diff);

                        self.insim
                            .send_command(format!("/spec {}", conn.uname))
                            .await?;
                        self.insim
                            .send_message(format!("Off by: {:?}", diff).yellow(), conn.ucid)
                            .await?;
                        self.insim
                            .send_message(format!("Best: {:?}", best).light_green(), conn.ucid)
                            .await?;
                        self.insim
                            .send_message("Rejoin to retry".yellow(), conn.ucid)
                            .await?;
                    }
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn score_round(&mut self, round_scores: HashMap<String, Duration>) {
        let mut ordered: Vec<_> = round_scores.into_iter().collect();
        ordered.sort_by_key(|(_, v)| *v);

        for (i, (uname, _)) in ordered.into_iter().take(self.max_scorers).enumerate() {
            let points = (self.max_scorers - i) as u32;
            let _ = self
                .scores
                .entry(uname)
                .and_modify(|e| *e = e.saturating_add(points))
                .or_insert(points);
        }

        tracing::info!("{:?}", self.scores);
    }

    async fn announce_results(&self) -> Result<(), Error> {
        let mut standings: Vec<_> = self.scores.iter().collect();
        standings.sort_by(|a, b| b.1.cmp(a.1));

        self.insim
            .send_message("Final standings".yellow(), ConnectionId::ALL)
            .await?;
        for (i, (name, score)) in standings.iter().take(10).enumerate() {
            self.insim
                .send_message(
                    format!("{}. {} - {} pts", i + 1, name, score).yellow(),
                    ConnectionId::ALL,
                )
                .await?;
        }
        Ok(())
    }
}

async fn wait_for_admin_quit(
    chat: chat::ChatHandle,
    presence: PresenceHandle,
) -> Result<(), Error> {
    let mut chat = chat.subscribe();
    loop {
        match chat.recv().await {
            Ok((chat::Chat::Quit, ucid)) => {
                if let Some(conn) = presence.connection(&ucid).await {
                    if conn.admin {
                        tracing::info!("Admin {} requested quit", conn.uname);
                        return Ok(());
                    }
                }
            },
            Ok(_) => {},
            Err(broadcast::error::RecvError::Lagged(_)) => {
                tracing::warn!("Chat commands lost due to lag");
            },
            Err(broadcast::error::RecvError::Closed) => {
                return Err(Error::ChatLost);
            },
        }
    }
}

// host + 1 player
const MIN_PLAYERS: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = cli::Args::parse();

    let (insim, _) = insim::tcp(args.addr.clone())
        .isi_admin_password(args.password.clone())
        .isi_iname("clockwork".to_owned())
        .isi_prefix('!')
        .spawn(100)
        .await?;

    tracing::info!("Starting clockwork carnage");

    let presence = kitcar::presence::Presence::spawn(insim.clone(), 32);
    let game = kitcar::game::GameInfo::spawn(insim.clone(), 32);
    let chat = chat::Chat::spawn(insim.clone());

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;

    // Composible/reusable scenes snap together, "just like little lego"!
    let clockwork = WaitForPlayers {
        insim: insim.clone(),
        presence: presence.clone(),
        min_players: MIN_PLAYERS,
    }
    .then(WaitForAdminStart {
        insim: insim.clone(),
        presence: presence.clone(),
        chat: chat.clone(),
    })
    .then(SetupTrack {
        insim: insim.clone(),
        presence: presence.clone(),
        min_players: MIN_PLAYERS,
        game: game.clone(),
        track: Track::Fe1x,
    })
    // example/test of using then_with. we don't actually need it in this case
    // but lets imagine that SetupTrack is actually VoteForTrack!
    .then_with({
        let game = game.clone();
        let presence = presence.clone();
        let chat = chat.clone();
        let insim = insim.clone();

        move |_| { 
            Clockwork {
                game: game.clone(),
                presence: presence.clone(),
                chat: chat.clone(),
                rounds: 5,
                max_scorers: 10,
                min_players: MIN_PLAYERS,
                target: Duration::from_secs(20),
                insim: insim.clone(),
            }
        }
    })
    .repeat();

    tokio::select! {
        res = clockwork.run() => {
            tracing::info!("{:?}", res);
        },
        _ = wait_for_admin_quit(chat, presence) => {}
    }

    Ok(())
}
