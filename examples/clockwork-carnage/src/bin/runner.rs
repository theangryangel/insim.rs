//! Clockwork Carnage — Runner
//! Unified binary that drives both metronome and shortcut modes via a reconciliation loop
//! polling a `sessions` table in SQLite.

use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use clockwork_carnage::{
    db::{self, SessionMode},
    runner::{self, GameCtx, execute},
};
use insim::{WithRequestId, core::track::Track, insim::TinyType};
use kitcar::{game, presence};
use sqlx::types::Json;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.db")]
    db: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the runner (connects to InSim, polls for active sessions)
    Run {
        #[arg(short, long)]
        addr: SocketAddr,

        #[arg(short, long)]
        password: Option<String>,
    },

    /// Queue a new session
    Add {
        #[command(subcommand)]
        mode: AddMode,
    },

    /// List all sessions
    List,

    /// Activate a pending session (sets status to ACTIVE)
    Activate {
        /// Session ID to activate
        id: i64,
    },

    /// Set the post-event write-up for a session
    Writeup {
        /// Session ID
        id: i64,

        /// Write-up text
        text: String,
    },
}

#[derive(Debug, Subcommand)]
enum AddMode {
    /// Create a metronome (event) session
    Metronome {
        #[arg(short, long)]
        track: Track,

        #[arg(short, long, default_value = "")]
        layout: String,

        #[arg(short, long, default_value_t = 5)]
        rounds: i64,

        #[arg(long, default_value_t = 20)]
        target: u64,

        #[arg(short, long, default_value_t = 10)]
        max_scorers: i64,

        #[arg(long, default_value_t = 300)]
        lobby_duration_secs: u64,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        description: Option<String>,

        #[arg(long)]
        scheduled_at: Option<String>,
    },

    /// Create a shortcut (challenge) session
    Shortcut {
        #[arg(short, long)]
        track: Track,

        #[arg(short, long, default_value = "")]
        layout: String,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        description: Option<String>,

        #[arg(long)]
        scheduled_at: Option<String>,
    },
}

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
    let pool = db::connect(&args.db).await?;

    match args.command {
        Command::Add { mode } => match mode {
            AddMode::Metronome {
                track,
                layout,
                rounds,
                target,
                max_scorers,
                lobby_duration_secs,
                name,
                description,
                scheduled_at,
            } => {
                let target_ms = (target * 1000) as i64;
                let id = db::create_metronome_session(
                    &pool,
                    &db::CreateMetronomeParams {
                        track,
                        layout,
                        rounds,
                        target_ms,
                        max_scorers,
                        lobby_duration_secs: lobby_duration_secs as i64,
                        name,
                        description,
                        scheduled_at,
                    },
                )
                .await?;
                println!("Created metronome session #{id}");
            },
            AddMode::Shortcut { track, layout, name, description, scheduled_at } => {
                let id = db::create_shortcut_session(
                    &pool,
                    &db::CreateShortcutParams {
                        track,
                        layout,
                        name,
                        description,
                        scheduled_at,
                    },
                ).await?;
                println!("Created shortcut session #{id}");
            },
        },

        Command::List => {
            let sessions = db::all_sessions(&pool).await?;
            if sessions.is_empty() {
                println!("No sessions.");
            } else {
                for s in sessions {
                    let label = s.name.as_deref().unwrap_or("");
                    println!(
                        "#{} {:?} {:?} {}/{} {} ({})",
                        s.id, s.mode, s.status, s.track, s.layout, label, s.created_at
                    );
                }
            }
        },

        Command::Activate { id } => {
            match db::pending_session(&pool, id).await? {
                Some(_) => {
                    db::activate_session(&pool, id).await?;
                    println!("Activated session #{id}");
                },
                None => {
                    eprintln!("Session #{id} not found or not in PENDING status.");
                    std::process::exit(1);
                },
            }
        },

        Command::Writeup { id, text } => {
            db::update_session_writeup(&pool, id, &text).await?;
            println!("Updated write-up for session #{id}");
        },

        Command::Run { addr, password } => {
            run_loop(pool, addr, password).await?;
        },
    }

    Ok(())
}

