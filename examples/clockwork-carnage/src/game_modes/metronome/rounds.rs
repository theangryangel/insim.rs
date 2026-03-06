use std::{collections::HashMap, time::Duration};

use insim::{
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colour,
    },
    insim::{ObjectInfo, Uco},
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneResult},
    time::Countdown,
    ui::{self, Component},
};
use tokio::time::sleep;

use super::chat;
use crate::{
    components::{
        Dialog, DialogMsg, DialogProps, EventLeaderboard, scoreboard,
        theme::{hud_active, hud_muted, hud_text, hud_title},
        topbar,
    },
    db,
};

const EVENT_HELP_LINES: &[&str] = &[
    " - Match the target lap time as closely as possible.",
    " - Crossing the first checkpoint starts your timed attempt.",
    " - Find one of the finishes as close to the target time as possible.",
    " - Full contact is permitted.",
    " - Don't be a dick.",
    " - Lower delta ranks higher and earns more points.",
    " - Retry as many times as you want each round.",
    "",
    "Good luck.",
];

#[derive(Debug, Clone, Default)]
struct ClockworkRoundGlobalProps {
    remaining: Duration,
    round: usize,
    rounds: usize,
    target: Duration,
    leaderboard: EventLeaderboard,
}

#[derive(Debug, Clone, Default)]
struct ClockworkRoundConnectionProps {
    uname: String,
    in_progress: bool,
    round_best: Option<Duration>,
}

#[derive(Clone, Debug)]
enum ClockworkRoundMessage {
    Help(DialogMsg),
}

struct ClockworkRoundView {
    help_dialog: Dialog,
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
            return self
                .help_dialog
                .render(DialogProps {
                    title: "Welcome to Clockwork Carnage",
                    lines: EVENT_HELP_LINES,
                })
                .map(ClockworkRoundMessage::Help);
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
                .with_child(
                    ui::text(format!("Target: {:.2?}", props.global.target), hud_text())
                        .w(20.)
                        .h(5.),
                )
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

impl From<ui::UiState<ClockworkRoundGlobalProps, ClockworkRoundConnectionProps>>
    for ClockworkRoundProps
{
    fn from(state: ui::UiState<ClockworkRoundGlobalProps, ClockworkRoundConnectionProps>) -> Self {
        Self {
            global: state.global,
            connection: state.connection,
        }
    }
}

/// Rounds scene - runs multiple rounds and tracks scores
#[derive(Clone)]
pub struct Rounds {
    pub insim: InsimTask,
    pub game: game::Game,
    pub presence: presence::Presence,
    pub chat: chat::EventChat,
    pub start_round: usize,
    pub rounds: usize,
    pub target: Duration,
    pub max_scorers: usize,
    pub db: db::Pool,
    pub session_id: i64,
}

impl Scene for Rounds {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<Self::Output>, SceneError> {
        let mut state = RoundsState {
            round_best: HashMap::new(),
            active_runs: HashMap::new(),
        };

        let (ui, _ui_handle) = ui::mount_with(
            self.insim.clone(),
            ClockworkRoundGlobalProps::default(),
            |_ucid, _invalidator| ClockworkRoundView {
                help_dialog: Dialog::default(),
            },
            self.chat.subscribe(),
            |(ucid, msg)| {
                matches!(msg, chat::EventChatMsg::Help)
                    .then_some((ucid, ClockworkRoundMessage::Help(DialogMsg::Show)))
            },
        );

        for round in self.start_round..=self.rounds {
            state.broadcast_rankings(&self, &ui).await?;
            state.run_round(round, &mut self, &ui).await?;
        }

        Ok(SceneResult::Continue(()))
    }
}

struct RoundsState {
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

    async fn event_leaderboard(config: &Rounds) -> Result<EventLeaderboard, SceneError> {
        let standings = db::metronome_standings(&config.db, config.session_id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "rounds::event_leaderboard",
                cause: Box::new(cause),
            })?;

        Ok(standings
            .into_iter()
            .map(|s| (s.uname, s.pname, s.total_points as u32))
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
        let leaderboard = Self::event_leaderboard(config).await?;

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
                                target: config.target,
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

        self.score_round(round, config).await;
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
                                let elapsed = time.saturating_sub(start);
                                let diff = config.target.abs_diff(elapsed);
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

    async fn score_round(&mut self, round: usize, config: &Rounds) {
        let mut ordered: Vec<_> = self.round_best.drain().collect();
        ordered.sort_by_key(|(_, v)| *v);

        for (i, (uname, delta)) in ordered.into_iter().take(config.max_scorers).enumerate() {
            let points = (config.max_scorers - i) as u32;

            let delta_ms = delta.as_millis() as i64;
            if let Err(e) = db::insert_metronome_result(
                &config.db,
                config.session_id,
                round as i64,
                &uname,
                delta_ms,
                points as i64,
            )
            .await
            {
                tracing::warn!("Failed to persist event round result: {e}");
            }
        }

        if let Err(e) = db::update_metronome_round(&config.db, config.session_id, round as i64).await {
            tracing::warn!("Failed to update event round: {e}");
        }
    }
}
