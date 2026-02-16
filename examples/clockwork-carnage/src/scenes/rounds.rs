use std::{collections::HashMap, time::Duration};

use insim::{
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colour,
    },
    identifiers::ConnectionId,
    insim::{ObjectInfo, Uco},
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneResult},
    time::Countdown,
    ui::{self, Component},
};
use tokio::time::sleep;

use crate::{
    chat,
    components::{
        EnrichedLeaderboard, HelpDialog, HelpDialogMsg, hud_active, hud_muted, hud_text, hud_title,
        scoreboard, topbar,
    },
    leaderboard,
};

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

#[derive(Clone, Debug)]
enum ClockworkRoundMessage {
    Help(HelpDialogMsg),
}

impl ui::IntoViewInput<ClockworkRoundMessage> for (ConnectionId, chat::ChatMsg) {
    fn into_view_input(self) -> Option<(ConnectionId, ClockworkRoundMessage)> {
        let (ucid, msg) = self;
        matches!(msg, chat::ChatMsg::Help)
            .then_some((ucid, ClockworkRoundMessage::Help(HelpDialogMsg::Show)))
    }
}

struct ClockworkRoundView {
    help_dialog: HelpDialog,
}

#[derive(Debug, Clone, Default)]
struct ClockworkRoundProps {
    global: ClockworkRoundGlobalProps,
    connection: ClockworkRoundConnectionProps,
}

impl ui::Component for ClockworkRoundView {
    type Props = ClockworkRoundProps;
    type Message = ClockworkRoundMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            ClockworkRoundMessage::Help(help_msg) => {
                Component::update(&mut self.help_dialog, help_msg);
            },
        }
    }

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        if self.help_dialog.is_visible() {
            return self.help_dialog.render(()).map(ClockworkRoundMessage::Help);
        }

        let (status, status_style) = if props.connection.in_progress {
            ("In progress".to_string(), hud_active())
        } else {
            match props.connection.round_best {
                Some(d) => (format!("Best: {:.2?}", d), hud_text()),
                None => ("Waiting for start".to_string(), hud_muted()),
            }
        };

        let players = scoreboard(&props.global.leaderboard, &props.connection.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar(&format!(
                    "Round {}/{} - {:?} remaining",
                    props.global.round, props.global.rounds, props.global.remaining,
                ))
                .with_child(ui::text(status, status_style).w(20.).h(5.)),
            )
            .with_child(
                ui::container()
                    .flex()
                    .pr(5.)
                    .w(200.)
                    .mt(90.)
                    .flex_col()
                    .items_end()
                    .with_child(ui::text("Scores!", hud_title()).w(35.).h(5.))
                    .with_children(players),
            )
    }
}

impl ui::View for ClockworkRoundView {
    type GlobalState = ClockworkRoundGlobalProps;
    type ConnectionState = ClockworkRoundConnectionProps;

    fn mount(_invalidator: ui::InvalidateHandle) -> Self {
        Self {
            help_dialog: HelpDialog::default(),
        }
    }

    fn compose(global: Self::GlobalState, connection: Self::ConnectionState) -> Self::Props {
        ClockworkRoundProps { global, connection }
    }
}

/// Rounds scene - runs multiple rounds and tracks scores
#[derive(Clone)]
pub struct Rounds {
    pub insim: InsimTask,
    pub game: game::Game,
    pub presence: presence::Presence,
    pub chat: chat::Chat,
    pub rounds: usize,
    pub target: Duration,
    pub max_scorers: usize,
}

impl Scene for Rounds {
    type Output = leaderboard::Leaderboard;

    async fn run(mut self) -> Result<SceneResult<Self::Output>, SceneError> {
        let mut state = RoundsState {
            scores: leaderboard::Leaderboard::default(),
            round_best: HashMap::new(),
            active_runs: HashMap::new(),
        };

        let (ui, _ui_handle) = ui::attach::<ClockworkRoundView>(
            self.insim.clone(),
            ClockworkRoundGlobalProps::default(),
        );

        let _chat_task = ui.update_from_broadcast(self.chat.subscribe());

        for round in 1..=self.rounds {
            state.broadcast_rankings(&self, &ui).await?;
            state.run_round(round, &mut self, &ui).await?;
        }

        Ok(SceneResult::Continue(state.scores))
    }
}

struct RoundsState {
    scores: leaderboard::Leaderboard,
    round_best: HashMap<String, Duration>,
    active_runs: HashMap<String, Duration>,
}

impl RoundsState {
    async fn broadcast_rankings(
        &mut self,
        config: &Rounds,
        ui: &ui::Ui<
            ClockworkRoundMessage,
            ClockworkRoundGlobalProps,
            ClockworkRoundConnectionProps,
        >,
    ) -> Result<(), SceneError> {
        let connections =
            config
                .presence
                .connections()
                .await
                .map_err(|cause| SceneError::Custom {
                    scene: "rounds::broadcast_rankings::connections",
                    cause: Box::new(cause),
                })?;

        for conn in connections {
            let props = self.connection_props(&conn.uname);
            ui.set_player_state(conn.ucid, props).await;
        }

        Ok(())
    }

