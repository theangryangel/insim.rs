//! Clockwork Carnage — Event mode
//! Manually-started event: admin types `!start`, N rounds play out, victory screen, loop.

use std::{net::SocketAddr, time::Duration};

use clap::{Parser, Subcommand};
use clockwork_carnage::{MIN_PLAYERS, db, event, setup_track};
use insim::{WithRequestId, core::track::Track, insim::TinyType};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneExt, wait_for_players::WaitForPlayers},
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "clockwork-carnage.db")]
    db: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Set/replace the active event configuration
    Add {
        #[arg(short, long)]
        track: Track,

        #[arg(short, long)]
        layout: String,

        #[arg(short, long, default_value_t = 5)]
        rounds: usize,

        #[arg(long, default_value_t = 20)]
        target: u64,
    },

    /// Show the active event configuration
    List,

    /// Run the event (reads config from active event in DB)
    Run {
        #[arg(short, long)]
        addr: SocketAddr,

        #[arg(short, long)]
        password: Option<String>,

        #[arg(short, long, default_value_t = 10)]
        max_scorers: usize,
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
        Command::Add {
            track,
            layout,
            rounds,
            target,
        } => {
            if let Some(existing) = db::any_active_event(&pool).await? {
                db::end_event(&pool, existing.id).await?;
                println!(
                    "Ended previous event #{} ({}/{})",
                    existing.id, existing.track, existing.layout
                );
            }

            let target_ms = Duration::from_secs(target).as_millis() as i64;
            let id =
                db::create_event(&pool, &track, &layout, rounds as i64, target_ms).await?;
            println!("Created event #{id} ({track}/{layout}, {rounds} rounds, target {target}s)");
        },

        Command::List => match db::any_active_event(&pool).await? {
            Some(ev) => {
                let target_secs = ev.target_ms as f64 / 1000.0;
                println!(
                    "Active event #{}: track={}, layout={}, rounds={}, target={target_secs}s",
                    ev.id, ev.track, ev.layout, ev.rounds
                );
            },
            None => {
                println!("No active event configured");
            },
        },

        Command::Run {
            addr,
            password,
            max_scorers,
        } => {
            let ev = match db::any_active_event(&pool).await? {
                Some(ev) => ev,
                None => {
                    eprintln!("No active event configured. Use 'add' first.");
                    std::process::exit(1);
                },
            };

            let track = ev.track;
            let layout = ev.layout.clone();
            let event_id = ev.id;
            let rounds = ev.rounds as usize;
            let target = Duration::from_millis(ev.target_ms as u64);
            let start_round = ev.current_round as usize + 1;

            tracing::info!(
                "Running event #{event_id} on {}/{} (round {start_round}/{rounds})",
                ev.track,
                layout
            );

            let (insim, insim_handle) = insim::tcp(addr)
                .isi_admin_password(password)
                .isi_iname("clockwork".to_owned())
                .isi_prefix('!')
                .isi_flag_mso_cols(true)
                .spawn(100)
                .await?;

            tracing::info!("Starting clockwork carnage");

            let (presence, presence_handle) = presence::spawn(insim.clone(), 32);
            let (game, game_handle) = game::spawn(insim.clone(), 32);
            let (chat, chat_handle) = event::chat::spawn(insim.clone());
            let user_sync_handle = db::spawn_user_sync(&presence, pool.clone());

            insim.send(TinyType::Ncn.with_request_id(1)).await?;
            insim.send(TinyType::Npl.with_request_id(2)).await?;
            insim.send(TinyType::Sst.with_request_id(3)).await?;

            // Take over.
            // TODO: Probably want to consider if this is right.
            for &cmd in &["/select no", "/vote no", "/autokick no"] {
                insim.send_command(cmd).await?;
            }

            // Composible/reusable scenes snap together, "just like little lego"!
            let clockwork = WaitForPlayers {
                insim: insim.clone(),
                presence: presence.clone(),
                min_players: MIN_PLAYERS,
            }
            .then(event::WaitForAdminStart {
                insim: insim.clone(),
                presence: presence.clone(),
                chat: chat.clone(),
            })
            .then(
                setup_track::SetupTrack {
                    insim: insim.clone(),
                    presence: presence.clone(),
                    min_players: MIN_PLAYERS,
                    game: game.clone(),
                    track,
                    layout: Some(layout),
                }
                .with_timeout(Duration::from_secs(60)),
            )
            .then(event::Clockwork {
                game: game.clone(),
                presence: presence.clone(),
                chat: chat.clone(),
                start_round,
                rounds,
                max_scorers,
                target,
                insim: insim.clone(),
                db: pool.clone(),
                event_id,
            })
            .loop_until_quit();

            tokio::select! {
                res = insim_handle => {
                    match res {
                        Ok(Ok(())) => tracing::info!("Insim background task exited"),
                        Ok(Err(e)) => tracing::error!("Insim background task failed: {e:?}"),
                        Err(e) => tracing::error!("Insim background task join failed: {e}"),
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
                res = chat_handle => {
                    match res {
                        Ok(Ok(())) => tracing::info!("Chat background task exited"),
                        Ok(Err(e)) => tracing::error!("Chat background task failed: {e}"),
                        Err(e) => tracing::error!("Chat background task join failed: {e}"),
                    }
                },
                res = user_sync_handle => {
                    match res {
                        Ok(Ok(())) => tracing::info!("User sync background task exited"),
                        Ok(Err(e)) => tracing::error!("User sync background task failed: {e}"),
                        Err(e) => tracing::error!("User sync background task join failed: {e}"),
                    }
                },
                res = clockwork.run() => {
                    tracing::info!("{res:?}");
                    if let Err(e) = db::end_event(&pool, event_id).await {
                        tracing::warn!("Failed to end event in DB: {e}");
                    }
                },
                _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, event::chat::EventChatMsg::Quit)) => {}
            }
        },
    }

    Ok(())
}
