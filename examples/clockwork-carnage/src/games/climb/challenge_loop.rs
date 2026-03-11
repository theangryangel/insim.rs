use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use insim::{
    WithRequestId,
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colour,
        vehicle::Vehicle,
    },
    insim::{ObjectInfo, PmoAction, PmoFlags, TinyType, Uco},
};
use kitcar::{
    game, presence,
    scenes::{FromContext, Scene, SceneError, SceneResult},
    ui::{self, Component},
};
use tokio::sync::broadcast;

use super::chat;
use crate::{
    db,
    hud::{
        ChallengeLeaderboard, Dialog, DialogMsg, DialogProps, challenge_scoreboard,
        theme::{hud_active, hud_muted, hud_text, hud_title},
        topbar,
    },
};

const CLIMB_AXM_REQUEST_ID: insim::identifiers::RequestId = insim::identifiers::RequestId(239);

const CHALLENGE_HELP_LINES: &[&str] = &[
    " - Hit all required checkpoints, then cross the finish.",
    " - Timer starts on your first checkpoint crossing.",
    " - All checkpoints must be hit before finish is counted.",
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
                    title: "Climb Challenge",
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
                topbar("Climb Challenge").with_child(ui::text(status, status_style).w(20.).h(5.)),
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

/// Climb mode — runs indefinitely, players must hit all checkpoints before finish.
#[derive(Clone)]
pub struct ClimbLoop {
    pub chat: chat::ClimbChat,
    pub session_id: i64,
}

impl<Ctx> Scene<Ctx> for ClimbLoop
where
    InsimTask: FromContext<Ctx>,
    game::Game: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    db::Pool: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        let inner = ClimbLoopInner {
            insim: InsimTask::from_context(ctx),
            game: game::Game::from_context(ctx),
            presence: presence::Presence::from_context(ctx),
            db: db::Pool::from_context(ctx),
            chat: self.chat,
            session_id: self.session_id,
        };
        inner.run_inner().await
    }
}

struct ClimbLoopInner {
    insim: InsimTask,
    game: game::Game,
    presence: presence::Presence,
    db: db::Pool,
    chat: chat::ClimbChat,
    session_id: i64,
}

