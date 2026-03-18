use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use insim::{
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colour,
        vehicle::Vehicle,
    },
    identifiers::ConnectionId,
    insim::{Cnl, ObjectInfo, Pll, Uco},
};
use kitcar::{
    game, presence,
    scenes::{FromContext, Scene, SceneError, SceneResult},
    ui::{self, Component},
};

use super::chat;
use crate::{
    db,
    hud::{
        BombLeaderboard, Dialog, DialogMsg, DialogProps, bomb_scoreboard,
        theme::{hud_active, hud_muted, hud_text, hud_title},
        topbar,
    },
};

const BOMB_HELP_LINES: &[&str] = &[
    " - Hit ^2checkpoint 1^7 objects before the timer expires or your run ends (BOOM).",
    " - Time carries over: arrive late and you have less time to reach the next one.",
    " - Hit a ^3checkpoint 2/3^7 or ^3finish^7 object to refresh your timer to the 1/4th window.",
    " - Score = checkpoints hit. Survival time breaks ties.",
    " - Your best run is recorded on the leaderboard.",
    "",
    "Good luck.",
];

#[derive(Debug, Clone, Default)]
struct BombGlobalProps {
    leaderboard: BombLeaderboard,
    active_runs: Vec<(String, String, i64, Instant, Duration)>, // (uname, pname, cps, deadline, current_timeout)
}

#[derive(Debug, Clone, Default)]
struct BombConnectionProps {
    uname: String,
    in_run: bool,
}

#[derive(Clone, Debug)]
enum BombMessage {
    Help(DialogMsg),
}

struct BombView {
    help_dialog: Dialog,
    _tick_handle: tokio::task::JoinHandle<()>,
}

#[derive(Debug, Clone, Default)]
struct BombProps {
    global: BombGlobalProps,
    connection: BombConnectionProps,
}

impl ui::Component for BombView {
    type Props<'a> = BombProps;
    type Message = BombMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            BombMessage::Help(help_msg) => {
                Component::update(&mut self.help_dialog, help_msg);
            },
        }
    }

    fn render(&self, props: Self::Props<'_>) -> ui::Node<Self::Message> {
        if self.help_dialog.is_visible() {
            return self
                .help_dialog
                .render(DialogProps {
                    title: "Bomb",
                    lines: BOMB_HELP_LINES,
                })
                .map(BombMessage::Help);
        }

        let status_str = if props.connection.in_run {
            "In run".to_string()
        } else {
            "Waiting".to_string()
        };
        let status_style = if props.connection.in_run {
            hud_active()
        } else {
            hud_muted()
        };

        let leaderboard_rows = bomb_scoreboard(&props.global.leaderboard, &props.connection.uname);

        let now = Instant::now();
        let active_run_rows: Vec<ui::Node<BombMessage>> = props
            .global
            .active_runs
            .iter()
            .map(|(uname, pname, cps, deadline, current_timeout)| {
                let secs_left = deadline.saturating_duration_since(now).as_secs_f64();
                let fraction = if current_timeout.is_zero() {
                    0.0
                } else {
                    (secs_left / current_timeout.as_secs_f64()).clamp(0.0, 1.0)
                };
                let cps_str = format!("{cps} cps");
                let time_str = format!("{secs_left:.1}s");
                // 8-char progress bar
                let filled = (fraction * 8.0).round() as usize;
                let bar: String = (0..8).map(|i| if i < filled { '█' } else { '░' }).collect();
                let style = if uname.as_str() == props.connection.uname.as_str() {
                    hud_active()
                } else {
                    hud_text()
                };

                ui::container().flex().flex_row().with_children([
                    ui::text(pname.as_str(), style.align_left()).w(15.).h(5.),
                    ui::text(cps_str, style.align_right()).w(8.).h(5.),
                    ui::text(time_str, style.align_right()).w(8.).h(5.),
                    ui::text(bar, style).w(10.).h(5.),
                ])
            })
            .collect();

        ui::container()
            .flex()
            .flex_col()
            .with_child(topbar("Bomb").with_child(ui::text(status_str, status_style).w(15.).h(5.)))
            .with_child(
                ui::container()
                    .flex()
                    .pr(5.)
                    .w(200.)
                    .mt(90.)
                    .flex_col()
                    .items_end()
                    .with_child(ui::text("Active Runs", hud_title()).w(35.).h(5.))
                    .with_children(active_run_rows)
                    .with_child(ui::text("Session Best", hud_title()).w(35.).h(5.))
                    .with_children(leaderboard_rows),
            )
    }
}

