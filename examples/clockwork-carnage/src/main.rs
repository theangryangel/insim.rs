//! Clockwork Carnage — unified binary (InSim runner + web dashboard).

#![allow(missing_docs, missing_debug_implementations)]

mod db;

use tracing_subscriber::fmt::format::FmtSpan;
mod games;
mod hud;
mod web;

type ChatError = kitcar::chat::RuntimeError;
const MIN_PLAYERS: usize = 2;

use std::net::SocketAddr;

use anyhow::Context as _;
use clap::Parser;
use db::EventMode;
use games::{GameCtx, execute};
use insim::{WithRequestId, insim::TinyType};
use kitcar::{game, presence};
use sqlx::types::Json;

// -- Config -------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(default = "default_db")]
    db: String,
    insim: Option<InsimSection>,
    web: Option<WebSection>,
}

#[derive(Debug, serde::Deserialize)]
struct InsimSection {
    addr: SocketAddr,
    password: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct WebSection {
    #[serde(default = "default_listen")]
    listen: SocketAddr,
    oauth_client_id: String,
    oauth_client_secret: String,
    oauth_redirect_uri: String,
    session_key: Option<String>,
}

fn default_db() -> String {
    "clockwork-carnage.db".to_owned()
}

fn default_listen() -> SocketAddr {
    "127.0.0.1:3000".parse().unwrap()
}

fn load_config(path: &std::path::Path) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read config file {path:?}"))?;
    let config: Config =
        toml::from_str(&text).with_context(|| format!("cannot parse config file {path:?}"))?;
    if config.insim.is_none() && config.web.is_none() {
        anyhow::bail!("at least one of [insim] or [web] must be present in config");
    }
    if let Some(web) = &config.web {
        let key_len = web.session_key.as_deref().unwrap_or("").len();
        if key_len > 0 && key_len < 64 {
            anyhow::bail!("`session_key` must be at least 64 bytes (got {key_len})");
        }
    }
    Ok(config)
}

// -- CLI ----------------------------------------------------------------------

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.toml")]
    config: std::path::PathBuf,
}

// -- Announcements ------------------------------------------------------------

