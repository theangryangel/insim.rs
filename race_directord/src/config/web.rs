use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct WebConfig {
    listen: Option<SocketAddr>,
}

