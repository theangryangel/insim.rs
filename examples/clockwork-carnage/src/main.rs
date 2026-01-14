//! Clockwork carnage with generic, reusable scene system
//! Scenes can be shared across different game server projects

use std::{collections::HashMap, time::Duration};

use clap::Parser;
use insim::{
    builder::SpawnedHandle, core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colourify,
        track::Track,
    }, identifiers::ConnectionId, insim::{Btn, BtnStyle, ObjectInfo, TinyType, Uco}, WithRequestId
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneExt, SceneResult, wait_for_players::WaitForPlayers},
    time::Countdown,
    ui,
};
use tokio::{sync::broadcast, time::sleep};

mod chat;
mod cli;
mod marquee;
mod wait_for_admin_start;
mod setup_track;
mod topbar;

/// Clockwork Carnage event
// TODO: split up into Lobby, Rounds, Victory
#[derive(Clone)]
struct Clockwork {
    game: game::Game,
    presence: presence::Presence,
    chat: chat::Chat,
    insim: SpawnedHandle,

    rounds: usize,
    target: Duration,
    max_scorers: usize,
    min_players: usize,
}

impl Scene for Clockwork {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<()>, SceneError> {
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
                Ok(SceneResult::bail_with("Admin ended event"))
            }
            _ = self.game.wait_for_end() => {
                tracing::info!("Players voted to end");
                Ok(SceneResult::Continue(()))
            }
            _ = self.presence.wait_for_connection_count(|val| *val < self.min_players) => {
                tracing::info!("Lost players during event");
                Ok(SceneResult::bail_with("Clockwork lost players during event"))
            }
        }
    }
}

async fn wait_for_admin_end(
    chat: &mut broadcast::Receiver<(chat::ChatMsg, ConnectionId)>,
    presence: presence::Presence,
) -> Result<(), SceneError> {
    loop {
        match chat.recv().await {
            Ok((chat::ChatMsg::End, ucid)) => {
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
                return Err(SceneError::Custom {
                    scene: "wait_for_admin_end",
                    cause: Box::new(chat::ChatError::HandleLost),
                });
            },
        }
    }
}

struct ClockworkLobbyView {}
impl ui::View for ClockworkLobbyView {
    type GlobalProps = Duration;
    type ConnectionProps = ();
    type Message = ();

    fn mount(tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn render(
        &self,
        global_props: Self::GlobalProps,
        _connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        topbar::topbar(&format!("Warm up - {:?} remaining", global_props))
    }
}

#[derive(Debug, Clone, Default)]
struct ClockworkRoundGlobalProps {
    remaining: Duration,
    round: usize,
    rounds: usize,
}

struct ClockworkRoundView {}
impl ui::View for ClockworkRoundView {
    type GlobalProps = ClockworkRoundGlobalProps;
    type ConnectionProps = u32;
    type Message = ();

    fn mount(tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn render(
        &self,
        global_props: Self::GlobalProps,
        connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        topbar::topbar(&format!(
            "Round {}/{} - {:?} remaining", 
            global_props.round,
            global_props.rounds,
            global_props.remaining,
        )).with_child(
            ui::text(&format!("{} pts", connection_props).white(), BtnStyle::default().dark())
        )
    }
}

struct ClockworkInner {
    scores: HashMap<String, u32>,
    rounds: usize,
    max_scorers: usize,
    target: Duration,

    insim: SpawnedHandle,
    game: game::Game,
    presence: presence::Presence,
}

impl ClockworkInner {
    async fn run(&mut self) -> Result<(), SceneError> {
        self.lobby().await?;
        for round in 1..=self.rounds {
            self.round(round).await?;
        }
        self.announce_results().await?;
        Ok(())
    }

    async fn lobby(&self) -> Result<(), SceneError> {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 20);
        let ui = ui::attach::<ClockworkLobbyView>(self.insim.clone(), self.presence.clone(), Duration::ZERO);

        while let Some(_) = countdown.tick().await {
            let remaining = countdown.remaining_duration();
            ui.update_global_props(remaining);
        }

        Ok(())
    }

    async fn round(&mut self, round: usize) -> Result<(), SceneError> {
        let mut active_runs: HashMap<String, Duration> = HashMap::new();
        let mut round_scores: HashMap<String, Duration> = HashMap::new();
        let ui = ui::attach::<ClockworkRoundView>(
            self.insim.clone(), self.presence.clone(), ClockworkRoundGlobalProps::default()
        );

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
                            ui.update_global_props(ClockworkRoundGlobalProps { 
                                remaining: dur, 
                                round, 
                                rounds: self.rounds
                            });
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
                    self.handle_packet(packet.map_err(|_| SceneError::InsimHandleLost)?, &mut active_runs, &mut round_scores).await?;
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
    ) -> Result<(), SceneError> {
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

    async fn announce_results(&self) -> Result<(), SceneError> {
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
    chat: chat::Chat,
    presence: presence::Presence,
) -> Result<(), SceneError> {
    let mut chat = chat.subscribe();
    loop {
        match chat.recv().await {
            Ok((chat::ChatMsg::Quit, ucid)) => {
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
                return Err(SceneError::Custom {
                    scene: "wait_for_admin_quit",
                    cause: Box::new(chat::ChatError::HandleLost),
                });
            },
        }
    }
}

// host + 1 player
const MIN_PLAYERS: usize = 2;

#[tokio::main]
async fn main() -> Result<(), SceneError> {
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

    let presence = presence::spawn(insim.clone(), 32);
    let game = game::spawn(insim.clone(), 32);
    let chat = chat::spawn(insim.clone());

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;

    // Composible/reusable scenes snap together, "just like little lego"!
    let clockwork = WaitForPlayers {
        insim: insim.clone(),
        presence: presence.clone(),
        min_players: MIN_PLAYERS,
    }
    .then(wait_for_admin_start::WaitForAdminStart {
        insim: insim.clone(),
        presence: presence.clone(),
        chat: chat.clone(),
    })
    .then(
        setup_track::SetupTrack {
            insim: insim.clone(),
            presence: presence.clone(),
            min_players: MIN_PLAYERS,
            game: game.clone(),
            track: Track::Fe1x,
        }
        .with_timeout(Duration::from_secs(60)),
    )
    // example/test of using then_with. we don't actually need it in this case
    // but lets imagine that SetupTrack is actually VoteForTrack!
    .and_then({
        let game = game.clone();
        let presence = presence.clone();
        let chat = chat.clone();
        let insim = insim.clone();

        move |_| Clockwork {
            game: game.clone(),
            presence: presence.clone(),
            chat: chat.clone(),
            rounds: 5,
            max_scorers: 10,
            min_players: MIN_PLAYERS,
            target: Duration::from_secs(20),
            insim: insim.clone(),
        }
    })
    .loop_until_quit();

    tokio::select! {
        res = clockwork.run() => {
            tracing::info!("{:?}", res);
        },
        _ = wait_for_admin_quit(chat, presence) => {}
    }

    Ok(())
}
