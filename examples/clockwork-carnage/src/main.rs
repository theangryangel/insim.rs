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

    insim::{BtnStyle, ObjectInfo, TinyType, Uco},
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneExt, SceneResult, wait_for_players::WaitForPlayers},
    time::Countdown,
    ui,
};
use tokio::time::sleep;

mod chat;
mod cli;
mod leaderboard;
mod marquee;
mod scoreboard;
mod setup_track;
mod topbar;
mod wait_for_admin_start;
mod wait_for_admin_cmd;

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
            round_best: Default::default(),
            active_runs: Default::default(),
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
            _ = wait_for_admin_cmd::wait_for_admin_cmd(&mut chat, self.presence.clone(), |msg| matches!(msg, chat::ChatMsg::End)) => {
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

struct ClockworkLobbyView {}
impl ui::View for ClockworkLobbyView {
    type GlobalProps = Duration;
    type ConnectionProps = ();
    type Message = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
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

use scoreboard::EnrichedLeaderboard;

#[derive(Debug, Clone, Default)]
struct ClockworkRoundGlobalProps {
    remaining: Duration,
    round: usize,
    rounds: usize,
    leaderboard: EnrichedLeaderboard,
}

#[derive(Debug, Clone, Default)]
struct ClockworkRoundConnectionProps {
    uname: String,
    in_progress: bool,
    round_best: Option<Duration>,
}

struct ClockworkRoundView {}
impl ui::View for ClockworkRoundView {
    type GlobalProps = ClockworkRoundGlobalProps;
    type ConnectionProps = ClockworkRoundConnectionProps;
    type Message = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn render(
        &self,
        global_props: Self::GlobalProps,
        connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        let status = if connection_props.in_progress {
            "In progress".light_green()
        } else {
            match connection_props.round_best {
                Some(d) => format!("Best: {:.2?}", d).white(),
                None => "Waiting for start".red(),
            }
        };

        let players = scoreboard::scoreboard(&global_props.leaderboard, &connection_props.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar::topbar(&format!(
                    "Round {}/{} - {:?} remaining",
                    global_props.round, global_props.rounds, global_props.remaining,
                ))
                .with_child(ui::text(&status, BtnStyle::default().dark()).w(15.).h(5.)),
            )
            .with_child(
                ui::container()
                    .flex()
                    .mt(20.)
                    .w(200.)
                    .flex_col()
                    .items_start()
                    .with_child(
                        ui::text("Scores!".yellow(), BtnStyle::default().dark())
                            .w(35.)
                            .h(5.),
                    )
                    .with_children(players),
            )
    }
}

#[derive(Debug, Clone, Default)]
struct ClockworkVictoryGlobalProps {
    standings: EnrichedLeaderboard,
    countdown: Duration,
}

#[derive(Debug, Clone, Default)]
struct ClockworkVictoryConnectionProps {
    uname: String,
}

struct ClockworkVictoryView {}
impl ui::View for ClockworkVictoryView {
    type GlobalProps = ClockworkVictoryGlobalProps;
    type ConnectionProps = ClockworkVictoryConnectionProps;
    type Message = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn render(
        &self,
        global_props: Self::GlobalProps,
        connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        let players = scoreboard::scoreboard(&global_props.standings, &connection_props.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(topbar::topbar(&format!(
                "Final Standings! - Next game in {:?}",
                global_props.countdown
            )))
            .with_child(
                ui::container()
                    .flex()
                    .mt(20.)
                    .w(200.)
                    .flex_col()
                    .items_start()
                    .with_child(
                        ui::text("Victory!".yellow(), BtnStyle::default().dark())
                            .w(35.)
                            .h(5.),
                    )
                    .with_children(players),
            )
    }
}

struct ClockworkInner {
    scores: leaderboard::Leaderboard,
    rounds: usize,
    max_scorers: usize,
    target: Duration,

    round_best: HashMap<String, Duration>,
    active_runs: HashMap<String, Duration>,

    insim: SpawnedHandle,
    game: game::Game,
    presence: presence::Presence,
}

impl ClockworkInner {
    async fn run(&mut self) -> Result<(), SceneError> {
        self.lobby().await?;

        let ui = ui::attach::<ClockworkRoundView>(
            self.insim.clone(),
            self.presence.clone(),
            ClockworkRoundGlobalProps::default(),
        );
        for round in 1..=self.rounds {
            self.broadcast_rankings(&ui).await;
            self.round(round, &ui).await?;
        }
        drop(ui);

        self.announce_results().await?;
        Ok(())
    }

    async fn broadcast_rankings(&mut self, ui: &ui::Ui<ClockworkRoundView>) {
        if let Some(connections) = self.presence.connections().await {
            for conn in connections {
                let props = self.connection_props(&conn.uname);
                ui.update_connection_props(conn.ucid, props).await;
            }
        }
    }

    async fn enriched_leaderboard(&self) -> EnrichedLeaderboard {
        let ranking = self.scores.ranking();
        let names = self.presence.last_known_names(ranking.iter().map(|(uname, _)| {
            uname
        })).await.unwrap_or_default();
        self.scores
            .ranking()
            .iter()
            .map(|(uname, pts)| {
                let pname = names 
                    .get(uname)
                    .cloned()
                    .unwrap_or_else(|| uname.clone());
                (uname.clone(), pname, *pts)
            })
            .collect()
    }

    fn connection_props(&self, uname: &str) -> ClockworkRoundConnectionProps {
        ClockworkRoundConnectionProps {
            uname: uname.to_string(),
            in_progress: self.active_runs.contains_key(uname),
            round_best: self.round_best.get(uname).copied(),
        }
    }

    async fn lobby(&self) -> Result<(), SceneError> {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 20);
        let ui = ui::attach::<ClockworkLobbyView>(
            self.insim.clone(),
            self.presence.clone(),
            Duration::ZERO,
        );

        while let Some(_) = countdown.tick().await {
            let remaining = countdown.remaining_duration();
            tracing::info!("updating global props");
            ui.update_global_props(remaining);
        }

        Ok(())
    }

    async fn round(
        &mut self,
        round: usize,
        ui: &ui::Ui<ClockworkRoundView>,
    ) -> Result<(), SceneError> {
        self.round_best.clear();
        self.active_runs.clear();

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
                                rounds: self.rounds,
                                leaderboard: self.enriched_leaderboard().await,
                            });
                        }
                        None => break,
                    }
                },
                packet = packets.recv() => {
                    self.handle_packet(packet.map_err(|_| SceneError::InsimHandleLost)?, ui).await?;
                }
            }
        }

        self.score_round();
        Ok(())
    }

    async fn handle_packet(
        &mut self,
        packet: insim::Packet,
        ui: &ui::Ui<ClockworkRoundView>,
    ) -> Result<(), SceneError> {
        match packet {
            insim::Packet::Ncn(ncn) => {
                self.insim
                    .send_message("Welcome! Game in progress", ncn.ucid)
                    .await?;

                if let Some(conn) = self.presence.connection(&ncn.ucid).await {
                    ui.update_connection_props(ncn.ucid, self.connection_props(&conn.uname))
                        .await;
                }
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
                        let _ = self.active_runs.insert(conn.uname.clone(), time);
                        ui.update_connection_props(conn.ucid, self.connection_props(&conn.uname))
                            .await;
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
                    if let Some(start) = self.active_runs.remove(&conn.uname);
                    then {
                        let delta = time.saturating_sub(start);
                        let diff = self.target.abs_diff(delta);
                        let best = {
                            let entry = self.round_best
                                .entry(conn.uname.clone())
                                .and_modify(|e| {
                                    if diff < *e {
                                        *e = diff;
                                    }
                                })
                                .or_insert(diff);
                            *entry
                        };

                        ui.update_connection_props(conn.ucid, self.connection_props(&conn.uname))
                            .await;

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

    fn score_round(&mut self) {
        let mut ordered: Vec<_> = self.round_best.drain().collect();
        ordered.sort_by_key(|(_, v)| *v);

        for (i, (uname, _)) in ordered.into_iter().take(self.max_scorers).enumerate() {
            let points = (self.max_scorers - i) as u32;
            let _ = self.scores.add_points(uname, points);
        }

        self.scores.rank();

        tracing::info!("{:?}", self.scores);
    }

    async fn announce_results(&mut self) -> Result<(), SceneError> {
        let ui = ui::attach::<ClockworkVictoryView>(
            self.insim.clone(),
            self.presence.clone(),
            ClockworkVictoryGlobalProps::default(),
        );

        if let Some(connections) = self.presence.connections().await {
            for conn in connections {
                ui.update_connection_props(
                    conn.ucid,
                    ClockworkVictoryConnectionProps {
                        uname: conn.uname.clone(),
                    },
                )
                .await;
            }
        }

        let mut countdown = Countdown::new(Duration::from_secs(1), 15);
        while let Some(remaining) = countdown.tick().await {
            ui.update_global_props(ClockworkVictoryGlobalProps {
                standings: self.enriched_leaderboard().await,
                countdown: Duration::from_secs(remaining.into()),
            });
        }

        Ok(())
    }
}



// host + 1 player
const MIN_PLAYERS: usize = 2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = cli::Args::parse();

    let (insim, insim_handle) = insim::tcp(args.addr.clone())
        .isi_admin_password(args.password.clone())
        .isi_iname("clockwork".to_owned())
        .isi_prefix('!')
        .isi_flag_mso_cols(true)
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
            layout: Some("CC".to_string()),
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

    let mut chat_rx = chat.subscribe();
    tokio::select! {
        res = insim_handle => {
            let _ = res.expect("Did not expect insim to die");
        },
        res = clockwork.run() => {
            tracing::info!("{:?}", res);
        },
        _ = wait_for_admin_cmd::wait_for_admin_cmd(&mut chat_rx, presence, |msg| matches!(msg, chat::ChatMsg::Quit)) => {}
    }

    Ok(())
}
