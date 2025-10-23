//! 20s league
mod combo;
mod components;
mod no_vote;
mod phases;

use std::{fs, time::Duration};

use anyhow::{Context, Result};
use insim::{Packet, WithRequestId, core::track::Track, insim::TinyType};
use kitcar::{combos::ComboList, game::GameInfo, presence::Presence, ui};
use tokio::signal;

use crate::{
    components::{RootPhase, RootProps},
    phases::Transition,
};

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

    let mut packets = insim.subscribe();

    let _ = insim.send(TinyType::Ncn.with_request_id(1)).await;
    let _ = insim.send(TinyType::Npl.with_request_id(2)).await;
    let _ = insim.send(TinyType::Sst.with_request_id(3)).await;

    // Why not avoid async? Because the state machine for the game gets a bit difficult to follow.
    // We want to allow the ability to do things like "wait_for_restart().await?", or
    // "wait_for_players().await?" without impacting the legibility of the code.

    // Why no pinning? Because it's an utter ballache. I've got some very specific requirements
    // where I want to setup things like "wait_for_restart().await?" as a function. However, doing
    // so then causes the issue where we don't want to stop handling packets. This in turn makes
    // the implementation a lot more fiddly. By simply spawning a task per-phase we shortcut a
    // whole bunch of problems. Laziness for the win!

    let mut task = phases::PhaseIdle::spawn(insim.clone(), presence_handle.clone());
    let mut current_phase = Transition::Idle;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                // TODO: graceful handling
                task.abort();
                break;
            },

            // Task completed naturally
            result = &mut task => {
                let next_phase = result?;
                current_phase = next_phase;

                task = match next_phase {
                    Transition::Idle => phases::PhaseIdle::spawn(insim.clone(), presence_handle.clone()),
                    Transition::Lobby => phases::PhaseLobby::spawn(insim.clone(), presence_handle.clone(), ui_handle.clone()),
                    Transition::Game => phases::PhaseGame::spawn(insim.clone(), presence_handle.clone(), ui_handle.clone()),
                    Transition::Shutdown => break,
                };
            },

            // External packet forcing transition
            packet = packets.recv() => {
                if let Packet::Mso(mso) = packet? {
                    if_chain::if_chain! {
                        if mso.msg_from_textstart() == "!start";
                        if let Some(conn_info) = presence_handle.connection(&mso.ucid).await;
                        if conn_info.admin;
                        if current_phase != Transition::Game;
                        then {
                            // Abort current task and start new one
                            task.abort();

                            println!("Transitioning to game");

                            let _ = insim.send_command("/end").await;
                            println!("Waiting for end state");
                            game_state_handle.wait_for_end().await;

                            println!("REquesting track change");
                            let _ = insim.send_command("/track FE1").await;
                            println!("Waiting for track");
                            game_state_handle.wait_for_track(Track::Fe1).await;

                            println!("Waiting for game to start");
                            game_state_handle.wait_for_racing().await;

                            task = phases::PhaseLobby::spawn(insim.clone(), presence_handle.clone(), ui_handle.clone());
                            current_phase = Transition::Lobby;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
