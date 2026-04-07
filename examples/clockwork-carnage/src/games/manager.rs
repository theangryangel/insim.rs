use std::time::{Duration, Instant};

use insim_extras::scenes::SceneError;
use sqlx::types::Json;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::vehicle_restrictions;
use crate::{
    db,
    games::{MiniGame, MiniGameCtx},
};

/// Manages the lifecycle of mini-games by:
/// 1. Scheduling: Auto-starting/stopping events in the DB based on time.
/// 2. Reconciling: Ensuring the currently running InSim task matches the ACTIVE event.
/// 3. Announcing: Sending periodic chat messages about upcoming events.
pub struct MiniGameManager {
    pool: db::Pool,
    ctx: Option<MiniGameCtx>,
}

impl MiniGameManager {
    pub fn new(pool: db::Pool, ctx: Option<MiniGameCtx>) -> Self {
        Self { pool, ctx }
    }

    /// Generic executor: setup, run until quit or cancellation, teardown.
    async fn execute<G: MiniGame>(
        event: &db::Event,
        ctx: &MiniGameCtx,
        cancel: CancellationToken,
    ) -> Result<(), SceneError> {
        vehicle_restrictions::apply(&ctx.insim, &event.allowed_vehicles.0).await?;
        let (game, _guard) = G::setup(event, ctx).await?;
        game.run(ctx, cancel).await?;
        game.teardown(event, ctx).await?;
        vehicle_restrictions::apply(&ctx.insim, &[]).await?;
        ctx.insim.send_command("/axclear").await?;
        if let Some(ref url) = ctx.base_url {
            let _ = ctx
                .insim
                .send_message(format!("Results: {url}/events/{}", event.id), None)
                .await;
        }
        Ok(())
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let mut current_event_id: Option<i64> = None;
        let mut current_task: Option<JoinHandle<Result<(), SceneError>>> = None;
        let mut current_cancel: Option<CancellationToken> = None;

        let mut idle_task: Option<JoinHandle<Result<(), SceneError>>> = None;
        let mut idle_cancel: Option<CancellationToken> = None;

        let mut last_announced_at: Option<Instant> = None;
        let mut last_announced_event_id: Option<i64> = None;

        loop {
            // 1. Run scheduler logic
            let sleep_secs = match tick_scheduler(&self.pool).await {
                Ok(secs) => secs,
                Err(e) => {
                    tracing::warn!("Scheduler tick failed: {e}");
                    5
                },
            };

            // 2. Run reconciliation + announcement (only if InSim context is available)
            if let Some(ref ctx) = self.ctx {
                // --- Reconciliation ---
                let desired = db::active_event(&ctx.pool).await;

                match (&current_task, desired) {
                    (_, Err(e)) => {
                        tracing::warn!("Failed to poll active event: {e}");
                    },

                    (None, Ok(None)) => {
                        // No event running - start idle if not already running.
                        if idle_task.as_ref().map_or(true, |t| t.is_finished()) {
                            tracing::info!("No active event - starting idle mode");
                            let cancel = CancellationToken::new();
                            idle_cancel = Some(cancel.clone());
                            let ctx_clone = MiniGameCtx {
                                pool: ctx.pool.clone(),
                                insim: ctx.insim.clone(),
                                presence: ctx.presence.clone(),
                                game: ctx.game.clone(),
                                base_url: ctx.base_url.clone(),
                            };
                            idle_task = Some(tokio::spawn(async move {
                                super::idle::run(&ctx_clone, cancel).await
                            }));
                        }
                    },

                    (None, Ok(Some(event))) => {
                        // Cancel idle if it's running before starting the real event.
                        if let Some(token) = idle_cancel.take() {
                            token.cancel();
                        }
                        if let Some(task) = idle_task.take() {
                            let _ = tokio::time::timeout(Duration::from_secs(5), task).await;
                        }

                        tracing::info!(
                            "Starting event #{} ({:?} on {}/{})",
                            event.id,
                            event.mode,
                            event.track,
                            event.layout
                        );
                        current_event_id = Some(event.id);
                        let cancel = CancellationToken::new();
                        current_cancel = Some(cancel.clone());

                        let event_clone = event.clone();
                        let ctx_clone = MiniGameCtx {
                            pool: ctx.pool.clone(),
                            insim: ctx.insim.clone(),
                            presence: ctx.presence.clone(),
                            game: ctx.game.clone(),
                            base_url: ctx.base_url.clone(),
                        };

                        current_task = Some(tokio::spawn(async move {
                            match event_clone.mode {
                                Json(db::EventMode::Metronome { .. }) => {
                                    Self::execute::<crate::games::metronome::MetronomeGame>(
                                        &event_clone,
                                        &ctx_clone,
                                        cancel,
                                    )
                                    .await
                                },
                                Json(db::EventMode::Shortcut) => {
                                    Self::execute::<crate::games::shortcut::ShortcutGame>(
                                        &event_clone,
                                        &ctx_clone,
                                        cancel,
                                    )
                                    .await
                                },
                                Json(db::EventMode::Bomb { .. }) => {
                                    Self::execute::<crate::games::bomb::BombGame>(
                                        &event_clone,
                                        &ctx_clone,
                                        cancel,
                                    )
                                    .await
                                },
                            }
                        }));
                    },

                    (Some(task), Ok(Some(event)))
                        if current_event_id == Some(event.id) && !task.is_finished() => {},

                    (Some(_), Ok(Some(event))) if current_event_id == Some(event.id) => {
                        current_cancel = None;
                        let task = current_task.take().unwrap();
                        match task.await {
                            Ok(Ok(())) => {
                                tracing::info!("Event #{} completed", event.id);
                            },
                            Ok(Err(e)) => {
                                tracing::error!(
                                    "Event #{} failed: {e:?} (leaving ACTIVE for crash recovery)",
                                    event.id
                                );
                            },
                            Err(e) => {
                                tracing::error!(
                                    "Event #{} join error: {e} (leaving ACTIVE for crash recovery)",
                                    event.id
                                );
                            },
                        }
                        current_event_id = None;
                    },

                    (Some(_), Ok(_)) => {
                        tracing::info!("Desired event changed, cancelling current task");
                        if let Some(token) = current_cancel.take() {
                            token.cancel();
                        }
                        if let Some(task) = current_task.take() {
                            let _ = tokio::time::timeout(std::time::Duration::from_secs(10), task)
                                .await;
                        }
                        current_event_id = None;
                    },
                }

                // --- Announcement ---
                if let Ok(Some((event, secs))) = db::next_scheduled_event(&self.pool).await {
                    let now_inst = Instant::now();
                    let should_announce = match (last_announced_at, last_announced_event_id) {
                        (Some(last), Some(id)) if id == event.id => {
                            now_inst.duration_since(last) >= next_announce_interval(secs)
                        },
                        _ => true,
                    };

                    if should_announce {
                        let mode = match &*event.mode {
                            db::EventMode::Metronome { .. } => "Metronome",
                            db::EventMode::Shortcut => "Shortcut",
                            db::EventMode::Bomb { .. } => "Bomb",
                        };
                        let name = event
                            .name
                            .as_deref()
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("{} / {}", event.track, event.layout));
                        let remaining = Duration::from_secs(secs as u64);
                        let msg = format!(
                            "Upcoming: {} - {} on {} in {remaining:.0?}",
                            name, mode, event.track,
                        );
                        if let Err(e) = ctx.insim.send_message(msg, None).await {
                            tracing::warn!("Failed to send event announcement: {e}");
                        }
                        if let Some(ref url) = ctx.base_url {
                            let url_msg = format!("{url}/event/{}", event.id);
                            if let Err(e) = ctx.insim.send_message(url_msg, None).await {
                                tracing::warn!("Failed to send event URL announcement: {e}");
                            }
                        }
                        last_announced_at = Some(now_inst);
                        last_announced_event_id = Some(event.id);
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(sleep_secs.min(3))).await;
        }
    }
}

async fn tick_scheduler(pool: &db::Pool) -> Result<u64, sqlx::Error> {
    let now = db::Timestamp::now();

    // Auto-stop: if the active event's scheduled_end_at has passed, complete it.
    if let Some(event) = db::active_event(pool).await? {
        if let Some(end) = event.scheduled_end_at {
            if end <= now {
                tracing::info!(
                    "Auto-completing event #{} (scheduled_end_at reached)",
                    event.id
                );
                db::complete_event(pool, event.id).await?;
                return Ok(1);
            }
            let secs_to_end = (end.as_second() - now.as_second()).max(0) as u64;
            return Ok(secs_to_end.min(5));
        }
        return Ok(30);
    }

    // Auto-start: if no event is active, check for a due PENDING event.
    if let Some(event) = db::next_due_event(pool).await? {
        tracing::info!("Auto-starting event #{} (scheduled_at reached)", event.id);
        db::switch_event(pool, event.id).await?;
        return Ok(1);
    }

    // No active, no due event. Check how soon the next one is.
    match db::next_scheduled_event(pool).await? {
        Some((_, secs)) => Ok((secs as u64).clamp(1, 30)),
        None => Ok(30),
    }
}

fn next_announce_interval(secs: i64) -> Duration {
    Duration::from_secs(if secs > 3600 {
        1800
    } else if secs > 900 {
        600
    } else if secs > 300 {
        300
    } else if secs > 60 {
        60
    } else {
        15
    })
}
