use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum ConnectionConfig {
    #[serde(rename_all = "kebab-case")]
    Tcp {
        addr: SocketAddr,
        connection_attempts: Option<usize>,
    },
    #[serde(rename_all = "kebab-case")]
    Udp {
        bind: Option<SocketAddr>,
        addr: SocketAddr,
        connection_attempts: Option<usize>,
    },
    #[serde(rename_all = "kebab-case")]
    Relay {
        auto_select_host: Option<String>,
        websocket: bool,
        admin: Option<String>,
        spectator: Option<String>,
        connection_attempts: Option<usize>,
    },
}
