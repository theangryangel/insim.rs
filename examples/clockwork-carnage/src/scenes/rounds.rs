use std::{collections::HashMap, time::Duration};

use insim::{
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colourify,
    },
    insim::{BtnStyle, ObjectInfo, Uco},
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneResult},
    time::Countdown,
    ui,
};
use tokio::time::sleep;

use crate::{
    components::{EnrichedLeaderboard, scoreboard, topbar},
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

        let players = scoreboard(&global_props.leaderboard, &connection_props.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar(&format!(
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

/// Rounds scene - runs multiple rounds and tracks scores
#[derive(Clone)]
pub struct Rounds {
    pub insim: InsimTask,
    pub game: game::Game,
    pub presence: presence::Presence,
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
            self.presence.clone(),
            ClockworkRoundGlobalProps::default(),
        );

        for round in 1..=self.rounds {
            state.broadcast_rankings(&self, &ui).await;
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
    async fn broadcast_rankings(&mut self, config: &Rounds, ui: &ui::Ui<ClockworkRoundView>) {
        if let Some(connections) = config.presence.connections().await {
            for conn in connections {
                let props = self.connection_props(&conn.uname);
                ui.update_connection_props(conn.ucid, props).await;
            }
        }
    }

    async fn enriched_leaderboard(&self, config: &Rounds) -> EnrichedLeaderboard {
        let ranking = self.scores.ranking();
        let names = config
            .presence
            .last_known_names(ranking.iter().map(|(uname, _)| uname))
            .await
            .unwrap_or_default();
        self.scores
            .ranking()
            .iter()
            .map(|(uname, pts)| {
                let pname = names.get(uname).cloned().unwrap_or_else(|| uname.clone());
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

    async fn run_round(
        &mut self,
        round: usize,
        config: &mut Rounds,
        ui: &ui::Ui<ClockworkRoundView>,
    ) -> Result<(), SceneError> {
        self.round_best.clear();
        self.active_runs.clear();

        config.insim.send_command("/restart").await?;
        sleep(Duration::from_secs(5)).await;
        config.game.wait_for_racing().await;
        sleep(Duration::from_secs(1)).await;

        tracing::info!("Round {}/{}", round, config.rounds);

        let mut countdown = Countdown::new(Duration::from_secs(1), 60);
        let mut packets = config.insim.subscribe();

        loop {
            tokio::select! {
                remaining = countdown.tick() => {
                    match remaining {
                        Some(_) => {
                            let dur = countdown.remaining_duration();
                            ui.update_global_props(ClockworkRoundGlobalProps {
                                remaining: dur,
                                round,
                                rounds: config.rounds,
                                leaderboard: self.enriched_leaderboard(config).await,
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
        ui: &ui::Ui<ClockworkRoundView>,
    ) -> Result<(), SceneError> {
        match packet {
            insim::Packet::Ncn(ncn) => {
                config
                    .insim
                    .send_message("Welcome! Game in progress", ncn.ucid)
                    .await?;

                if let Some(conn) = config.presence.connection(&ncn.ucid).await {
                    ui.update_connection_props(ncn.ucid, self.connection_props(&conn.uname))
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
                if_chain::if_chain! {
                    if let Some(player) = config.presence.player(&plid).await;
                    if !player.ptype.is_ai();
                    if let Some(conn) = config.presence.connection_by_player(&plid).await;
                    then {
                        match kind {
                            InsimCheckpointKind::Checkpoint1 => {
                                let _ = self.active_runs.insert(conn.uname.clone(), time);
                            }
                            InsimCheckpointKind::Finish => {
                                if let Some(start) = self.active_runs.remove(&conn.uname) {
                                    let delta = time.saturating_sub(start);
                                    let diff = config.target.abs_diff(delta);
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

                                    config.insim
                                        .send_command(format!("/spec {}", conn.uname))
                                        .await?;
                                    config.insim
                                        .send_message(format!("Off by: {:?}", diff).yellow(), conn.ucid)
                                        .await?;
                                    config.insim
                                        .send_message(format!("Best: {:?}", best).light_green(), conn.ucid)
                                        .await?;
                                    config.insim
                                        .send_message("Rejoin to retry".yellow(), conn.ucid)
                                        .await?;
                                }
                            }
                            _ => {}
                        }
                        ui.update_connection_props(conn.ucid, self.connection_props(&conn.uname))
                            .await;
                    }
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
            let _ = self.scores.add_points(uname, points);
        }

        self.scores.rank();

        tracing::info!("{:?}", self.scores);
    }
}
