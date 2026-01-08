//! Clockwork carnage with generic, reusable scene system
//! Scenes can be shared across different game server projects

use std::{collections::HashMap, marker::PhantomData, time::Duration};

use clap::Parser;
use insim::{
    WithRequestId,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colourify,
        track::Track,
    },
    identifiers::ConnectionId,
    insim::{ObjectInfo, RaceLaps, TinyType, Uco},
};
use kitcar::time::countdown::Countdown;
use tokio::{sync::broadcast, time::sleep};

mod chat;
mod cli;
mod context;
mod scene;

use context::*;
use scene::{Scene, SceneExt, SceneResult};

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
struct WaitForPlayers<C> {
    min_players: usize,
    _phantom: PhantomData<C>,
}

impl<C> WaitForPlayers<C> {
    fn new(min_players: usize) -> Self {
        Self {
            min_players,
            _phantom: PhantomData,
        }
    }
}

impl<C> Scene for WaitForPlayers<C>
where
    C: HasInsim + HasPresence + Clone + Send + 'static,
{
    type Context = C;
    type Output = ();
    type Error = Error;

    async fn run(self, ctx: C) -> Result<SceneResult<()>, Error> {
        tracing::info!("Waiting for {} players...", self.min_players);
        let mut packets = ctx.insim().subscribe();
        let mut presence = ctx.presence();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    if let insim::Packet::Ncn(ncn) = packet? {
                        ctx.insim().send_message("Waiting for players", ncn.ucid).await?;
                    }
                }
                _ = presence.wait_for_connection_count(|val| *val >= self.min_players) => {
                    tracing::info!("Got minimum player count!");
                    return Ok(SceneResult::Continue(()));
                }
            }
        }
    }
}

/// Wait for admin to start
#[derive(Clone)]
struct WaitForAdminStart<C> {
    _phantom: PhantomData<C>,
}

