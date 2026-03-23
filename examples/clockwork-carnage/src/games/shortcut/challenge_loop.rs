use std::{collections::HashMap, time::Duration};

use insim::{
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colour,
        vehicle::Vehicle,
    },
    identifiers::ConnectionId,
    insim::{ObjectInfo, Pll, Plp, Uco},
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
        ChallengeLeaderboard, Dialog, DialogMsg, DialogProps, challenge_scoreboard,
        theme::{hud_active, hud_muted, hud_overlay_action, hud_overlay_text, hud_panel_bg, hud_text, hud_title},
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

const ALT_MAX: f32 = u8::MAX as f32;
const ALT_BAR_LEN: usize = 18;

#[derive(Debug, Clone, Default)]
struct ChallengeGlobalProps {
    leaderboard: ChallengeLeaderboard,
    /// (pname, altitude) for all players currently on track.
    altitudes: Vec<(String, f32)>,
    /// Full URL to this event's results page, if a base URL is configured.
    event_url: Option<String>,
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
    Altitude(DialogMsg),
}

struct ChallengeView {
    help_dialog: Dialog,
    altitude_dialog: Dialog,
}

#[derive(Debug, Clone, Default)]
struct ChallengeProps {
    global: ChallengeGlobalProps,
    connection: ChallengeConnectionProps,
}

fn render_altitude_overlay(altitudes: &[(String, f32)]) -> ui::Node<ChallengeMessage> {
    let mut sorted = altitudes.to_vec();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let rows: Vec<ui::Node<ChallengeMessage>> = sorted
        .iter()
        .map(|(pname, height)| {
            let h = height.clamp(0.0, ALT_MAX);
            let filled = (h / ALT_MAX * ALT_BAR_LEN as f32).round() as usize;
            let bar = format!(
                "[{}{}]",
                "#".repeat(filled),
                " ".repeat(ALT_BAR_LEN - filled)
            );
            let text = format!("{}  {}  {:3.0}m", pname, bar, h);
            ui::text(text, hud_overlay_text().align_left().white())
                .w_auto()
                .h(6.)
        })
        .collect();

    let content = if rows.is_empty() {
        vec![ui::text("No players on track.", hud_overlay_text().align_left().white())
            .w_auto()
            .h(6.)]
    } else {
        rows
    };

    ui::container()
        .flex()
        .flex_col()
        .justify_center()
        .items_center()
        .w(200.)
        .h(200.)
        .with_child(
            ui::container()
                .flex()
                .flex_col()
                .with_child(
                    ui::background(hud_panel_bg())
                        .w(100.)
                        .flex()
                        .flex_col()
                        .p(1.)
                        .with_child(
                            ui::text("Altitude Tracker", hud_overlay_text().align_left().yellow())
                                .h(8.)
                                .mb(2.)
                                .w_auto(),
                        )
                        .with_children(content),
                )
                .with_child(
                    ui::clickable(
                        "Close",
                        hud_overlay_action().green().dark(),
                        ChallengeMessage::Altitude(DialogMsg::Hide),
                    )
                    .self_end()
                    .w(12.)
                    .h(8.)
                    .mt(2.)
                    .key("alt-close"),
                ),
        )
}

