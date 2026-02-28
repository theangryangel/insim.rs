use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use insim::{
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colour,
        vehicle::Vehicle,
    },
    insim::{ObjectInfo, Uco},
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneResult},
    ui::{self, Component},
};

use crate::{
    chat,
    components::{
        ChallengeLeaderboard, Dialog, DialogMsg, DialogProps, challenge_scoreboard,
        theme::{hud_active, hud_muted, hud_text, hud_title},
        topbar,
    },
};

const CHALLENGE_HELP_LINES: &[&str] = &[
    " - Drive from checkpoint to finish as fast as possible.",
    " - Crossing the any Start checkpoint starts your timed attempt.",
    " - Reach any Finish to record your time.",
    " - Your personal best is tracked across attempts.",
    " - Rejoin to retry as many times as you want.",
    "",
    "Good luck.",
];

#[derive(Debug, Clone, Default)]
struct ChallengeGlobalProps {
    leaderboard: ChallengeLeaderboard,
}

#[derive(Debug, Clone, Default)]
struct ChallengeConnectionProps {
    uname: String,
    in_progress: bool,
    best_time: Option<Duration>,
}

#[derive(Clone, Debug)]
enum ChallengeMessage {
    Help(DialogMsg),
}

struct ChallengeView {
    help_dialog: Dialog,
}

#[derive(Debug, Clone, Default)]
struct ChallengeProps {
    global: ChallengeGlobalProps,
    connection: ChallengeConnectionProps,
}

impl ui::Component for ChallengeView {
    type Props = ChallengeProps;
    type Message = ChallengeMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            ChallengeMessage::Help(help_msg) => {
                Component::update(&mut self.help_dialog, help_msg);
            },
        }
    }

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        if self.help_dialog.is_visible() {
            return self
                .help_dialog
                .render(DialogProps {
                    title: "Weekly Challenge",
                    lines: CHALLENGE_HELP_LINES,
                })
                .map(ChallengeMessage::Help);
        }

        let (status, status_style) = if props.connection.in_progress {
            ("In progress".to_string(), hud_active())
        } else {
            match props.connection.best_time {
                Some(d) => (format!("PB: {:.2?}", d), hud_text()),
                None => ("Waiting for start".to_string(), hud_muted()),
            }
        };

        let players = challenge_scoreboard(&props.global.leaderboard, &props.connection.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar("Weekly Challenge").with_child(ui::text(status, status_style).w(20.).h(5.)),
            )
            .with_child(
                ui::container()
                    .flex()
                    .pr(5.)
                    .w(200.)
                    .mt(90.)
                    .flex_col()
                    .items_end()
                    .with_child(ui::text("Best Times", hud_title()).w(35.).h(5.))
                    .with_children(players),
            )
    }
}

impl From<ui::UiState<ChallengeGlobalProps, ChallengeConnectionProps>> for ChallengeProps {
    fn from(state: ui::UiState<ChallengeGlobalProps, ChallengeConnectionProps>) -> Self {
        Self {
            global: state.global,
            connection: state.connection,
        }
    }
}

/// Challenge mode — runs indefinitely, players compete for fastest time.
#[derive(Clone)]
pub struct ChallengeLoop {
    pub insim: InsimTask,
    pub game: game::Game,
    pub presence: presence::Presence,
    pub chat: chat::ChallengeChat,
}

