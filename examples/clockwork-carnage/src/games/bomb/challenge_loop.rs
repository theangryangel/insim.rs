use std::time::{Duration, Instant};

use insim::{
    Colour,
    builder::InsimTask,
    core::object::insim::{InsimCheckpoint, InsimCheckpointKind},
    insim::{Cnl, Con, Crs, ObjectInfo, Pit, Pll, Plp, Toc, Uco},
};
use kitcar::{
    game, presence,
    scenes::{FromContext, Scene, SceneError, SceneResult},
    ui::{self, Component},
};

use super::{chat, state};
use crate::{
    db,
    hud::{
        BombLeaderboard, Dialog, DialogMsg, DialogProps, bomb_scoreboard,
        theme::{hud_active, hud_muted, hud_text, hud_title},
        topbar,
    },
};

const BOMB_HELP_LINES: &[&str] = &[
    " - Hit ^2checkpoint^7 objects before the timer expires or your run ends (BOOM).",
    " - Each checkpoint shrinks the window: ^1next window = current window - penalty^7.",
    " - Hit a ^3finish^7 object to fully reset the window back to the base time.",
    " - ^1Resetting your car^7 deducts the penalty directly from your remaining time.",
    " - ^1Pitting^7 ends your run immediately — commit to your fuel before you start.",
    " - ^1Collisions^7 cost time — harder impacts cost more, up to the collision max penalty.",
    " - Score = checkpoints hit. Survival time breaks ties.",
    " - Your best run is recorded on the leaderboard.",
    "",
    "Good luck.",
];

#[derive(Debug, Clone, Default)]
struct BombGlobalProps {
    leaderboard: BombLeaderboard,
    active_runs: Vec<(String, String, i64, Instant, Duration)>, // (uname, pname, cps, deadline, current_timeout)
    event_url: Option<String>,
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
                    ui::text(time_str, style.align_right()).w(10.).h(5.),
                    ui::text(bar, style).w(10.).h(5.),
                ])
            })
            .collect();

        let mut scoreboard = ui::container()
            .flex()
            .pl(5.)
            .w(200.)
            .mt(10.)
            .flex_col()
            .items_start()
            .with_child(ui::text("Active Runs", hud_title()).w(43.).h(5.))
            .with_children(active_run_rows)
            .with_child(ui::text("Session Best", hud_title()).w(43.).h(5.))
            .with_children(leaderboard_rows);

        if let Some(url) = &props.global.event_url {
            scoreboard =
                scoreboard.with_child(ui::text(url, hud_muted().align_left()).w(43.).h(5.));
        }

        ui::container()
            .flex()
            .flex_col()
            .with_child(topbar("Bomb").with_child(ui::text(status_str, status_style).w(15.).h(5.)))
            .with_child(scoreboard)
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

#[derive(Clone)]
pub struct BombLoop {
    pub chat: chat::BombChat,
    pub session_id: i64,
    pub checkpoint_timeout: Duration,
    pub checkpoint_penalty: Duration,
    pub collision_max_penalty: Duration,
    pub base_url: Option<String>,
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
        let insim = InsimTask::from_context(ctx);
        let mut game = game::Game::from_context(ctx);
        let presence = presence::Presence::from_context(ctx);
        let pool = db::Pool::from_context(ctx);