fn next_announce_interval(secs: i64) -> std::time::Duration {
    std::time::Duration::from_secs(if secs > 3600 {
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

async fn announce_loop(pool: db::Pool, insim: insim::builder::InsimTask) {
    loop {
        let sleep = match db::next_scheduled_event(&pool).await {
            Ok(Some((event, secs))) => {
                let mode = match &*event.mode {
                    EventMode::Metronome { .. } => "Metronome",
                    EventMode::Shortcut => "Shortcut",
                    EventMode::Bomb { .. } => "Bomb",
                };
                let name = event
                    .name
                    .as_deref()
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("{} / {}", event.track, event.layout));
                let remaining = std::time::Duration::from_secs(secs as u64);
                let msg = format!(
                    "Upcoming: {} — {} on {} in {remaining:.0?}",
                    name, mode, event.track,
                );
                if let Err(e) = insim.send_message(msg, None).await {
                    tracing::warn!("Failed to send event announcement: {e}");
                }
                next_announce_interval(secs)
            },
            Ok(None) => std::time::Duration::from_secs(60),
            Err(e) => {
                tracing::warn!("Failed to fetch next scheduled event: {e}");
                std::time::Duration::from_secs(60)
            },
        };
        tokio::time::sleep(sleep).await;
    }
}

// -- Scheduler ----------------------------------------------------------------

async fn scheduler_loop(pool: db::Pool) {
    loop {
        let now = db::Timestamp::now();

        let sleep_secs: u64 = async {
            // Auto-stop: if the active event's scheduled_end_at has passed, complete it.
            if let Some(event) = db::active_event(&pool).await? {
                if let Some(end) = event.scheduled_end_at {
                    if end <= now {
                        tracing::info!(
                            "Auto-completing event #{} (scheduled_end_at reached)",
                            event.id
                        );
                        db::complete_event(&pool, event.id).await?;
                        return Ok(5);
                    }
                    let secs_to_end = (end.as_second() - now.as_second()).max(0) as u64;
                    return Ok(secs_to_end.min(30));
                }
                return Ok(30);
            }

            // Auto-start: if no event is active, check for a due PENDING event.
            if let Some(event) = db::next_due_event(&pool).await? {
                tracing::info!("Auto-starting event #{} (scheduled_at reached)", event.id);
                db::switch_event(&pool, event.id).await?;
                return Ok(5);
            }

            // No active, no due event. Check how soon the next one is.
            match db::next_scheduled_event(&pool).await? {
                Some((_, secs)) => Ok((secs as u64).min(30).max(5)),
                None => Ok(30),
            }
        }
        .await
        .unwrap_or_else(|e: sqlx::Error| {
            tracing::warn!("Scheduler error: {e}");
            30
        });

        tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
    }
}

// -- Runner -------------------------------------------------------------------

async fn run_loop(pool: db::Pool, config: Config) -> anyhow::Result<()> {
    // InSim setup — only if [insim] present
    let (insim_handle, presence_handle, game_handle, user_sync_handle, ctx) =
        if let Some(insim_cfg) = config.insim {
            let (insim, insim_handle) = insim::tcp(insim_cfg.addr)
                .isi_admin_password(insim_cfg.password)
                .isi_iname("carnage".to_owned())
                .isi_prefix('!')
                .isi_flag_mso_cols(true)
                .isi_flag_mci(true)
                .isi_interval(std::time::Duration::from_millis(250))
                .spawn(100)
                .await?;

            tracing::info!("Connected to InSim");

            let (presence, presence_handle) = presence::spawn(insim.clone(), 32);
            let (game, game_handle) = game::spawn(insim.clone(), 32);
            let user_sync_handle = db::spawn_user_sync(&presence, pool.clone());

            insim.send(TinyType::Ncn.with_request_id(1)).await?;
            insim.send(TinyType::Npl.with_request_id(2)).await?;
            insim.send(TinyType::Sst.with_request_id(3)).await?;

            for &cmd in &["/select no", "/vote no", "/autokick no"] {
                insim.send_command(cmd).await?;
            }

            let ctx = GameCtx {
                pool: pool.clone(),
                insim: insim.clone(),
                presence,
                game,
            };

            (
                Some(insim_handle),
                Some(presence_handle),
                Some(game_handle),
                Some(user_sync_handle),
                Some(ctx),
            )
        } else {
            (None, None, None, None, None)
        };

    // Web — only if [web] present
    let web_listen = config.web.as_ref().map(|w| w.listen);
    let web_cfg = config.web.map(|w| web::WebConfig {
        oauth_client_id: w.oauth_client_id,
        oauth_client_secret: w.oauth_client_secret,
        oauth_redirect_uri: w.oauth_redirect_uri,
        session_key: w.session_key.unwrap_or_else(|| "a".repeat(64)),
    });

    let announce_data = ctx.as_ref().map(|c| (c.pool.clone(), c.insim.clone()));
    let scheduler_pool = pool.clone();

    let reconcile = async move {
        let Some(ctx) = ctx else {
            std::future::pending::<()>().await;
            unreachable!()
        };

        let mut current_event_id: Option<i64> = None;
        let mut current_task: Option<
            tokio::task::JoinHandle<Result<(), kitcar::scenes::SceneError>>,
        > = None;
        let mut current_cancel: Option<tokio_util::sync::CancellationToken> = None;

        loop {
            let desired = db::active_event(&ctx.pool).await;

            match (&current_task, desired) {
                (_, Err(e)) => {
                    tracing::warn!("Failed to poll active event: {e}");
                },

                (None, Ok(None)) => {},

                (None, Ok(Some(event))) => {
                    tracing::info!(
                        "Starting event #{} ({:?} on {}/{})",
                        event.id,
                        event.mode,
                        event.track,
                        event.layout
                    );
                    current_event_id = Some(event.id);
                    let ctx_ref = &ctx;
                    let cancel = tokio_util::sync::CancellationToken::new();
                    current_cancel = Some(cancel.clone());
                    current_task = Some(tokio::spawn({
                        let event = event.clone();
                        let pool = ctx_ref.pool.clone();
                        let insim = ctx_ref.insim.clone();
                        let presence = ctx_ref.presence.clone();
                        let game = ctx_ref.game.clone();
                        async move {
                            let ctx = GameCtx {
                                pool,
                                insim,
                                presence,
                                game,
                            };
                            match event.mode {
                                Json(EventMode::Metronome { .. }) => {
                                    execute::<games::metronome::MetronomeGame>(&event, &ctx, cancel)
                                        .await
                                },
                                Json(EventMode::Shortcut) => {
                                    execute::<games::shortcut::ShortcutGame>(&event, &ctx, cancel)
                                        .await
                                },
                                Json(EventMode::Bomb { .. }) => {
                                    execute::<games::bomb::BombGame>(&event, &ctx, cancel).await
                                },
                            }
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
                        let _ =
                            tokio::time::timeout(std::time::Duration::from_secs(10), task).await;
                    }
                    current_event_id = None;
                },
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    };

    // Wrap each optional future so disabled branches park forever instead of
    // panicking — tokio::select! evaluates future expressions before checking
    // guards, so .unwrap() on None would panic even with an `if false` guard.
    let web_fut = async move {
        match (web_listen, web_cfg) {
            (Some(listen), Some(cfg)) => web::serve(listen, pool.clone(), cfg).await,
            _ => std::future::pending().await,
        }
    };
    let insim_fut = async move {
        match insim_handle {
            Some(h) => h.await,
            None => std::future::pending().await,
        }
    };
    let presence_fut = async move {
        match presence_handle {
            Some(h) => h.await,
            None => std::future::pending().await,
        }
    };
    let game_fut = async move {
        match game_handle {
            Some(h) => h.await,
            None => std::future::pending().await,
        }
    };
    let user_sync_fut = async move {
        match user_sync_handle {
            Some(h) => h.await,
            None => std::future::pending().await,
        }
    };
    let announce_fut = async move {
        match announce_data {
            Some((pool, insim)) => announce_loop(pool, insim).await,
            None => std::future::pending().await,
        }
    };

    tokio::select! {
        _ = reconcile => {},
        _ = scheduler_loop(scheduler_pool) => {},
        result = web_fut => {
            if let Err(e) = result {
                tracing::error!("Web server error: {e}");
            }
        },
        res = insim_fut => {
            match res {
                Ok(Ok(())) => tracing::info!("InSim background task exited"),
                Ok(Err(e)) => tracing::error!("InSim background task failed: {e:?}"),
                Err(e) => tracing::error!("InSim background task join failed: {e}"),
            }
        },
        res = presence_fut => {
            match res {
                Ok(Ok(())) => tracing::info!("Presence background task exited"),
                Ok(Err(e)) => tracing::error!("Presence background task failed: {e}"),
                Err(e) => tracing::error!("Presence background task join failed: {e}"),
            }
        },
        res = game_fut => {
            match res {
                Ok(Ok(())) => tracing::info!("Game background task exited"),
                Ok(Err(e)) => tracing::error!("Game background task failed: {e}"),
                Err(e) => tracing::error!("Game background task join failed: {e}"),
            }
        },
        res = user_sync_fut => {
            match res {
                Ok(Ok(())) => tracing::info!("User sync background task exited"),
                Ok(Err(e)) => tracing::error!("User sync background task failed: {e}"),
                Err(e) => tracing::error!("User sync background task join failed: {e}"),
            }
        },
        _ = announce_fut => {},
    }

    Ok(())
}

// -- Entry point --------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();
    let config = load_config(&args.config)?;
    let pool = db::connect(&config.db).await?;

    run_loop(pool, config).await?;

    Ok(())
}
