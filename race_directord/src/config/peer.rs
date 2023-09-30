use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum PeerConfig {
    Tcp {
        addr: SocketAddr,
    },
    Udp {
        bind: Option<SocketAddr>,
        addr: SocketAddr,
    },
    #[serde(rename_all = "kebab-case")]
    Relay {
        auto_select_host: Option<String>,
        websocket: bool,
        admin: Option<String>,
        spectator: Option<String>,
    },
}