        let (ui, _ui_handle) = ui::mount_with(
            insim.clone(),
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

        let event_url = self
            .base_url
            .as_deref()
            .map(|base| format!("{}/event/{}", base.trim_end_matches('/'), self.session_id));

        let config = state::BombConfig {
            checkpoint_timeout: self.checkpoint_timeout,
            checkpoint_penalty: self.checkpoint_penalty,
            collision_max_penalty: self.collision_max_penalty,
        };
        let mut state = state::BombState::new(config, BombLeaderboard::default());
        reload_leaderboard(&pool, self.session_id, &mut state).await?;

        ui.set_global_state(BombGlobalProps {
            leaderboard: state.leaderboard.clone(),
            active_runs: vec![],
            event_url: event_url.clone(),
        });

        let mut packets = insim.subscribe();
        let mut tick = tokio::time::interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    let packet = packet.map_err(|_| SceneError::InsimHandleLost)?;
                    match packet {
                        insim::Packet::Ncn(ncn) => {
                            insim
                                .send_message(
                                    format!(
                                        "Welcome to Bomb Mode! Hit checkpoints before {}s timer expires.",
                                        self.checkpoint_timeout.as_secs()
                                    ),
                                    ncn.ucid,
                                )
                                .await?;

                            if let Some(conn) = presence.connection(&ncn.ucid).await.map_err(|cause| SceneError::Custom {
                                scene: "bomb::ncn::connection",
                                cause: Box::new(cause),
                            })? {
                                ui.set_player_state(ncn.ucid, BombConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_run: false,
                                }).await;
                            }
                        },
                        insim::Packet::Pll(Pll { plid, .. }) | insim::Packet::Plp(Plp { plid, .. }) => {
                            if let Some(res) = state.on_leave(plid) {
                                let now = Instant::now();
                                let survival_ms = res.run.survival_ms(now);
                                let msg = format!("Run ended — left race after {} checkpoints.", res.run.checkpoints).red();
                                insim.send_message(msg, res.run.ucid).await?;
                                persist_run(&pool, self.session_id, &mut state, &res.run, survival_ms).await?;
                                let active = state.active_runs_props();
                                ui.set_global_state(BombGlobalProps { leaderboard: state.leaderboard.clone(), active_runs: active, event_url: event_url.clone() });
                                ui.set_player_state(res.run.ucid, BombConnectionProps {
                                    uname: res.run.uname.clone(),
                                    in_run: false,
                                }).await;
                            }
                        },
                        insim::Packet::Toc(Toc { plid, newucid, .. }) => {
                            state.on_toc(plid, newucid);
                        },
                        insim::Packet::Cnl(Cnl { ucid, .. }) => {
                            if let Some(res) = state.on_leave_by_ucid(ucid) {
                                let survival_ms = res.run.survival_ms(Instant::now());
                                persist_run(&pool, self.session_id, &mut state, &res.run, survival_ms).await?;
                                let active = state.active_runs_props();
                                ui.set_global_state(BombGlobalProps { leaderboard: state.leaderboard.clone(), active_runs: active, event_url: event_url.clone() });
                            }
                        },
                        insim::Packet::Crs(Crs { plid, .. }) => {
                            if let Some(res) = state.on_reset(plid, Instant::now()) {
                                insim.send_message(
                                    format!(
                                        "PENALTY — -{:.2}s — {:.1}s left",
                                        res.penalty.as_secs_f64(),
                                        res.time_left.as_secs_f64()
                                    ).red(),
                                    res.ucid,
                                ).await?;
                                let active = state.active_runs_props();
                                ui.set_global_state(BombGlobalProps { leaderboard: state.leaderboard.clone(), active_runs: active, event_url: event_url.clone() });
                            }
                        },
                        insim::Packet::Pit(Pit { plid, .. }) => {
                            if let Some(res) = state.on_pit(plid) {
                                let now = Instant::now();
                                let survival_ms = res.run.survival_ms(now);
                                let msg = format!("PITTED — run ended after {} checkpoints. Commit to your fuel before the run.", res.run.checkpoints).red();
                                insim.send_message(msg, res.run.ucid).await?;
                                insim.send_command(format!("/spec {}", res.run.uname)).await?;
                                persist_run(&pool, self.session_id, &mut state, &res.run, survival_ms).await?;
                                let active = state.active_runs_props();
                                ui.set_global_state(BombGlobalProps { leaderboard: state.leaderboard.clone(), active_runs: active, event_url: event_url.clone() });
                                ui.set_player_state(res.run.ucid, BombConnectionProps {
                                    uname: res.run.uname.clone(),
                                    in_run: false,
                                }).await;
                            }
                        },
                        insim::Packet::Con(Con { spclose, a, b, .. }) => {
                            let now = Instant::now();
                            for plid in [a.plid, b.plid] {
                                if let Some(res) = state.on_collision(plid, spclose.to_meters_per_sec(), now) {
                                    insim.send_message(
                                        format!(
                                            "PENALTY — -{:.2}s — {:.1}s left",
                                            res.penalty.as_secs_f64(),
                                            res.time_left.as_secs_f64()
                                        ).red(),
                                        res.ucid,
                                    ).await?;
                                    let active = state.active_runs_props();
                                    ui.set_global_state(BombGlobalProps { leaderboard: state.leaderboard.clone(), active_runs: active, event_url: event_url.clone() });
                                }
                            }
                        },
                        insim::Packet::Uco(Uco {
                            info: ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }),
                            plid,
                            ..
                        }) => {
                             if let Some(player) = presence.player(&plid).await.map_err(|cause| SceneError::Custom {
                                scene: "bomb::uco::player",
                                cause: Box::new(cause),
                            })?
                                && !player.ptype.is_ai()
                                && let Some(conn) = presence.connection_by_player(&plid).await.map_err(|cause| SceneError::Custom {
                                    scene: "bomb::uco::connection",
                                    cause: Box::new(cause),
                                })?
                            {
                                if let Some(res) = state.on_checkpoint(
                                    conn.uname.clone(),
                                    conn.pname.clone(),
                                    conn.ucid,
                                    plid,
                                    player.vehicle,
                                    matches!(kind, InsimCheckpointKind::Finish),
                                    Instant::now(),
                                ) {
                                    match res {
                                        state::CheckpointResult::Refreshed { ucid, checkpoints, new_window } => {
                                            let new_secs = new_window.as_secs_f64();
                                            insim.send_message(
                                                format!("FINISH — checkpoint {checkpoints} — REFRESHED {new_secs:.1}s").yellow(),
                                                ucid
                                            ).await?;
                                        },
                                        state::CheckpointResult::Extended { ucid, checkpoints, penalty, time_left } => {
                                            let new_secs = time_left.as_secs_f64();
                                            insim.send_message(
                                                format!(
                                                    "checkpoint {checkpoints} — -{:.2}s — {new_secs:.1}s left",
                                                    penalty.as_secs_f64()
                                                )
                                                .light_green(),
                                                ucid
                                            ).await?;
                                        },
                                        state::CheckpointResult::Started { ucid } => {
                                            ui.set_player_state(ucid, BombConnectionProps {
                                                uname: conn.uname.clone(),
                                                in_run: true,
                                            }).await;
                                        }
                                    }

                                    reload_leaderboard(&pool, self.session_id, &mut state).await?;
                                    let active = state.active_runs_props();
                                    ui.set_global_state(BombGlobalProps { leaderboard: state.leaderboard.clone(), active_runs: active, event_url: event_url.clone() });
                                }
                            }
                        },
                        _ => {},
                    }
                },

                _ = tick.tick() => {
                    let now = Instant::now();
                    for res in state.tick(now) {
                        let survival_ms = res.run.survival_ms(now);
                        let n = res.run.checkpoints;
                        let survival_secs = survival_ms as f64 / 1000.0;
                        let msg = format!("BOOM — {n} checkpoints, {survival_secs:.1}s").red();
                        insim.send_message(msg, res.run.ucid).await?;
                        insim.send_command(format!("/spec {}", res.run.uname)).await?;
                        persist_run(&pool, self.session_id, &mut state, &res.run, survival_ms).await?;
                        let active = state.active_runs_props();
                        ui.set_global_state(BombGlobalProps { leaderboard: state.leaderboard.clone(), active_runs: active, event_url: event_url.clone() });
                        ui.set_player_state(res.run.ucid, BombConnectionProps {
                            uname: res.run.uname.clone(),
                            in_run: false,
                        }).await;
                    }
                },

                _ = self.chat.wait_for_admin_cmd(presence.clone(), |msg| matches!(msg, chat::BombChatMsg::End)) => {
                    tracing::info!("Admin ended bomb session");
                    return Ok(SceneResult::Continue(()));
                },

                _ = game.wait_for_end() => {
                    tracing::info!("Game ended");
                    return Ok(SceneResult::Continue(()));
                },
            }
        }
    }
}

async fn reload_leaderboard(
    pool: &db::Pool,
    session_id: i64,
    state: &mut state::BombState,
) -> Result<(), SceneError> {
    let rows = db::bomb_best_runs(pool, session_id)
        .await
        .map_err(|cause| SceneError::Custom {
            scene: "bomb::reload_leaderboard",
            cause: Box::new(cause),
        })?;
    state.leaderboard = rows
        .into_iter()
        .map(|r| (r.uname, r.pname, r.checkpoint_count, r.survival_ms))
        .collect::<Vec<_>>()
        .into();
    Ok(())
}

async fn persist_run(
    pool: &db::Pool,
    session_id: i64,
    state: &mut state::BombState,
    run: &state::ActiveRun,
    survival_ms: i64,
) -> Result<(), SceneError> {
    if let Err(e) = db::insert_bomb_run(
        pool,
        session_id,
        &run.uname,
        &run.vehicle.to_string(),
        run.checkpoints,
        survival_ms,
    )
    .await
    {
        tracing::warn!("Failed to persist bomb run: {e}");
    }
    reload_leaderboard(pool, session_id, state).await
}
