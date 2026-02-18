//! Clockwork carnage with generic, reusable scene system
//! Scenes can be shared across different game server projects

use std::time::Duration;

use clap::Parser;
use insim::{WithRequestId, core::track::Track, insim::TinyType};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneExt, wait_for_players::WaitForPlayers},
};

mod chat;
mod cli;
mod components;
mod leaderboard;
mod scenes;
mod spawn_control;

// host + 1 player
const MIN_PLAYERS: usize = 2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = cli::Args::parse();

    let (insim, insim_handle) = insim::tcp(args.addr)
        .isi_admin_password(args.password.clone())
        .isi_iname("clockwork".to_owned())
        .isi_prefix('!')
        .isi_flag_mso_cols(true)
        .spawn(100)
        .await?;

    tracing::info!("Starting clockwork carnage");

    let (presence, presence_handle) = presence::spawn(insim.clone(), 32);
    let (game, game_handle) = game::spawn(insim.clone(), 32);
    let (chat, chat_handle) = chat::spawn(insim.clone());

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
    .then(scenes::WaitForAdminStart {
        insim: insim.clone(),
        presence: presence.clone(),
        chat: chat.clone(),
    })
    .then(
        scenes::SetupTrack {
            insim: insim.clone(),
            presence: presence.clone(),
            min_players: MIN_PLAYERS,
            game: game.clone(),
            track: args.track.unwrap_or(Track::Fe1x),
            layout: Some(args.layout.unwrap_or("CC".to_string())),
        }
        .with_timeout(Duration::from_secs(60)),
    )
    .then(scenes::Clockwork {
        game: game.clone(),
        presence: presence.clone(),
        chat: chat.clone(),
        rounds: args.rounds.unwrap_or(5),
        max_scorers: args.max_scorers.unwrap_or(10),
        target: Duration::from_secs(20),
        insim: insim.clone(),
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
        res = clockwork.run() => {
            tracing::info!("{res:?}");
        },
        _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, chat::ChatMsg::Quit)) => {}
    }

    Ok(())
}
