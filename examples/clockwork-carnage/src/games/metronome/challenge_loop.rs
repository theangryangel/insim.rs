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
    scenes::{FromContext, Scene, SceneError, SceneResult},
    ui::{self, Component},
};

use super::chat;
use crate::{
    hud::{
        MetronomeLeaderboard, Dialog, DialogMsg, DialogProps, metronome_scoreboard,
        theme::{hud_active, hud_muted, hud_text, hud_title},
        topbar,
    },
    db,
};

const METRONOME_HELP_LINES: &[&str] = &[
    " - Cross checkpoint 1 to start your timed attempt.",
    " - Reach the finish to record your delta from the target.",
    " - The smallest delta wins.",
    " - Retry as many times as you like — no spec, no limit.",
    " - Platinum: ≤0.1s | Gold: ≤0.5s | Silver: ≤2s | Bronze: ≤5s",
    "",
    "Good luck.",
];

fn tier_label(delta: Duration) -> Option<&'static str> {
    let ms = delta.as_millis();
    if ms <= 100 {
        Some("Platinum")
    } else if ms <= 500 {
        Some("Gold")
    } else if ms <= 2000 {
        Some("Silver")
    } else if ms <= 5000 {
        Some("Bronze")
    } else {
        None
    }
}

#[derive(Debug, Clone, Default)]
struct MetronomeGlobalProps {
    target: Duration,
    leaderboard: MetronomeLeaderboard,
}

#[derive(Debug, Clone, Default)]
struct MetronomeConnectionProps {
    uname: String,
    in_progress: bool,
    best_delta: Option<Duration>,
}

#[derive(Clone, Debug)]
enum MetronomeMessage {
    Help(DialogMsg),
}

struct MetronomeView {
    help_dialog: Dialog,
}

#[derive(Debug, Clone, Default)]
struct MetronomeProps {
    global: MetronomeGlobalProps,
    connection: MetronomeConnectionProps,
}

impl ui::Component for MetronomeView {
    type Props = MetronomeProps;
    type Message = MetronomeMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            MetronomeMessage::Help(help_msg) => {
                Component::update(&mut self.help_dialog, help_msg);
            },
        }
    }

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        if self.help_dialog.is_visible() {
            return self
                .help_dialog
                .render(DialogProps {
                    title: "Clockwork Carnage",
                    lines: METRONOME_HELP_LINES,
                })
                .map(MetronomeMessage::Help);
        }

        let (status, status_style) = if props.connection.in_progress {
            ("In progress".to_string(), hud_active())
        } else {
            match props.connection.best_delta {
                Some(d) => {
                    let tier = tier_label(d).unwrap_or("No tier");
                    (format!("Best: {:.2?} [{}]", d, tier), hud_text())
                },
                None => ("Waiting for start".to_string(), hud_muted()),
            }
        };

        let players = metronome_scoreboard(&props.global.leaderboard, &props.connection.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar(&format!("Target: {:.2?}", props.global.target))
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
                    .with_child(ui::text("Best Deltas", hud_title()).w(35.).h(5.))
                    .with_children(players),
            )
    }
}

impl From<ui::UiState<MetronomeGlobalProps, MetronomeConnectionProps>> for MetronomeProps {
    fn from(state: ui::UiState<MetronomeGlobalProps, MetronomeConnectionProps>) -> Self {
        Self {
            global: state.global,
            connection: state.connection,
        }
    }
}

/// Open-format metronome — runs indefinitely, players compete for closest delta to target.
#[derive(Clone)]
pub struct ChallengeLoop {
    pub chat: chat::EventChat,
    pub target: Duration,
    pub session_id: i64,
}

impl<Ctx> Scene<Ctx> for ChallengeLoop
where
    InsimTask: FromContext<Ctx>,
    game::Game: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    db::Pool: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError> {
        let inner = ChallengeLoopInner {
            insim: InsimTask::from_context(ctx),
            game: game::Game::from_context(ctx),
            presence: presence::Presence::from_context(ctx),
            db: db::Pool::from_context(ctx),
            chat: self.chat,
            target: self.target,
            session_id: self.session_id,
        };
        inner.run_inner().await
    }
}

struct ChallengeLoopInner {
    insim: InsimTask,
    game: game::Game,
    presence: presence::Presence,
    db: db::Pool,
    chat: chat::EventChat,
    target: Duration,
    session_id: i64,
}