async fn run_loop(pool: db::Pool, addr: SocketAddr, password: Option<String>) -> anyhow::Result<()> {
    let (insim, insim_handle) = insim::tcp(addr)
        .isi_admin_password(password)
        .isi_iname("carnage".to_owned())
        .isi_prefix('!')
        .isi_flag_mso_cols(true)
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

    let reconcile = async {
        let mut current_session_id: Option<i64> = None;
        let mut current_task: Option<tokio::task::JoinHandle<Result<(), kitcar::scenes::SceneError>>> = None;

        loop {
            // Auto-activate any due scheduled session, interrupting the current one if needed
            if let Ok(Some(session)) = db::next_scheduled_session(&pool).await {
                tracing::info!("Auto-activating scheduled session #{}", session.id);
                let _ = db::switch_session(&pool, session.id).await;
            }

            let desired = db::active_session(&pool).await;

            match (&current_task, desired) {
                // Error polling DB — log and retry next tick
                (_, Err(e)) => {
                    tracing::warn!("Failed to poll active session: {e}");
                },

                // Nothing running, nothing desired — idle
                (None, Ok(None)) => {},

                // Nothing running, new session desired — start it
                (None, Ok(Some(session))) => {
                    tracing::info!(
                        "Starting session #{} ({:?} on {}/{})",
                        session.id, session.mode, session.track, session.layout
                    );
                    current_session_id = Some(session.id);
                    let ctx_ref = &ctx;
                    current_task = Some(tokio::spawn({
                        let session = session.clone();
                        let pool = ctx_ref.pool.clone();
                        let insim = ctx_ref.insim.clone();
                        let presence = ctx_ref.presence.clone();
                        let game = ctx_ref.game.clone();
                        async move {
                            let ctx = GameCtx { pool, insim, presence, game };
                            match session.mode {
                                Json(SessionMode::Metronome { .. }) => {
                                    execute::<runner::metronome::MetronomeGame>(&session, &ctx).await
                                },
                                Json(SessionMode::Shortcut) => {
                                    execute::<runner::shortcut::ShortcutGame>(&session, &ctx).await
                                },
                            }
                        }
                    }));
                },

                // Running, same session still desired, task not finished — noop
                (Some(task), Ok(Some(session)))
                    if current_session_id == Some(session.id) && !task.is_finished() => {},

                // Running, same session, task finished — check result and clear
                (Some(_), Ok(Some(session)))
                    if current_session_id == Some(session.id) =>
                {
                    let task = current_task.take().unwrap();
                    match task.await {
                        Ok(Ok(())) => {
                            tracing::info!("Session #{} completed", session.id);
                        },
                        Ok(Err(e)) => {
                            tracing::error!(
                                "Session #{} failed: {e:?} (leaving ACTIVE for crash recovery)",
                                session.id
                            );
                        },
                        Err(e) => {
                            tracing::error!(
                                "Session #{} join error: {e} (leaving ACTIVE for crash recovery)",
                                session.id
                            );
                        },
                    }
                    current_session_id = None;
                },

                // Running something, but desired changed (different session or none) — abort
                (Some(_), Ok(_)) => {
                    tracing::info!("Desired session changed, aborting current task");
                    if let Some(task) = current_task.take() {
                        task.abort();
                    }
                    current_session_id = None;
                    // Next tick will pick up the new session
                },
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    };

    tokio::select! {
        res = insim_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("InSim background task exited"),
                Ok(Err(e)) => tracing::error!("InSim background task failed: {e:?}"),
                Err(e) => tracing::error!("InSim background task join failed: {e}"),
            }
        },
        res = presence_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("Presence background task exited"),
                Ok(Err(e)) => tracing::error!("Presence background task failed: {e}"),
                Err(e) => tracing::error!("Presence background task join failed: {e}"),
            }
        },
        res = game_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("Game background task exited"),
                Ok(Err(e)) => tracing::error!("Game background task failed: {e}"),
                Err(e) => tracing::error!("Game background task join failed: {e}"),
            }
        },
        res = user_sync_handle => {
            match res {
                Ok(Ok(())) => tracing::info!("User sync background task exited"),
                Ok(Err(e)) => tracing::error!("User sync background task failed: {e}"),
                Err(e) => tracing::error!("User sync background task join failed: {e}"),
            }
        },
        _ = reconcile => {},
    }

    Ok(())
}
