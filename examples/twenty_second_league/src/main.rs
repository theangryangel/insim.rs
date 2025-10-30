//! 20s league
mod combo;
mod components;
mod stages;

use std::{fs, time::Duration};

use anyhow::{Context, Result};
use insim::{WithRequestId, insim::TinyType};
use kitcar::{
    combos::ComboList,
    game::{GameHandle, GameInfo},
    presence::{Presence, PresenceHandle},
    ui,
};

use crate::components::{RootPhase, RootProps};

/// Config
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Insim IName
    pub iname: Option<String>,
    /// Server address
    pub addr: String,
    /// admin password
    pub admin: Option<String>,
    /// Warmup duration
    #[serde(with = "humantime_serde")]
    pub warmup_duration: Duration,
    /// Combinations
    pub combos: ComboList<combo::ComboExt>,
    /// Number of rounds
    pub rounds: Option<usize>,
}

#[derive(Debug, Clone)]
struct MyContext {
    pub ui: ui::ManagerHandle<components::Root>,
    pub presence: PresenceHandle,
    pub game: GameHandle,
}

#[derive(Debug)]
struct MyGame {
    pub insim: insim::builder::SpawnedHandle,
    pub state: MyContext,
    pub desired_state: GameState,
}

#[derive(Debug)]
#[allow(dead_code)] // for exit. For now.
enum GameState {
    Idle,
    Lobby,
    Game,
    Exit,
}

impl MyGame {
    async fn poll(&mut self) -> anyhow::Result<()> {
        loop {
            let mut handle = match self.desired_state {
                GameState::Idle => {
                    tokio::task::spawn(stages::idle(self.insim.clone(), self.state.clone()))
                },
                GameState::Lobby => {
                    tokio::task::spawn(stages::lobby(self.insim.clone(), self.state.clone()))
                },
                GameState::Game => {
                    tokio::task::spawn(stages::game(self.insim.clone(), self.state.clone()))
                },
                GameState::Exit => {
                    break;
                },
            };

            loop {
                tokio::select! {
                    result = &mut handle => {
                        self.desired_state = result??;
                        break;
                    }
                    // TODO: add something
                }
            }
        }

        Ok(())
    }
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

    let config: Config = serde_norway::from_str(
        &fs::read_to_string("config.yaml").context("could not read config.yaml")?,
    )
    .context("Could not parse config.yaml")?;

    let (insim, _join_handle) = insim::tcp(config.addr.as_str())
        .isi_admin_password(config.admin.clone())
        .isi_iname(config.iname.clone())
        .spawn(100)
        .await?;

    let (ui_handle, _ui_thread) = ui::Manager::spawn::<components::Root>(
        insim.clone(),
        RootProps {
            phase: RootPhase::Idle,
            show: true,
        },
    );

    let presence_handle = Presence::spawn(insim.clone(), 32);
    let game_state_handle = GameInfo::spawn(insim.clone(), 32);

    println!("20 Second League started!");

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;
    let _ = insim.send(TinyType::Sst.with_request_id(3)).await;

    let mut game = MyGame {
        insim: insim.clone(),
        desired_state: GameState::Idle,
        state: MyContext {
            ui: ui_handle,
            presence: presence_handle.clone(),
            game: game_state_handle,
        },
    };

    let game_fut = game.poll();
    // pin for cancel safety so we can use a select! loop below.
    tokio::pin!(game_fut);

    let mut packets = insim.subscribe();

    loop {
        tokio::select! {
            result = &mut game_fut => {
                result?;
                break;
            },
            packet = packets.recv() => {
                match packet? {
                    insim::Packet::Mso(mso) => {
                        if_chain::if_chain! {
                            if mso.msg_from_textstart() == "!quit";
                            if let Some(conn_info) = presence_handle.connection(&mso.ucid).await;
                            if conn_info.admin;
                            then {
                                break;
                            }
                        }
                    },
                    _ => {}
                }
            }

        }
    }

    Ok(())
}