impl ui::Component for ChallengeView {
    type Props<'a> = ChallengeProps;
    type Message = ChallengeMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            ChallengeMessage::Help(m) => Component::update(&mut self.help_dialog, m),
            ChallengeMessage::Altitude(m) => Component::update(&mut self.altitude_dialog, m),
        }
    }

    fn render(&self, props: Self::Props<'_>) -> ui::Node<Self::Message> {
        if self.help_dialog.is_visible() {
            return self
                .help_dialog
                .render(DialogProps {
                    title: "Shortcut",
                    lines: CHALLENGE_HELP_LINES,
                })
                .map(ChallengeMessage::Help);
        }

        if self.altitude_dialog.is_visible() {
            return render_altitude_overlay(&props.global.altitudes);
        }

        let (status, status_style) = if props.connection.in_progress {
            ("In progress".to_string(), hud_active())
        } else {
            match props.connection.best_time {
                Some(d) => (
                    format!("PB: {}", crate::hud::format_duration(d)),
                    hud_text(),
                ),
                None => ("Waiting for start".to_string(), hud_muted()),
            }
        };

        let players = challenge_scoreboard(&props.global.leaderboard, &props.connection.uname);

        let mut scoreboard = ui::container()
            .flex()
            .pl(5.)
            .w(200.)
            .mt(10.)
            .flex_col()
            .items_start()
            .with_child(ui::text("Best Times", hud_title()).w(40.).h(5.))
            .with_children(players);

        if let Some(url) = &props.global.event_url {
            scoreboard = scoreboard.with_child(
                ui::text(url, hud_muted().align_left())
                    .w(40.)
                    .h(5.),
            );
        }

        ui::container()
            .flex()
            .flex_col()
            .w(200.)
            .with_child(topbar("Shortcut").with_child(ui::text(status, status_style).w(20.).h(5.)))
            .with_child(scoreboard)
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

fn build_altitudes(
    heights: &HashMap<ConnectionId, f32>,
    names: &HashMap<ConnectionId, String>,
) -> Vec<(String, f32)> {
    heights
        .iter()
        .filter_map(|(ucid, &h)| names.get(ucid).map(|n| (n.clone(), h)))
        .collect()
}

/// Challenge mode — runs indefinitely, players compete for fastest time.
#[derive(Clone)]
pub struct ChallengeLoop {
    pub chat: chat::ChallengeChat,
    pub session_id: i64,
    pub base_url: Option<String>,
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

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        let inner = ChallengeLoopInner {
            insim: InsimTask::from_context(ctx),
            game: game::Game::from_context(ctx),
            presence: presence::Presence::from_context(ctx),
            db: db::Pool::from_context(ctx),
            chat: self.chat,
            session_id: self.session_id,
            base_url: self.base_url,
        };
        inner.run_inner().await
    }
}

/// Internal struct combining config and extracted infrastructure.
struct ChallengeLoopInner {
    insim: InsimTask,
    game: game::Game,
    presence: presence::Presence,
    db: db::Pool,
    chat: chat::ChallengeChat,
    session_id: i64,
    base_url: Option<String>,
}

