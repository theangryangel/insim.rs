use std::{collections::HashMap, time::Duration};

use insim::{
    builder::InsimTask,
    core::{
        object::insim::{InsimCheckpoint, InsimCheckpointKind},
        string::colours::Colour,
    },
    identifiers::{ConnectionId, PlayerId},
    insim::{ObjectInfo, Uco},
};
use insim_extras::{
    presence,
    presence::PresenceEvent,
    scenes::{FromContext, IntoSceneError as _, Scene, SceneError, SceneResult},
    ui::{self, Component},
};

use super::chat;
use crate::{
    db,
    hud::{
        Dialog, DialogMsg, DialogProps, MetronomeLeaderboard, metronome_scoreboard,
        theme::{hud_active, hud_muted, hud_text, hud_title},
        topbar,
    },
};

const METRONOME_HELP_LINES: &[&str] = &[
    " - Cross checkpoint 1 to start your timed attempt.",
    " - Reach the finish to record your delta from the target.",
    " - The smallest delta wins.",
    " - Retry as many times as you like - no spec, no limit.",
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
    event_url: Option<String>,
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

impl ui::Component for MetronomeView {
    type Props<'a> = (&'a MetronomeGlobalProps, &'a MetronomeConnectionProps);
    type Message = MetronomeMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            MetronomeMessage::Help(help_msg) => {
                Component::update(&mut self.help_dialog, help_msg);
            },
        }
    }

    fn render(&self, (global, player): Self::Props<'_>) -> ui::Node<Self::Message> {
        if self.help_dialog.is_visible() {
            return self
                .help_dialog
                .render(DialogProps {
                    title: "Clockwork Carnage",
                    lines: METRONOME_HELP_LINES,
                })
                .map(MetronomeMessage::Help);
        }

        let (status, status_style) = if player.in_progress {
            ("In progress".to_string(), hud_active())
        } else {
            match player.best_delta {
                Some(d) => {
                    let tier = tier_label(d).unwrap_or("No tier");
                    (
                        format!("Best: {} [{}]", crate::hud::format_duration(d), tier),
                        hud_text(),
                    )
                },
                None => ("Waiting for start".to_string(), hud_muted()),
            }
        };

        let players = metronome_scoreboard(&global.leaderboard, &player.uname);

        let mut scoreboard = ui::container()
            .flex()
            .pl(5.)
            .w(200.)
            .mt(10.)
            .flex_col()
            .items_start()
            .with_child(ui::text("Best Deltas", hud_title()).w(35.).h(5.))
            .with_children(players);

        if let Some(url) = &global.event_url {
            scoreboard =
                scoreboard.with_child(ui::text(url, hud_muted().align_left()).w(35.).h(5.));
        }

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar(&format!("Target: {:.2?}", global.target))
                    .with_child(ui::text(status, status_style).w(20.).h(5.)),
            )
            .with_child(scoreboard)
    }
}

/// Open-format metronome - runs indefinitely, players compete for closest delta to target.
#[derive(Clone)]
pub struct ChallengeLoop {
    pub chat: chat::EventChat,
    pub target: Duration,
    pub session_id: i64,
    pub base_url: Option<String>,
}

impl<Ctx> Scene<Ctx> for ChallengeLoop
where
    InsimTask: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    db::Pool: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError> {
        let inner = ChallengeLoopInner {
            insim: InsimTask::from_context(ctx),
            presence: presence::Presence::from_context(ctx),
            db: db::Pool::from_context(ctx),
            chat: self.chat,
            target: self.target,
            session_id: self.session_id,
            base_url: self.base_url,
        };
        inner.run_inner().await
    }
}

struct ChallengeLoopInner {
    insim: InsimTask,
    presence: presence::Presence,
    db: db::Pool,
    chat: chat::EventChat,
    target: Duration,
    session_id: i64,
    base_url: Option<String>,
}

