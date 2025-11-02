//! 20s league
mod chat;
mod combo;
mod components;
mod config;
mod scenes;

use std::sync::Arc;

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
use tokio::task::JoinHandle;

use crate::{chat::MyChatCommands, components::RootProps, config::Config};

#[derive(Debug, Clone)]
enum GameState {
    Idle,
    TrackRotation {
        combo: Combo<combo::ComboExt>,
    },
    Lobby {
        combo: Combo<combo::ComboExt>,
    },
    Round {
        round: u32,
        combo: Combo<combo::ComboExt>,
    },
    Victory,
}

#[derive(Debug, Clone)]
struct Context {
    insim: insim::builder::SpawnedHandle,
    ui: ui::ManagerHandle<components::Root>,
    presence: PresenceHandle,
    game: GameHandle,
    leaderboard: LeaderboardHandle<String>,
    config: Arc<Config>,
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

    let config = config::Config::from_file("config.yaml")?;

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
    let mut scene_handle: Option<JoinHandle<anyhow::Result<GameState>>> = None;
    let mut scene = GameState::Idle;

    let cx = Context {
        insim: insim.clone(),
        ui: ui_handle.clone(),
        presence: Presence::spawn(insim.clone(), 32),
        game: GameInfo::spawn(insim.clone(), 32),
        leaderboard: Leaderboard::<String>::spawn(32),
        config: Arc::new(config.clone()),
    };

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;
    let _ = insim.send(TinyType::Sst.with_request_id(3)).await;

    loop {
        if scene_handle.is_none() {
            scene_handle = Some(match scene {
                GameState::Idle => tokio::task::spawn(scenes::idle(cx.clone())),
                GameState::TrackRotation { ref combo } => {
                    tokio::task::spawn(scenes::track_rotation(cx.clone(), combo.clone()))
                },
                GameState::Lobby { ref combo } => {
                    tokio::task::spawn(scenes::lobby(cx.clone(), combo.clone()))
                },
                GameState::Round {
                    round, ref combo, ..
                } => tokio::task::spawn(scenes::round(cx.clone(), round, combo.clone())),
                GameState::Victory => tokio::task::spawn(scenes::victory(cx.clone())),
            });
        }

        tokio::select! {
            Some(result) = async {
                match scene_handle.as_mut() {
                    Some(h) => h.await.ok(),
                    None => None,
                }
            } => {
                scene_handle = None;
                scene = result?;
            },

            packet = packets.recv() => {
                match packet? {
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