impl ClimbLoopInner {
    async fn scan_cp1_positions(
        &self,
    ) -> Result<HashSet<(i16, i16, u8)>, SceneError> {
        let mut packets = self.insim.subscribe();

        self.insim
            .send(TinyType::Axm.with_request_id(CLIMB_AXM_REQUEST_ID))
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "climb::scan_cp1::send",
                cause: Box::new(cause),
            })?;

        let mut required_positions = HashSet::new();

        loop {
            match packets.recv().await {
                Ok(insim::Packet::Axm(axm))
                    if axm.reqi == CLIMB_AXM_REQUEST_ID
                        && matches!(axm.pmoaction, PmoAction::TinyAxm) =>
                {
                    let is_final = axm.pmoflags.contains(PmoFlags::FILE_END);

                    for object in axm.info {
                        if let ObjectInfo::InsimCheckpoint(cp) = object {
                            if cp.kind == InsimCheckpointKind::Checkpoint1 {
                                let _ = required_positions.insert((cp.xyz.x, cp.xyz.y, cp.xyz.z));
                            }
                        }
                    }

                    if is_final {
                        break;
                    }
                },
                Ok(_) => {},
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("Climb AXM scan lagged by {skipped} packets");
                },
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(SceneError::InsimHandleLost);
                },
            }
        }

        tracing::info!(
            "Climb: found {} required checkpoint positions",
            required_positions.len()
        );
        Ok(required_positions)
    }

    async fn run_inner(mut self) -> Result<SceneResult<()>, SceneError> {
        let required_positions = self.scan_cp1_positions().await?;

        let _spawn_control = crate::games::spawn_control::spawn(self.insim.clone())
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "climb::spawn_control",
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
                matches!(msg, chat::ClimbChatMsg::Help)
                    .then_some((ucid, ChallengeMessage::Help(DialogMsg::Show)))
            },
        );

        let leaderboard = self.challenge_leaderboard().await?;
        ui.set_global_state(ChallengeGlobalProps { leaderboard });

        let mut plid_to_ucid = HashMap::new();
        let mut active_runs: HashMap<
            insim::identifiers::ConnectionId,
            (Duration, HashSet<(i16, i16, u8)>),
        > = HashMap::new();
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    let packet = packet.map_err(|_| SceneError::InsimHandleLost)?;
                    match packet {
                        insim::Packet::Ncn(ncn) => {
                            self.insim
                                .send_message(
                                    "Welcome to Climb! Hit all checkpoints, then finish.",
                                    ncn.ucid,
                                )
                                .await?;

                            if let Some(conn) = self.presence.connection(&ncn.ucid).await.map_err(|cause| SceneError::Custom {
                                scene: "climb::ncn::connection",
                                cause: Box::new(cause),
                            })? {
                                let pb = self.personal_best(&conn.uname).await?;
                                ui.set_player_state(ncn.ucid, ChallengeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: false,
                                    best_time: pb,
                                }).await;
                            }
                        },
                        insim::Packet::Npl(npl) => {
                            let _ = plid_to_ucid.insert(npl.plid, npl.ucid);
                        },
                        insim::Packet::Pll(pll) => {
                            if let Some(ucid) = plid_to_ucid.remove(&pll.plid) {
                                let _ = active_runs.remove(&ucid);
                            }
                        },
                        insim::Packet::Plp(plp) => {
                            if let Some(&ucid) = plid_to_ucid.get(&plp.plid) {
                                let _ = active_runs.remove(&ucid);
                            }
                        },
                        insim::Packet::Cnl(cnl) => {
                            let _ = active_runs.remove(&cnl.ucid);
                            plid_to_ucid.retain(|_, v| *v != cnl.ucid);
                        },
                        insim::Packet::Uco(Uco {
                            info:
                                ObjectInfo::InsimCheckpoint(InsimCheckpoint {
                                    kind:
                                        kind @ (InsimCheckpointKind::Checkpoint1
                                            | InsimCheckpointKind::Finish),
                                    xyz,
                                    ..
                                }),
                            plid,
                            time,
                            ..
                        }) => {
                            if let Some(player) = self.presence.player(&plid).await.map_err(|cause| SceneError::Custom {
                                scene: "climb::uco::player",
                                cause: Box::new(cause),
                            })?
                                && !player.ptype.is_ai()
                                && let Some(conn) = self.presence.connection_by_player(&plid).await.map_err(|cause| SceneError::Custom {
                                    scene: "climb::uco::connection_by_player",
                                    cause: Box::new(cause),
                                })?
                            {
                                let ucid = conn.ucid;
                                match kind {
                                    InsimCheckpointKind::Checkpoint1 => {
                                        let pos = (xyz.x, xyz.y, xyz.z);
                                        if required_positions.contains(&pos) {
                                            let run = active_runs
                                                .entry(ucid)
                                                .or_insert_with(|| (time, HashSet::new()));
                                            let _ = run.1.insert(pos);
                                        }
                                    },
                                    InsimCheckpointKind::Finish => {
                                        if let Some((start, hit_set)) = active_runs.remove(&ucid) {
                                            let vehicle = player.vehicle;
                                            if hit_set == required_positions {
                                                let lap_time = time.saturating_sub(start);
                                                let time_ms = lap_time.as_millis() as i64;

                                                let prev_pb = self.personal_best(&conn.uname).await?;
                                                let is_pb = match prev_pb {
                                                    Some(prev) => lap_time < prev,
                                                    None => true,
                                                };

                                                if let Err(e) = db::insert_climb_time(
                                                    &self.db,
                                                    self.session_id,
                                                    &conn.uname,
                                                    &vehicle.to_string(),
                                                    time_ms,
                                                )
                                                .await
                                                {
                                                    tracing::warn!(
                                                        "Failed to persist climb time: {e}"
                                                    );
                                                }

                                                self.insim
                                                    .send_command(format!("/spec {}", conn.uname))
                                                    .await?;

                                                if is_pb {
                                                    self.insim
                                                        .send_message(
                                                            format!(
                                                                "New PB! {:.2?} ({})",
                                                                lap_time, vehicle
                                                            )
                                                            .light_green(),
                                                            ucid,
                                                        )
                                                        .await?;
                                                } else if let Some(pb) = prev_pb {
                                                    self.insim
                                                        .send_message(
                                                            format!(
                                                                "Time: {:.2?}, PB: {:.2?}",
                                                                lap_time, pb
                                                            )
                                                            .yellow(),
                                                            ucid,
                                                        )
                                                        .await?;
                                                }

                                                self.insim
                                                    .send_message("Rejoin to retry".yellow(), ucid)
                                                    .await?;

                                                let leaderboard =
                                                    self.challenge_leaderboard().await?;
                                                ui.set_global_state(ChallengeGlobalProps {
                                                    leaderboard,
                                                });
                                            } else {
                                                self.insim
                                                    .send_command(format!("/spec {}", conn.uname))
                                                    .await?;
                                                self.insim
                                                    .send_message(
                                                        "Missed checkpoints — rejoin to retry"
                                                            .red(),
                                                        ucid,
                                                    )
                                                    .await?;
                                            }
                                        }
                                    },
                                    _ => {},
                                }

                                let pb = self.personal_best(&conn.uname).await?;
                                ui.set_player_state(ucid, ChallengeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: active_runs.contains_key(&ucid),
                                    best_time: pb,
                                })
                                .await;
                            }
                        },
                        _ => {},
                    }
                },
                _ = self.chat.wait_for_admin_cmd(self.presence.clone(), |msg| matches!(msg, chat::ClimbChatMsg::End)) => {
                    tracing::info!("Admin ended climb");
                    return Ok(SceneResult::Continue(()));
                },
                _ = self.game.wait_for_end() => {
                    tracing::info!("Game ended");
                    return Ok(SceneResult::Continue(()));
                },
            }
        }
    }

    async fn challenge_leaderboard(&self) -> Result<ChallengeLeaderboard, SceneError> {
        let rows = db::climb_best_times(&self.db, self.session_id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "climb::challenge_leaderboard",
                cause: Box::new(cause),
            })?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let vehicle: Vehicle = row.vehicle.parse().unwrap_or(Vehicle::Uf1);
                let time = Duration::from_millis(row.time_ms as u64);
                (row.uname, row.pname, vehicle, time)
            })
            .collect::<Vec<_>>()
            .into())
    }

    async fn personal_best(&self, uname: &str) -> Result<Option<Duration>, SceneError> {
        let row = db::climb_personal_best(&self.db, self.session_id, uname)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "climb::personal_best",
                cause: Box::new(cause),
            })?;

        Ok(row.map(|r| Duration::from_millis(r.time_ms as u64)))
    }
}