impl From<ui::UiState<BombGlobalProps, BombConnectionProps>> for BombProps {
    fn from(state: ui::UiState<BombGlobalProps, BombConnectionProps>) -> Self {
        Self {
            global: state.global,
            connection: state.connection,
        }
    }
}

struct ActiveRun {
    started_at: Instant,
    deadline: Instant,
    current_timeout: Duration,
    checkpoints: i64,
    vehicle: Vehicle,
    uname: String,
    pname: String,
    ucid: ConnectionId,
}

#[derive(Clone)]
pub struct BombLoop {
    pub chat: chat::BombChat,
    pub session_id: i64,
    pub checkpoint_timeout: Duration,
}

impl<Ctx> Scene<Ctx> for BombLoop
where
    InsimTask: FromContext<Ctx>,
    game::Game: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    db::Pool: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        let inner = BombLoopInner {
            insim: InsimTask::from_context(ctx),
            game: game::Game::from_context(ctx),
            presence: presence::Presence::from_context(ctx),
            db: db::Pool::from_context(ctx),
            chat: self.chat,
            session_id: self.session_id,
            checkpoint_timeout: self.checkpoint_timeout,
        };
        inner.run_inner().await
    }
}

struct BombLoopInner {
    insim: InsimTask,
    game: game::Game,
    presence: presence::Presence,
    db: db::Pool,
    chat: chat::BombChat,
    session_id: i64,
    checkpoint_timeout: Duration,
}

