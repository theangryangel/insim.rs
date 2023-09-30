pub(crate) mod peer;
pub(crate) mod web;

use crate::Result;

use std::{net::SocketAddr, collections::HashMap, path::Path, fs};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub web: web::WebConfig,
    pub peers: HashMap<String, peer::PeerConfig>,
}

impl Config {
    pub(crate) fn from_file(file: &str) -> Result<Self> {
        let contents = fs::read_to_string(file)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

