//! Twenty Second League
use std::{fs, time::Duration};

use eyre::Context as _;
use kitcar::Workshop;

use crate::combo::ComboCollection;

mod combo;
mod cpa;
mod dictator;
mod league;

/// Config
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Insim IName
    pub iname: String,
    /// Server address
    pub addr: String,
    /// admin password
    pub admin: Option<String>,
    /// Combinations
    pub combo: combo::ComboCollection,
    /// tick rate (hz)
    pub tick_rate: Option<u64>,
}

fn main() -> eyre::Result<()> {
    let config: Config = serde_norway::from_str(
        &fs::read_to_string("config.yaml").wrap_err("could not read config.yaml")?,
    )
    .wrap_err("Could not parse config.yaml")?;

    Workshop::<ComboCollection, (), (), ()>::new(config.combo.clone())
        .add_engine(league::League::Idle)
        .add_engine(cpa::Cpa)
        .add_engine(dictator::NoVote)
        .ignition(
            insim::tcp(config.addr)
                .isi_iname(config.iname)
                .isi_admin_password(config.admin)
                .isi_prefix('!')
                .set_non_blocking(true),
        )
        .wrap_err("Failed to execute kitcar ignition")?
        .run(Duration::from_millis(1000 / config.tick_rate.unwrap_or(64)));

    Ok(())
}