impl BombLoopInner {
    async fn run_inner(mut self) -> Result<SceneResult<()>, SceneError> {
        let (ui, _ui_handle) = ui::mount_with(
            self.insim.clone(),
            BombGlobalProps::default(),
            |_ucid, invalidator| {
                let handle = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_millis(100));
                    loop {
                        let _ = interval.tick().await;
                        invalidator.invalidate();
                    }
                });
                BombView {
                    help_dialog: Dialog::default(),
                    _tick_handle: handle,
                }
            },
            self.chat.subscribe(),
            |(ucid, msg)| {
                matches!(msg, chat::BombChatMsg::Help)
                    .then_some((ucid, BombMessage::Help(DialogMsg::Show)))
            },
        );

        // Load initial leaderboard
        let leaderboard = self.load_leaderboard().await?;
        ui.set_global_state(BombGlobalProps {
            leaderboard,
            active_runs: vec![],
        });

        // keyed by uname
        let mut active_runs: HashMap<String, ActiveRun> = HashMap::new();
        let mut packets = self.insim.subscribe();
        let mut tick = tokio::time::interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    let packet = packet.map_err(|_| SceneError::InsimHandleLost)?;
                    match packet {
                        insim::Packet::Ncn(ncn) => {
                            self.insim
                                .send_message(
                                    format!(
                                        "Welcome to Bomb Mode! Hit checkpoints before {}s timer expires.",
                                        self.checkpoint_timeout.as_secs()
                                    ),
                                    ncn.ucid,
                                )
                                .await?;

                            if let Some(conn) = self.presence.connection(&ncn.ucid).await.map_err(|cause| SceneError::Custom {
                                scene: "bomb::ncn::connection",
                                cause: Box::new(cause),
                            })? {
                                ui.set_player_state(ncn.ucid, BombConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_run: false,
                                }).await;
                            }
                        },
                        insim::Packet::Pll(Pll { plid, .. }) => {
                            // Player left race — look up connection via player
                            if let Some(conn) = self.presence.connection_by_player(&plid).await.map_err(|cause| SceneError::Custom {
                                scene: "bomb::pll::connection",
                                cause: Box::new(cause),
                            })? {
                                if let Some(run) = active_runs.remove(&conn.uname) {
                                    let now = Instant::now();
                                    let survival_ms = (run.deadline.min(now) - run.started_at).as_millis() as i64;
                                    if let Err(e) = db::insert_bomb_run(
                                        &self.db,
                                        self.session_id,
                                        &conn.uname,
                                        &run.vehicle.to_string(),
                                        run.checkpoints,
                                        survival_ms,
                                    ).await {
                                        tracing::warn!("Failed to persist bomb run on disconnect: {e}");
                                    }
                                    let leaderboard = self.load_leaderboard().await?;
                                    let active = self.build_active_runs_props(&active_runs);
                                    ui.set_global_state(BombGlobalProps { leaderboard, active_runs: active });
                                }
                            }
                        },
                        insim::Packet::Cnl(Cnl { ucid, .. }) => {
                            // Connection left — look up connection
                            if let Some(conn) = self.presence.connection(&ucid).await.map_err(|cause| SceneError::Custom {
                                scene: "bomb::cnl::connection",
                                cause: Box::new(cause),
                            })? {
                                if let Some(run) = active_runs.remove(&conn.uname) {
                                    let now = Instant::now();
                                    let survival_ms = (run.deadline.min(now) - run.started_at).as_millis() as i64;
                                    if let Err(e) = db::insert_bomb_run(
                                        &self.db,
                                        self.session_id,
                                        &conn.uname,
                                        &run.vehicle.to_string(),
                                        run.checkpoints,
                                        survival_ms,
                                    ).await {
                                        tracing::warn!("Failed to persist bomb run on disconnect: {e}");
                                    }
                                    let leaderboard = self.load_leaderboard().await?;
                                    let active = self.build_active_runs_props(&active_runs);
                                    ui.set_global_state(BombGlobalProps { leaderboard, active_runs: active });
                                }
                            }
                        },
                        insim::Packet::Uco(Uco {
                            info: ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }),
                            plid,
                            ..
                        }) => {
                            if let Some(player) = self.presence.player(&plid).await.map_err(|cause| SceneError::Custom {
                                scene: "bomb::uco::player",
                                cause: Box::new(cause),
                            })?
                                && !player.ptype.is_ai()
                                && let Some(conn) = self.presence.connection_by_player(&plid).await.map_err(|cause| SceneError::Custom {
                                    scene: "bomb::uco::connection",
                                    cause: Box::new(cause),
                                })?
                            {
                                let now = Instant::now();
                                let uname = conn.uname.clone();
                                let ucid = conn.ucid;
                                let pname = conn.pname.clone();
                                let vehicle = player.vehicle;

                                let run = active_runs.entry(uname.clone()).or_insert_with(|| {
                                    ActiveRun {
                                        started_at: now,
                                        deadline: now + self.checkpoint_timeout,
                                        current_timeout: self.checkpoint_timeout,
                                        checkpoints: 0,
                                        vehicle,
                                        uname: uname.clone(),
                                        pname: pname.clone(),
                                        ucid,
                                    }
                                });

                                let remaining = run.deadline.saturating_duration_since(now);

                                // Checkpoint1: carry over remaining time as the next window.
                                // Checkpoint2/3/Finish: refresh — top up to the 1/4 base window.
                                // A refresh checkpoint placed after a hard section rewards survival.
                                let (new_timeout, is_refresh) = match kind {
                                    InsimCheckpointKind::Checkpoint1 => (remaining, false),
                                    InsimCheckpointKind::Checkpoint2
                                    | InsimCheckpointKind::Checkpoint3
                                    | InsimCheckpointKind::Finish => {
                                        (remaining + (self.checkpoint_timeout / 4), true)
                                    },
                                };

                                let is_refresh = true;
                                let new_timeout = self.checkpoint_timeout;

                                run.deadline = now + new_timeout;
                                run.current_timeout = new_timeout;
                                run.checkpoints += 1;
                                let n = run.checkpoints;
                                let new_timeout_secs = new_timeout.as_secs_f64();

                                let msg = if is_refresh {
                                    format!("REFRESH — checkpoint {n} — {new_timeout_secs:.1}s").yellow()
                                } else {
                                    format!("checkpoint {n} — {new_timeout_secs:.1}s").light_green()
                                };
                                self.insim.send_message(msg, ucid).await?;

                                let active = self.build_active_runs_props(&active_runs);
                                let leaderboard = self.load_leaderboard().await?;
                                ui.set_global_state(BombGlobalProps { leaderboard, active_runs: active });
                                ui.set_player_state(ucid, BombConnectionProps {
                                    uname: uname.clone(),
                                    in_run: true,
                                }).await;
                            }
                        },
                        _ => {},
                    }
                },

                _ = tick.tick() => {
                    let now = Instant::now();
                    let expired: Vec<String> = active_runs
                        .iter()
                        .filter(|(_, run)| run.deadline < now)
                        .map(|(k, _)| k.clone())
                        .collect();

                    for uname in expired {
                        if let Some(run) = active_runs.remove(&uname) {
                            self.insim
                                .send_command(format!("/spec {}", uname))
                                .await?;

                            let survival_ms = (run.deadline - run.started_at).as_millis() as i64;
                            let n = run.checkpoints;
                            let survival_secs = survival_ms as f64 / 1000.0;

                            self.insim
                                .send_message(
                                    format!("BOOM — {n} checkpoints, {survival_secs:.1}s").red(),
                                    run.ucid,
                                )
                                .await?;
                            ui.set_player_state(run.ucid, BombConnectionProps {
                                uname: uname.clone(),
                                in_run: false,
                            }).await;

                            if let Err(e) = db::insert_bomb_run(
                                &self.db,
                                self.session_id,
                                &uname,
                                &run.vehicle.to_string(),
                                n,
                                survival_ms,
                            ).await {
                                tracing::warn!("Failed to persist bomb run: {e}");
                            }

                            let leaderboard = self.load_leaderboard().await?;
                            let active = self.build_active_runs_props(&active_runs);
                            ui.set_global_state(BombGlobalProps { leaderboard, active_runs: active });
                        }
                    }
                },

                _ = self.chat.wait_for_admin_cmd(self.presence.clone(), |msg| matches!(msg, chat::BombChatMsg::End)) => {
                    tracing::info!("Admin ended bomb session");
                    return Ok(SceneResult::Continue(()));
                },

                _ = self.game.wait_for_end() => {
                    tracing::info!("Game ended");
                    return Ok(SceneResult::Continue(()));
                },
            }
        }
    }

    async fn load_leaderboard(&self) -> Result<BombLeaderboard, SceneError> {
        let rows = db::bomb_best_runs(&self.db, self.session_id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "bomb::load_leaderboard",
                cause: Box::new(cause),
            })?;

        Ok(rows
            .into_iter()
            .map(|row| (row.uname, row.pname, row.checkpoint_count, row.survival_ms))
            .collect::<Vec<_>>()
            .into())
    }

    fn build_active_runs_props(
        &self,
        active_runs: &HashMap<String, ActiveRun>,
    ) -> Vec<(String, String, i64, Instant, Duration)> {
        let mut runs: Vec<_> = active_runs
            .values()
            .map(|run| {
                (
                    run.uname.clone(),
                    run.pname.clone(),
                    run.checkpoints,
                    run.deadline,
                    run.current_timeout,
                )
            })
            .collect();
        runs.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)));
        runs
    }
}
