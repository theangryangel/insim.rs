//! 20s league
mod chat;
mod combo;
mod components;
mod config;
mod db;
mod scenes;

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use insim::{WithRequestId, identifiers::ConnectionId, insim::TinyType};
use kitcar::{
    chat::Parse,
    combos::Combo,
    game::{GameHandle, GameInfo},
    leaderboard::{Leaderboard, LeaderboardHandle},
    presence::{Presence, PresenceHandle},
    ui,
};
use tokio::{task::JoinHandle, time::timeout};
use tokio_util::sync::CancellationToken;

use crate::{chat::MyChatCommands, components::RootProps, config::Config, db::Repo};

#[derive(Debug, Clone)]
enum GameState {
    Idle,
    TrackRotation {
        combo: Combo<combo::ComboExt>,
        game_id: i64,
    },
    Lobby {
        combo: Combo<combo::ComboExt>,
        game_id: i64,
    },
    Round {
        game_id: i64,
        round: u32,
        combo: Combo<combo::ComboExt>,
    },
    Victory {
        game_id: i64,
    },
}

#[derive(Debug, Clone)]
struct Context {
    insim: insim::builder::SpawnedHandle,
    ui: ui::ManagerHandle<components::Root>,
    presence: PresenceHandle,
    game: GameHandle,
    leaderboard: LeaderboardHandle<String>,
    config: Arc<Config>,
    shutdown: CancellationToken,
    database: Repo,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config = Arc::new(config::Config::from_file("config.yaml")?);

    tracing::info!("{:?}", config);

    let repo = db::Repo::new(&config.database);
    repo.migrate()?;

    let (insim, _join_handle) = insim::tcp(config.addr.as_str())
        .isi_admin_password(config.admin.clone())
        .isi_iname("cadence-cup".to_owned())
        .isi_prefix('!')
        .spawn(100)
        .await?;

    let (ui_handle, _ui_thread) = ui::Manager::spawn::<components::Root>(
        insim.clone(),
        RootProps {
            scene: components::RootScene::Idle,
        },
    );

    tracing::info!("20 Second League started!");

    let mut packets = insim.subscribe();
    let mut scene_handle: Option<JoinHandle<anyhow::Result<Option<GameState>>>> = None;
    let mut scene = GameState::Idle;

    let cx = Context {
        insim: insim.clone(),
        ui: ui_handle.clone(),
        presence: Presence::spawn(insim.clone(), 32),
        game: GameInfo::spawn(insim.clone(), 32),
        // FIXME: with database this is unrequired probably
        leaderboard: Leaderboard::<String>::spawn(32),
        config: config.clone(),
        shutdown: CancellationToken::new(),
        database: repo.clone(),
    };

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;
    let _ = insim.send(TinyType::Sst.with_request_id(3)).await;

    loop {
        // get a temporary handle for the select loop below
        let handle = scene_handle.get_or_insert_with(|| match scene {
            GameState::Idle => tokio::task::spawn(scenes::idle(cx.clone())),
            GameState::TrackRotation { ref combo, game_id } => {
                tokio::task::spawn(scenes::track_rotation(cx.clone(), combo.clone(), game_id))
            },
            GameState::Lobby { ref combo, game_id } => {
                tokio::task::spawn(scenes::lobby(cx.clone(), combo.clone(), game_id))
            },
            GameState::Round {
                round,
                ref combo,
                game_id,
            } => tokio::task::spawn(scenes::round(cx.clone(), game_id, round, combo.clone())),
            GameState::Victory { game_id } => {
                tokio::task::spawn(scenes::victory(cx.clone(), game_id))
            },
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                cx.shutdown.cancel();
                // we can take this because we're shutting down
                if let Some(scene_handle) = scene_handle.take() {
                    tracing::info!("Waiting 5 seconds for graceful shutdown...");
                    let _ = timeout(Duration::from_secs(5), scene_handle).await;
                }
                break;

            },
            result = handle => {
                scene_handle = None;
                match result?? {
                    Some(next) => { scene = next; },
                    None => { break; },
                }
            },
            packet = packets.recv() => {
                match packet? {
                    insim::Packet::Ncn(ncn) if ncn.ucid != ConnectionId::LOCAL => {
                        repo.upsert_player(&ncn.uname, &ncn.pname)?;
                    },

                    insim::Packet::Mso(mso) => {
                        match MyChatCommands::parse(mso.msg_from_textstart()) {
                            Ok(MyChatCommands::Quit) => {
                                if_chain::if_chain! {
                                    if let Some(conn_info) = cx.presence.connection(&mso.ucid).await;
                                    if conn_info.admin;
                                    then {
                                        insim.send_message("Quitting.. bye!", ConnectionId::ALL).await?;
                                        break;
                                    }
                                }
                            },
                            Ok(MyChatCommands::Echo { message }) => {
                                insim.send_message(&format!("Echo: {}", message), mso.ucid).await?;
                            }
                            Ok(MyChatCommands::Help) => {
                                insim.send_message("Available commands:", mso.ucid).await?;
                                for cmd in MyChatCommands::help() {
                                    insim.send_message(cmd, mso.ucid).await?;
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
