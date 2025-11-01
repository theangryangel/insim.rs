//! 20s league
mod combo;
mod components;
mod scenes;
mod config;
mod chat;

use anyhow::Result;
use humantime_serde::re::humantime::Duration;
use insim::{identifiers::{ConnectionId, PlayerId}, insim::TinyType, WithRequestId};
use kitcar::{
    combos::Combo, game::GameInfo, leaderboard::Leaderboard, presence::Presence, ui
};
use tokio::task::JoinHandle;

use crate::{chat::MyChatCommands, components::RootProps};

type MyUi = ui::ManagerHandle<components::Root>;

#[derive(Debug, Clone)]
#[allow(dead_code)] // for exit. For now.
enum GameState {
    Idle,
    TrackRotation { combo: Combo<combo::ComboExt> },
    Lobby { combo: Combo<combo::ComboExt> },
    Round { round: u32, combo: Combo<combo::ComboExt>, remaining: Duration },
    Victory,
    Exit,
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
        .spawn(100)
        .await?;

    let (ui_handle, _ui_thread) = ui::Manager::spawn::<components::Root>(
        insim.clone(),
        RootProps {
            phase: GameState::Idle,
            show: true,
        },
    );

    let presence_handle = Presence::spawn(insim.clone(), 32);
    let game_state_handle = GameInfo::spawn(insim.clone(), 32);
    let leaderboard_handle = Leaderboard::<PlayerId>::spawn(32);

    println!("20 Second League started!");

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;
    let _ = insim.send(TinyType::Sst.with_request_id(3)).await;

    let mut packets = insim.subscribe();
    let mut scene_handle: Option<JoinHandle<anyhow::Result<GameState>>> = None;
    let mut scene = GameState::Idle;

    loop {
        if scene_handle.is_none() {
            scene_handle = Some(match scene {
                GameState::Idle => {
                    tokio::task::spawn(scenes::idle(
                        insim.clone(), 
                        presence_handle.clone(), 
                        leaderboard_handle.clone(), 
                        config.combos.clone()
                    ))
                },
                GameState::TrackRotation { ref combo } => {
                    tokio::task::spawn(scenes::track_rotation(
                        insim.clone(), combo.clone(), game_state_handle.clone()
                    ))
                }
                GameState::Lobby { ref combo } => {
                    tokio::task::spawn(scenes::lobby(
                        insim.clone(), combo.clone(), ui_handle.clone(), config.lobby_duration
                    ))
                },
                GameState::Round { round, ref combo, .. } => {
                    tokio::task::spawn(scenes::round(
                        insim.clone(), 
                        leaderboard_handle.clone(), 
                        round, 
                        combo.clone(), 
                        game_state_handle.clone(), 
                        ui_handle.clone(),
                        config.scores_by_position.clone(),
                    ))
                },
                GameState::Victory => {
                    tokio::task::spawn(scenes::victory(insim.clone(), leaderboard_handle.clone()))
                },
                GameState::Exit => {
                    break;
                },
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

                        match MyChatCommands::parse_with_prefix(mso.msg_from_textstart(), Some('!')) {
                            Ok(MyChatCommands::Quit) => {
                                if_chain::if_chain! {
                                    if let Some(conn_info) = presence_handle.connection(&mso.ucid).await;
                                    if conn_info.admin;
                                    then {
                                        insim.send_message("Quitting.. bye!", ConnectionId::ALL).await?;
                                        break;
                                    }
                                }
                            },
                            Ok(MyChatCommands::Help) => {
                                println!("{:?}", MyChatCommands::help());
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
