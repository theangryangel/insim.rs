//! Config related stuff

use std::{fs, time::Duration};

use anyhow::Context;
use kitcar::combos::ComboList;

use crate::combo::ComboExt;


/// Config
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Server address
    pub addr: String,
    /// admin password
    pub admin: Option<String>,
    /// Lobby duration
    #[serde(with = "humantime_serde")]
    pub lobby_duration: Duration,
    /// Scores by position
    pub scores_by_position: Vec<i32>,
    /// Combinations
    pub combos: ComboList<ComboExt>,
}

impl Config {
    pub(crate) fn from_file(src: &str) -> anyhow::Result<Self> {
        let config: Config = serde_norway::from_str(
            &fs::read_to_string(src).context("could not read config.yaml")?,
        )
        .context("Could not parse config.yaml")?;

        Ok(config)
    }
}

