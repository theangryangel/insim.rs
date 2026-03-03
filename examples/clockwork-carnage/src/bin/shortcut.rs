//! Clockwork Carnage — Challenge mode
//! Always-on weekly challenge: players drop in, do timed runs, compete for fastest time.

use std::{net::SocketAddr, time::Duration};

use clap::{Parser, Subcommand};
use clockwork_carnage::{MIN_PLAYERS, shortcut, db, setup_track};
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
    /// Run the challenge (reads track/layout from active challenge in DB)
    Run {
        #[arg(short, long)]
        addr: SocketAddr,

        #[arg(short, long)]
        password: Option<String>,
    },

    /// Show the current active challenge configuration
    List,

    /// Set/replace the active challenge configuration
    Add {
        #[arg(short, long)]
        track: Track,

        #[arg(short, long)]
        layout: String,
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
        Command::Add { track, layout } => {
            // End any existing active challenge
            if let Some(existing) = db::any_active_challenge(&pool).await? {
                db::end_challenge(&pool, existing.id).await?;
                println!(
                    "Ended previous challenge #{} ({}/{})",
                    existing.id, existing.track, existing.layout
                );
            }

            let id = db::create_challenge(&pool, &track, &layout).await?;
            println!("Created challenge #{id} ({track}/{layout})");
        },

        Command::List => match db::any_active_challenge(&pool).await? {
            Some(c) => {
                println!("Active challenge #{}: track={}, layout={}", c.id, c.track, c.layout);
            },
            None => {
                println!("No active challenge configured");
            },
        },

        Command::Run { addr, password } => {
            let shortcut = match db::any_active_challenge(&pool).await? {
                Some(c) => c,
                None => {
                    eprintln!("No active challenge configured. Use 'add' first.");
                    std::process::exit(1);
                },
            };

            let track = shortcut.track;
            let layout = shortcut.layout.clone();
            let challenge_id = shortcut.id;

            tracing::info!(
                "Running challenge #{challenge_id} on {}/{}",
                shortcut.track,
                layout
            );

            let (insim, insim_handle) = insim::tcp(addr)
                .isi_admin_password(password)
                .isi_iname("challenge".to_owned())
                .isi_prefix('!')
                .isi_flag_mso_cols(true)
                .spawn(100)
                .await?;

            let (presence, presence_handle) = presence::spawn(insim.clone(), 32);
            let (game, game_handle) = game::spawn(insim.clone(), 32);
            let (chat, chat_handle) = shortcut::chat::spawn(insim.clone());
            let user_sync_handle = db::spawn_user_sync(&presence, pool.clone());

            insim.send(TinyType::Ncn.with_request_id(1)).await?;
            insim.send(TinyType::Npl.with_request_id(2)).await?;
            insim.send(TinyType::Sst.with_request_id(3)).await?;

            for &cmd in &["/select no", "/vote no", "/autokick no"] {
                insim.send_command(cmd).await?;
            }

            let challenge_scene = WaitForPlayers {
                insim: insim.clone(),
                presence: presence.clone(),
                min_players: MIN_PLAYERS,
            }
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
            .then(shortcut::ChallengeLoop {
                insim: insim.clone(),
                game: game.clone(),
                presence: presence.clone(),
                chat: chat.clone(),
                db: pool.clone(),
                challenge_id,
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
                res = challenge_scene.run() => {
                    tracing::info!("{res:?}");
                    if let Err(e) = db::end_challenge(&pool, challenge_id).await {
                        tracing::warn!("Failed to end challenge in DB: {e}");
                    }
                },
                _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, shortcut::chat::ChallengeChatMsg::Quit)) => {}
            }
        },
    }

    Ok(())
}
