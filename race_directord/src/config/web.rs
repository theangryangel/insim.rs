use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct WebConfig {
    pub enabled: bool,
    pub address: Option<SocketAddr>,
}