impl Scene for ChallengeLoop {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<()>, SceneError> {
        let _spawn_control = crate::spawn_control::spawn(self.insim.clone())
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "challenge::spawn_control",
                cause: Box::new(cause),
            })?;

        let (ui, _ui_handle) = ui::mount_with(
            self.insim.clone(),
            ChallengeGlobalProps::default(),
            |_ucid, _invalidator| ChallengeView {
                help_dialog: Dialog::default(),
            },
            self.chat.subscribe(),
            |(ucid, msg)| {
                matches!(msg, chat::ChallengeChatMsg::Help)
                    .then_some((ucid, ChallengeMessage::Help(DialogMsg::Show)))
            },
        );

        let mut best_times: HashMap<String, (Duration, Vehicle, SystemTime)> = HashMap::new();
        let mut active_runs: HashMap<String, Duration> = HashMap::new();
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    let packet = packet.map_err(|_| SceneError::InsimHandleLost)?;
                    match packet {
                        insim::Packet::Ncn(ncn) => {
                            self.insim
                                .send_message("Welcome to the Weekly Challenge! Drive checkpoint1 to finish for fastest time.", ncn.ucid)
                                .await?;

                            if let Some(conn) = self.presence.connection(&ncn.ucid).await.map_err(|cause| SceneError::Custom {
                                scene: "challenge::ncn::connection",
                                cause: Box::new(cause),
                            })? {
                                ui.set_player_state(ncn.ucid, ChallengeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: false,
                                    best_time: best_times.get(&conn.uname).map(|(t, _, _)| *t),
                                }).await;
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
                            if let Some(player) = self.presence.player(&plid).await.map_err(|cause| SceneError::Custom {
                                scene: "challenge::uco::player",
                                cause: Box::new(cause),
                            })?
                                && !player.ptype.is_ai()
                                && let Some(conn) = self.presence.connection_by_player(&plid).await.map_err(|cause| SceneError::Custom {
                                    scene: "challenge::uco::connection_by_player",
                                    cause: Box::new(cause),
                                })?
                            {
                                match kind {
                                    InsimCheckpointKind::Checkpoint1 => {
                                        let _ = active_runs.insert(conn.uname.clone(), time);
                                    },
                                    InsimCheckpointKind::Finish => {
                                        if let Some(start) = active_runs.remove(&conn.uname) {
                                            let lap_time = time.saturating_sub(start);
                                            let vehicle = player.vehicle;
                                            let is_pb = match best_times.get(&conn.uname) {
                                                Some(&(prev, _, _)) => lap_time < prev,
                                                None => true,
                                            };

                                            if is_pb {
                                                let _ = best_times.insert(conn.uname.clone(), (lap_time, vehicle, SystemTime::now()));
                                            }

                                            self.insim
                                                .send_command(format!("/spec {}", conn.uname))
                                                .await?;

                                            if is_pb {
                                                self.insim
                                                    .send_message(
                                                        format!("New PB! {:.2?} ({})", lap_time, vehicle).light_green(),
                                                        conn.ucid,
                                                    )
                                                    .await?;
                                            } else {
                                                let (pb, _, _) = best_times[&conn.uname];
                                                self.insim
                                                    .send_message(
                                                        format!("Time: {:.2?}, PB: {:.2?}", lap_time, pb).yellow(),
                                                        conn.ucid,
                                                    )
                                                    .await?;
                                            }

                                            self.insim
                                                .send_message("Rejoin to retry".yellow(), conn.ucid)
                                                .await?;

                                            // Update leaderboard
                                            let leaderboard = self.build_leaderboard(&best_times).await?;
                                            ui.set_global_state(ChallengeGlobalProps { leaderboard });
                                        }
                                    },
                                    _ => {},
                                }

                                ui.set_player_state(conn.ucid, ChallengeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: active_runs.contains_key(&conn.uname),
                                    best_time: best_times.get(&conn.uname).map(|(t, _, _)| *t),
                                }).await;
                            }
                        },
                        _ => {},
                    }
                },
                _ = self.chat.wait_for_admin_cmd(self.presence.clone(), |msg| matches!(msg, chat::ChallengeChatMsg::End)) => {
                    tracing::info!("Admin ended challenge");
                    return Ok(SceneResult::Continue(()));
                },
                _ = self.game.wait_for_end() => {
                    tracing::info!("Game ended");
                    return Ok(SceneResult::Continue(()));
                },
            }
        }
    }
}

impl ChallengeLoop {
    async fn build_leaderboard(
        &self,
        best_times: &HashMap<String, (Duration, Vehicle, SystemTime)>,
    ) -> Result<ChallengeLeaderboard, SceneError> {
        let mut entries: Vec<(String, Duration, Vehicle, SystemTime)> = best_times
            .iter()
            .map(|(k, &(t, v, ts))| (k.clone(), t, v, ts))
            .collect();
        entries.sort_by_key(|(_, t, _, _)| *t);

        let names = self
            .presence
            .last_known_names(entries.iter().map(|(uname, _, _, _)| uname))
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "challenge::build_leaderboard::last_known_names",
                cause: Box::new(cause),
            })?;

        Ok(entries
            .into_iter()
            .map(|(uname, time, vehicle, set_at)| {
                let pname = names.get(&uname).cloned().unwrap_or_else(|| uname.clone());
                (uname, pname, vehicle, time, set_at)
            })
            .collect::<Vec<_>>()
            .into())
    }
}
