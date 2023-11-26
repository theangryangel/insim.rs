use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct WebConfig {
    pub listen: Option<SocketAddr>,
}