impl ChallengeLoopInner {
    async fn run_inner(mut self) -> Result<SceneResult<()>, SceneError> {
        let _spawn_control = crate::games::spawn_control::spawn(self.insim.clone())
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "metronome::spawn_control",
                cause: Box::new(cause),
            })?;

        let (ui, _ui_handle) = ui::mount_with(
            self.insim.clone(),
            MetronomeGlobalProps {
                target: self.target,
                leaderboard: MetronomeLeaderboard::default(),
            },
            |_ucid, _invalidator| MetronomeView {
                help_dialog: Dialog::default(),
            },
            self.chat.subscribe(),
            |(ucid, msg)| {
                matches!(msg, chat::EventChatMsg::Help)
                    .then_some((ucid, MetronomeMessage::Help(DialogMsg::Show)))
            },
        );

        let leaderboard = self.metronome_leaderboard().await?;
        ui.set_global_state(MetronomeGlobalProps { target: self.target, leaderboard });

        let mut active_runs: HashMap<String, Duration> = HashMap::new();
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    let packet = packet.map_err(|_| SceneError::InsimHandleLost)?;
                    match packet {
                        insim::Packet::Ncn(ncn) => {
                            self.insim
                                .send_message("Welcome to Clockwork Carnage! Drive CP1→Finish. Match the target time.", ncn.ucid)
                                .await?;

                            if let Some(conn) = self.presence.connection(&ncn.ucid).await.map_err(|cause| SceneError::Custom {
                                scene: "metronome::ncn::connection",
                                cause: Box::new(cause),
                            })? {
                                let pb = self.personal_best(&conn.uname).await?;
                                ui.set_player_state(ncn.ucid, MetronomeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: active_runs.contains_key(&conn.uname),
                                    best_delta: pb,
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
                                scene: "metronome::uco::player",
                                cause: Box::new(cause),
                            })?
                                && !player.ptype.is_ai()
                                && let Some(conn) = self.presence.connection_by_player(&plid).await.map_err(|cause| SceneError::Custom {
                                    scene: "metronome::uco::connection_by_player",
                                    cause: Box::new(cause),
                                })?
                            {
                                match kind {
                                    InsimCheckpointKind::Checkpoint1 => {
                                        let _ = active_runs.insert(conn.uname.clone(), time);
                                    },
                                    InsimCheckpointKind::Finish => {
                                        if let Some(start) = active_runs.remove(&conn.uname) {
                                            let elapsed = time.saturating_sub(start);
                                            let delta = self.target.abs_diff(elapsed);

                                            if let Err(e) = db::insert_metronome_lap(
                                                &self.db,
                                                self.session_id,
                                                &conn.uname,
                                                delta.as_millis() as i64,
                                            ).await {
                                                tracing::warn!("Failed to persist metronome lap: {e}");
                                            }

                                            let prev_best = self.personal_best(&conn.uname).await?;
                                            let is_pb = match prev_best {
                                                Some(prev) => delta < prev,
                                                None => true,
                                            };

                                            let tier_str = tier_label(delta)
                                                .map(|t| format!(" [{}]", t))
                                                .unwrap_or_default();

                                            if is_pb {
                                                self.insim
                                                    .send_message(
                                                        format!("New best! Off by: {:.2?}{}", delta, tier_str).light_green(),
                                                        conn.ucid,
                                                    )
                                                    .await?;
                                            } else {
                                                self.insim
                                                    .send_message(
                                                        format!("Off by: {:.2?}{}", delta, tier_str).yellow(),
                                                        conn.ucid,
                                                    )
                                                    .await?;
                                            }

                                            let leaderboard = self.metronome_leaderboard().await?;
                                            ui.set_global_state(MetronomeGlobalProps { target: self.target, leaderboard });
                                        }
                                    },
                                    _ => {},
                                }

                                let pb = self.personal_best(&conn.uname).await?;
                                ui.set_player_state(conn.ucid, MetronomeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: active_runs.contains_key(&conn.uname),
                                    best_delta: pb,
                                }).await;
                            }
                        },
                        _ => {},
                    }
                },
                _ = self.game.wait_for_end() => {
                    tracing::info!("Metronome game ended");
                    return Ok(SceneResult::Continue(()));
                },
            }
        }
    }

    async fn metronome_leaderboard(&self) -> Result<MetronomeLeaderboard, SceneError> {
        let rows = db::metronome_standings(&self.db, self.session_id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "metronome::metronome_leaderboard",
                cause: Box::new(cause),
            })?;

        Ok(rows
            .into_iter()
            .map(|row| (row.uname, row.pname, Duration::from_millis(row.best_delta_ms as u64)))
            .collect::<Vec<_>>()
            .into())
    }

    async fn personal_best(&self, uname: &str) -> Result<Option<Duration>, SceneError> {
        let ms = db::metronome_personal_best(&self.db, self.session_id, uname)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "metronome::personal_best",
                cause: Box::new(cause),
            })?;

        Ok(ms.map(|v| Duration::from_millis(v as u64)))
    }
}