    async fn enriched_leaderboard(
        &self,
        config: &Rounds,
    ) -> Result<EnrichedLeaderboard, SceneError> {
        let ranking = self.scores.ranking();
        let names = config
            .presence
            .last_known_names(ranking.iter().map(|(uname, _)| uname))
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "rounds::enriched_leaderboard::last_known_names",
                cause: Box::new(cause),
            })?;

        Ok(ranking
            .iter()
            .map(|(uname, pts)| {
                let pname = names.get(uname).cloned().unwrap_or_else(|| uname.clone());
                (uname.clone(), pname, *pts)
            })
            .collect::<Vec<_>>()
            .into())
    }

    fn connection_props(&self, uname: &str) -> ClockworkRoundConnectionProps {
        ClockworkRoundConnectionProps {
            uname: uname.to_string(),
            in_progress: self.active_runs.contains_key(uname),
            round_best: self.round_best.get(uname).copied(),
        }
    }

    async fn run_round(
        &mut self,
        round: usize,
        config: &mut Rounds,
        ui: &ui::Ui<
            ClockworkRoundMessage,
            ClockworkRoundGlobalProps,
            ClockworkRoundConnectionProps,
        >,
    ) -> Result<(), SceneError> {
        self.round_best.clear();
        self.active_runs.clear();

        config.insim.send_command("/restart").await?;
        sleep(Duration::from_secs(5)).await;
        config
            .game
            .wait_for_racing()
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "rounds::wait_for_racing",
                cause: Box::new(cause),
            })?;
        sleep(Duration::from_secs(1)).await;

        tracing::info!("Round {}/{}", round, config.rounds);

        let mut countdown = Countdown::new(Duration::from_secs(1), 60);
        let mut packets = config.insim.subscribe();
        let leaderboard = self.enriched_leaderboard(config).await?;

        loop {
            tokio::select! {
                remaining = countdown.tick() => {
                    match remaining {
                        Some(_) => {
                            let dur = countdown.remaining_duration();
                            ui.set_global_state(ClockworkRoundGlobalProps {
                                remaining: dur,
                                round,
                                rounds: config.rounds,
                                leaderboard: leaderboard.clone(),
                            });
                        }
                        None => break,
                    }
                },
                packet = packets.recv() => {
                    self.handle_packet(packet.map_err(|_| SceneError::InsimHandleLost)?, config, ui).await?;
                }
            }
        }

        self.score_round(config);
        Ok(())
    }

    async fn handle_packet(
        &mut self,
        packet: insim::Packet,
        config: &Rounds,
        ui: &ui::Ui<
            ClockworkRoundMessage,
            ClockworkRoundGlobalProps,
            ClockworkRoundConnectionProps,
        >,
    ) -> Result<(), SceneError> {
        match packet {
            insim::Packet::Ncn(ncn) => {
                config
                    .insim
                    .send_message("Welcome! Game in progress", ncn.ucid)
                    .await?;

                if let Some(conn) =
                    config
                        .presence
                        .connection(&ncn.ucid)
                        .await
                        .map_err(|cause| SceneError::Custom {
                            scene: "rounds::handle_packet::connection",
                            cause: Box::new(cause),
                        })?
                {
                    ui.set_player_state(ncn.ucid, self.connection_props(&conn.uname))
                        .await;
                }
            },
            insim::Packet::Uco(Uco {
                info:
                    ObjectInfo::InsimCheckpoint(InsimCheckpoint {
                        kind:
                            kind @ (InsimCheckpointKind::Checkpoint1 | InsimCheckpointKind::Finish),
                        ..
                    }),
                plid,
                time,
                ..
            }) => {
                if let Some(player) =
                    config
                        .presence
                        .player(&plid)
                        .await
                        .map_err(|cause| SceneError::Custom {
                            scene: "rounds::handle_packet::player",
                            cause: Box::new(cause),
                        })?
                    && !player.ptype.is_ai()
                    && let Some(conn) =
                        config
                            .presence
                            .connection_by_player(&plid)
                            .await
                            .map_err(|cause| SceneError::Custom {
                                scene: "rounds::handle_packet::connection_by_player",
                                cause: Box::new(cause),
                            })?
                {
                    match kind {
                        InsimCheckpointKind::Checkpoint1 => {
                            let _ = self.active_runs.insert(conn.uname.clone(), time);
                        },
                        InsimCheckpointKind::Finish => {
                            if let Some(start) = self.active_runs.remove(&conn.uname) {
                                let delta = time.saturating_sub(start);
                                let diff = config.target.abs_diff(delta);
                                let best = {
                                    let entry = self
                                        .round_best
                                        .entry(conn.uname.clone())
                                        .and_modify(|e| {
                                            if diff < *e {
                                                *e = diff;
                                            }
                                        })
                                        .or_insert(diff);
                                    *entry
                                };

                                config
                                    .insim
                                    .send_command(format!("/spec {}", conn.uname))
                                    .await?;
                                config
                                    .insim
                                    .send_message(format!("Off by: {:?}", diff).yellow(), conn.ucid)
                                    .await?;
                                config
                                    .insim
                                    .send_message(
                                        format!("Best: {:?}", best).light_green(),
                                        conn.ucid,
                                    )
                                    .await?;
                                config
                                    .insim
                                    .send_message("Rejoin to retry".yellow(), conn.ucid)
                                    .await?;
                            }
                        },
                        _ => {},
                    }
                    ui.set_player_state(conn.ucid, self.connection_props(&conn.uname))
                        .await;
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn score_round(&mut self, config: &Rounds) {
        let mut ordered: Vec<_> = self.round_best.drain().collect();
        ordered.sort_by_key(|(_, v)| *v);

        for (i, (uname, _)) in ordered.into_iter().take(config.max_scorers).enumerate() {
            let points = (config.max_scorers - i) as u32;
            self.scores.add_points(uname, points);
        }

        self.scores.rank();

        tracing::info!("{:?}", self.scores);
    }
}