impl ChallengeLoopInner {
    async fn run_inner(mut self) -> Result<SceneResult<()>, SceneError> {
        let (ui, _ui_handle) = ui::mount_with(
            self.insim.clone(),
            ChallengeGlobalProps::default(),
            |_ucid, _invalidator| ChallengeView {
                help_dialog: Dialog::default(),
                altitude_dialog: Dialog::default(),
            },
            self.chat.subscribe(),
            |(ucid, msg)| match msg {
                chat::ChallengeChatMsg::Help => {
                    Some((ucid, ChallengeMessage::Help(DialogMsg::Show)))
                },
                chat::ChallengeChatMsg::Alt => {
                    Some((ucid, ChallengeMessage::Altitude(DialogMsg::Show)))
                },
                _ => None,
            },
        );

        // Load initial leaderboard from DB
        let event_url = self
            .base_url
            .as_deref()
            .map(|base| format!("{}/event/{}", base.trim_end_matches('/'), self.session_id));
        let mut current_leaderboard = self.challenge_leaderboard().await?;
        ui.set_global_state(ChallengeGlobalProps {
            leaderboard: current_leaderboard.clone(),
            altitudes: vec![],
            event_url: event_url.clone(),
        });

        let mut active_runs: HashMap<String, Duration> = HashMap::new();
        let mut plid_to_uname: HashMap<insim::identifiers::PlayerId, String> = HashMap::new();
        let mut player_heights: HashMap<ConnectionId, f32> = HashMap::new();
        let mut player_names: HashMap<ConnectionId, String> = HashMap::new();
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                packet = packets.recv() => {
                    let packet = packet.map_err(|_| SceneError::InsimHandleLost)?;
                    match packet {
                        insim::Packet::Ncn(ncn) => {
                            self.insim
                                .send_message("Welcome to the Shortcut! Compete for the fastest time.", ncn.ucid)
                                .await?;

                            if let Some(conn) = self.presence.connection(&ncn.ucid).await.map_err(|cause| SceneError::Custom {
                                scene: "challenge::ncn::connection",
                                cause: Box::new(cause),
                            })? {
                                let _ = player_names.insert(ncn.ucid, conn.pname.clone());
                                let pb = self.personal_best(&conn.uname).await?;
                                ui.set_player_state(ncn.ucid, ChallengeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: false,
                                    best_time: pb,
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
                                        let _ = plid_to_uname.insert(plid, conn.uname.clone());
                                    },
                                    InsimCheckpointKind::Finish => {
                                        let _ = plid_to_uname.remove(&plid);
                                        if let Some(start) = active_runs.remove(&conn.uname) {
                                            let lap_time = time.saturating_sub(start);
                                            let vehicle = player.vehicle;

                                            let prev_pb = self.personal_best(&conn.uname).await?;
                                            let is_pb = match prev_pb {
                                                Some(prev) => lap_time < prev,
                                                None => true,
                                            };

                                            // Persist every run to DB
                                            let time_ms = lap_time.as_millis() as i64;
                                            if let Err(e) = db::insert_shortcut_time(
                                                &self.db,
                                                self.session_id,
                                                &conn.uname,
                                                &vehicle.to_string(),
                                                time_ms,
                                            )
                                            .await
                                            {
                                                tracing::warn!("Failed to persist challenge time: {e}");
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
                                            } else if let Some(pb) = prev_pb {
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

                                            // Update leaderboard from DB
                                            current_leaderboard = self.challenge_leaderboard().await?;
                                            ui.set_global_state(ChallengeGlobalProps {
                                                leaderboard: current_leaderboard.clone(),
                                                altitudes: build_altitudes(&player_heights, &player_names),
                                                event_url: event_url.clone(),
                                            });
                                        }
                                    },
                                    _ => {},
                                }

                                let pb = self.personal_best(&conn.uname).await?;
                                ui.set_player_state(conn.ucid, ChallengeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: active_runs.contains_key(&conn.uname),
                                    best_time: pb,
                                }).await;
                            }
                        },
                        insim::Packet::Pll(Pll { plid, .. }) | insim::Packet::Plp(Plp { plid, .. }) => {
                            if let Some(uname) = plid_to_uname.remove(&plid) {
                                let _ = active_runs.remove(&uname);
                            }
                        },
                        insim::Packet::Cnl(cnl) => {
                            let _ = player_heights.remove(&cnl.ucid);
                            if let Some(uname) = player_names.remove(&cnl.ucid) {
                                let _ = active_runs.remove(&uname);
                            }
                            ui.set_global_state(ChallengeGlobalProps {
                                leaderboard: current_leaderboard.clone(),
                                altitudes: build_altitudes(&player_heights, &player_names),
                                event_url: event_url.clone(),
                            });
                        },
                        insim::Packet::Mci(mci) => {
                            for car in &mci.info {
                                if let Some(conn) = self.presence.connection_by_player(&car.plid).await.map_err(|cause| SceneError::Custom {
                                    scene: "challenge::mci::connection_by_player",
                                    cause: Box::new(cause),
                                })? {
                                    let _ = player_heights.insert(conn.ucid, car.xyz.z_metres());
                                }
                            }
                            ui.set_global_state(ChallengeGlobalProps {
                                leaderboard: current_leaderboard.clone(),
                                altitudes: build_altitudes(&player_heights, &player_names),
                                event_url: event_url.clone(),
                            });
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

    async fn challenge_leaderboard(&self) -> Result<ChallengeLeaderboard, SceneError> {
        let rows = db::shortcut_best_times(&self.db, self.session_id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "challenge::challenge_leaderboard",
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
        let row = db::shortcut_personal_best(&self.db, self.session_id, uname)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "challenge::personal_best",
                cause: Box::new(cause),
            })?;

        Ok(row.map(|r| Duration::from_millis(r.time_ms as u64)))
    }
}