impl ChallengeLoopInner {
    async fn run_inner(self) -> Result<SceneResult<()>, SceneError> {
        let event_url = self
            .base_url
            .as_deref()
            .map(|base| format!("{}/event/{}", base.trim_end_matches('/'), self.session_id));

        let (ui, _ui_handle) = ui::mount_with(
            self.insim.clone(),
            MetronomeGlobalProps {
                target: self.target,
                leaderboard: self.metronome_leaderboard().await?,
                event_url: event_url.clone(),
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

        // Subscribe before seeding to avoid missing events during the queries.
        let mut events = self.presence.subscribe_events();
        let mut connections: HashMap<ConnectionId, presence::ConnectionInfo> = self
            .presence
            .connections()
            .await
            .scene_err("metronome::init::connections")?
            .into_iter()
            .map(|c| (c.ucid, c))
            .collect();
        let mut players: HashMap<PlayerId, presence::PlayerInfo> = self
            .presence
            .players()
            .await
            .scene_err("metronome::init::players")?
            .into_iter()
            .map(|p| (p.plid, p))
            .collect();

        let mut active_runs: HashMap<String, Duration> = HashMap::new();
        let mut plid_to_uname: HashMap<PlayerId, String> = HashMap::new();
        let mut packets = self.insim.subscribe();

        loop {
            tokio::select! {
                event = events.recv() => {
                    match event {
                        Ok(PresenceEvent::Connected(info)) => {
                            self.insim
                                .send_message(
                                    "Metronome. Match the target time as closely as possible.",
                                    info.ucid,
                                )
                                .await?;
                            let pb = self.personal_best(&info.uname).await?;
                            ui.assign_to(info.ucid, MetronomeConnectionProps {
                                uname: info.uname.clone(),
                                in_progress: active_runs.contains_key(&info.uname),
                                best_delta: pb,
                            }).await;
                            let _ = connections.insert(info.ucid, info);
                        },
                        Ok(PresenceEvent::Disconnected(info)) => {
                            let _ = connections.remove(&info.ucid);
                            // active_runs already cleaned up by preceding PlayerLeft events.
                        },
                        Ok(PresenceEvent::PlayerJoined(info)) => {
                            let _ = players.insert(info.plid, info);
                        },
                        Ok(PresenceEvent::PlayerLeft(info))
                        | Ok(PresenceEvent::PlayerTeleportedToPits(info)) => {
                            let _ = players.remove(&info.plid);
                            if let Some(uname) = plid_to_uname.remove(&info.plid) {
                                let _ = active_runs.remove(&uname);
                            }
                        },
                        Ok(PresenceEvent::TakingOver { before, after }) => {
                            let _ = players.remove(&before.plid);
                            let _ = players.insert(after.plid, after);
                        },
                        Ok(PresenceEvent::Renamed { .. })
                        | Ok(PresenceEvent::ConnectionDetails(_))
                        | Ok(PresenceEvent::VehicleSelected { .. })
                        | Err(_) => {},
                    }
                },

                packet = packets.recv() => {
                    let packet = packet.map_err(|_| SceneError::InsimHandleLost)?;
                    if let insim::Packet::Uco(Uco {
                        info:
                            ObjectInfo::InsimCheckpoint(InsimCheckpoint {
                                kind:
                                    kind @ (InsimCheckpointKind::Checkpoint1 | InsimCheckpointKind::Finish),
                                ..
                            }),
                        plid,
                        time,
                        ..
                    }) = packet
                        && let Some(player) = players.get(&plid).cloned()
                        && !player.ptype.is_ai()
                        && let Some(conn) = connections.get(&player.ucid).cloned()
                    {
                                match kind {
                                    InsimCheckpointKind::Checkpoint1 => {
                                        let _ = active_runs.insert(conn.uname.clone(), time);
                                        let _ = plid_to_uname.insert(plid, conn.uname.clone());
                                    },
                                    InsimCheckpointKind::Finish => {
                                        let _ = plid_to_uname.remove(&plid);
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

                                            self.presence.spec(conn.ucid).await.scene_err("metronome::spec")?;

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
                                            ui.assign(MetronomeGlobalProps {
                                                target: self.target,
                                                leaderboard,
                                                event_url: event_url.clone(),
                                            });
                                        }
                                    },
                                    _ => {},
                                }

                                let pb = self.personal_best(&conn.uname).await?;
                                ui.assign_to(conn.ucid, MetronomeConnectionProps {
                                    uname: conn.uname.clone(),
                                    in_progress: active_runs.contains_key(&conn.uname),
                                    best_delta: pb,
                                }).await;
                    }
                },

            }
        }
    }

    async fn metronome_leaderboard(&self) -> Result<MetronomeLeaderboard, SceneError> {
        let rows = db::metronome_standings(&self.db, self.session_id)
            .await
            .scene_err("metronome::metronome_leaderboard")?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.uname,
                    row.pname,
                    Duration::from_millis(row.best_delta_ms as u64),
                )
            })
            .collect::<Vec<_>>()
            .into())
    }

    async fn personal_best(&self, uname: &str) -> Result<Option<Duration>, SceneError> {
        let ms = db::metronome_personal_best(&self.db, self.session_id, uname)
            .await
            .scene_err("metronome::personal_best")?;

        Ok(ms.map(|v| Duration::from_millis(v as u64)))
    }
}