impl<C> WaitForAdminStart<C> {
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<C> Scene for WaitForAdminStart<C>
where
    C: HasInsim + HasPresence + HasChat + Clone + Send + 'static,
{
    type Context = C;
    type Output = ();
    type Error = Error;

    async fn run(self, ctx: C) -> Result<SceneResult<()>, Error> {
        ctx.insim()
            .send_message("Ready for admin !start command", ConnectionId::ALL)
            .await?;
        let mut chat = ctx.chat().subscribe();

        loop {
            match chat.recv().await {
                Ok((chat::Chat::Start, ucid)) => {
                    if let Some(conn) = ctx.presence().connection(&ucid).await {
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
struct SetupTrack<C> {
    min_players: usize,
    track: Track,
    _phantom: PhantomData<C>,
}

impl<C> SetupTrack<C> {
    fn new(min_players: usize, track: Track) -> Self {
        Self {
            min_players,
            track,
            _phantom: PhantomData,
        }
    }
}

impl<C> Scene for SetupTrack<C>
where
    C: HasInsim + HasGame + HasPresence + Clone + Send + 'static,
{
    type Context = C;
    type Output = ();
    type Error = Error;

    async fn run(self, ctx: C) -> Result<SceneResult<()>, Error> {
        let mut game = ctx.game();
        let mut presence = ctx.presence();
        tokio::select! {
            _ = game.track_rotation(
                ctx.insim().clone(),
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
            _ = presence.wait_for_connection_count(|val| *val < self.min_players) => {
                tracing::info!("Lost players during track setup");
                Ok(SceneResult::Bail)
            }
        }
    }
}

/// Clockwork Carnage event
#[derive(Clone)]
struct Clockwork<C> {
    rounds: usize,
    max_scorers: usize,
    min_players: usize,
    _phantom: PhantomData<C>,
}

impl<C> Clockwork<C> {
    fn new(rounds: usize, max_scorers: usize, min_players: usize) -> Self {
        Self {
            rounds,
            max_scorers,
            min_players,
            _phantom: PhantomData,
        }
    }
}

impl<C> Scene for Clockwork<C>
where
    C: HasInsim + HasGame + HasPresence + HasChat + Clone + Send + 'static,
{
    type Context = C;
    type Output = ();
    type Error = Error;

    async fn run(self, ctx: C) -> Result<SceneResult<()>, Error> {
        let mut event = ClockworkInner::new(self.rounds, self.max_scorers);
        let mut chat = ctx.chat().subscribe();
        let mut presence = ctx.presence();
        let mut game = ctx.game();

        tokio::select! {
            res = event.run(&ctx) => {
                res?;
                Ok(SceneResult::Continue(()))
            }
            _ = wait_for_admin_end(&mut chat, &ctx) => {
                tracing::info!("Admin ended event");
                Ok(SceneResult::Bail)
            }
            _ = game.wait_for_end() => {
                tracing::info!("Players voted to end");
                Ok(SceneResult::Continue(()))
            }
            _ = presence.wait_for_connection_count(|val| *val < self.min_players) => {
                tracing::info!("Lost players during event");
                Ok(SceneResult::Bail)
            }
        }
    }
}

async fn wait_for_admin_end<C>(
    chat: &mut broadcast::Receiver<(chat::Chat, ConnectionId)>,
    ctx: &C,
) -> Result<(), Error>
where
    C: HasPresence,
{
    loop {
        match chat.recv().await {
            Ok((chat::Chat::End, ucid)) => {
                if let Some(conn) = ctx.presence().connection(&ucid).await {
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
}

impl ClockworkInner {
    fn new(rounds: usize, max_scorers: usize) -> Self {
        Self {
            scores: HashMap::new(),
            rounds,
            max_scorers,
        }
    }

    async fn run<C>(&mut self, ctx: &C) -> Result<(), Error>
    where
        C: HasInsim + HasGame + HasPresence,
    {
        self.lobby(ctx).await?;
        for round in 1..=self.rounds {
            self.round(ctx, round).await?;
        }
        self.announce_results(ctx).await?;
        Ok(())
    }

    async fn lobby<C>(&self, ctx: &C) -> Result<(), Error>
    where
        C: HasInsim,
    {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 20);

        while let Some(_) = countdown.tick().await {
            let remaining = countdown.remaining_duration();
            ctx.insim()
                .send_message(format!("Warm up: {:?}", remaining), ConnectionId::ALL)
                .await?;
        }

        Ok(())
    }

    async fn round<C>(&mut self, ctx: &C, round: usize) -> Result<(), Error>
    where
        C: HasInsim + HasGame + HasPresence,
    {
        let mut active_runs: HashMap<String, Duration> = HashMap::new();
        let mut round_scores: HashMap<String, Duration> = HashMap::new();

        ctx.insim().send_command("/restart").await?;
        sleep(Duration::from_secs(5)).await;
        ctx.game().wait_for_racing().await;
        sleep(Duration::from_secs(1)).await;

        tracing::info!("Round {}/{}", round, self.rounds);

        let mut countdown = Countdown::new(Duration::from_secs(1), 60);
        let mut packets = ctx.insim().subscribe();
        let target = Duration::from_secs(20);

        loop {
            tokio::select! {
                remaining = countdown.tick() => {
                    match remaining {
                        Some(_) => {
                            let dur = countdown.remaining_duration();
                            ctx.insim()
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
                    self.handle_packet(packet?, ctx, &mut active_runs, &mut round_scores, target).await?;
                }
            }
        }

        self.score_round(round_scores);
        Ok(())
    }

    async fn handle_packet<C>(
        &self,
        packet: insim::Packet,
        ctx: &C,
        active_runs: &mut HashMap<String, Duration>,
        round_scores: &mut HashMap<String, Duration>,
        target: Duration,
    ) -> Result<(), Error>
    where
        C: HasInsim + HasPresence,
    {
        match packet {
            insim::Packet::Ncn(ncn) => {
                ctx.insim()
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
                    if let Some(player) = ctx.presence().player(&plid).await;
                    if !player.ptype.is_ai();
                    if let Some(conn) = ctx.presence().connection_by_player(&plid).await;
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
                    if let Some(player) = ctx.presence().player(&plid).await;
                    if !player.ptype.is_ai();
                    if let Some(conn) = ctx.presence().connection_by_player(&plid).await;
                    if let Some(start) = active_runs.remove(&conn.uname);
                    then {
                        let delta = time.saturating_sub(start);
                        let diff = target.abs_diff(delta);
                        let best = round_scores
                            .entry(conn.uname.clone())
                            .and_modify(|e| {
                                if diff < *e {
                                    *e = diff;
                                }
                            })
                            .or_insert(diff);

                        ctx.insim()
                            .send_command(format!("/spec {}", conn.uname))
                            .await?;
                        ctx.insim()
                            .send_message(format!("Off by: {:?}", diff).yellow(), conn.ucid)
                            .await?;
                        ctx.insim()
                            .send_message(format!("Best: {:?}", best).light_green(), conn.ucid)
                            .await?;
                        ctx.insim()
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
    }

    async fn announce_results<C>(&self, ctx: &C) -> Result<(), Error>
    where
        C: HasInsim,
    {
        let mut standings: Vec<_> = self.scores.iter().collect();
        standings.sort_by(|a, b| b.1.cmp(a.1));

        ctx.insim()
            .send_message("Final standings".yellow(), ConnectionId::ALL)
            .await?;
        for (i, (name, score)) in standings.iter().take(10).enumerate() {
            ctx.insim()
                .send_message(
                    format!("{}. {} - {} pts", i + 1, name, score).yellow(),
                    ConnectionId::ALL,
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct ClockworkContext {
    insim: insim::builder::SpawnedHandle,
    presence: kitcar::presence::PresenceHandle,
    game: kitcar::game::GameHandle,
    chat: chat::ChatHandle,
}

impl HasInsim for ClockworkContext {
    fn insim(&self) -> insim::builder::SpawnedHandle {
        self.insim.clone()
    }
}

impl HasPresence for ClockworkContext {
    fn presence(&self) -> kitcar::presence::PresenceHandle {
        self.presence.clone()
    }
}

impl HasGame for ClockworkContext {
    fn game(&self) -> kitcar::game::GameHandle {
        self.game.clone()
    }
}

impl HasChat for ClockworkContext {
    fn chat(&self) -> chat::ChatHandle {
        self.chat.clone()
    }
}

async fn wait_for_admin_quit(chat: chat::ChatHandle, ctx: ClockworkContext) -> Result<(), Error> {
    let mut chat = chat.subscribe();
    loop {
        match chat.recv().await {
            Ok((chat::Chat::Quit, ucid)) => {
                if let Some(conn) = ctx.presence.connection(&ucid).await {
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

    let ctx = ClockworkContext {
        insim: insim.clone(),
        presence,
        game,
        chat: chat.clone(),
    };

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;

    // Composible/reusable scenes snap together, "just like little lego"!
    let game = WaitForPlayers::new(MIN_PLAYERS)
        .then(WaitForAdminStart::new())
        .then(SetupTrack::new(MIN_PLAYERS, Track::Fe1x))
        .then(Clockwork::new(5, 10, 2))
        .repeat();

    tokio::select! {
        res = game.run(ctx.clone()) => {
            tracing::info!("{:?}", res);
        },
        _ = wait_for_admin_quit(chat, ctx) => {}
    }

    Ok(())
}
