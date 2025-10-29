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
struct MyState {
    pub ui: ui::ManagerHandle<components::Root>,
    pub presence: PresenceHandle,
    pub game: GameHandle,
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

    kitcar::runtime::Runtime::new(
        insim.clone(),
        MyState {
            ui: ui_handle,
            presence: presence_handle,
            game: game_state_handle,
        },
    )
    .ignite(stages::idle)
    .await?;
    Ok(())
}
