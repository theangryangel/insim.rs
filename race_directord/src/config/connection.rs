use std::net::SocketAddr;

use insim::{connection::Connection, insim::Isi};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum ConnectionConfig {
    #[serde(rename_all = "kebab-case")]
    Tcp { addr: SocketAddr },
    #[serde(rename_all = "kebab-case")]
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

impl ConnectionConfig {
    pub(crate) fn into_connection(&self) -> Connection {
        match self {
            ConnectionConfig::Relay {
                auto_select_host,
                websocket,
                spectator,
                admin,
            } => insim::connection::Connection::relay(
                auto_select_host.clone(),
                *websocket,
                spectator.clone(),
                admin.clone(),
                Isi::default(),
            ),
            ConnectionConfig::Tcp { addr } => insim::connection::Connection::tcp(
                insim::codec::Mode::Compressed,
                *addr,
                true,
                Isi::default(),
            ),

            ConnectionConfig::Udp { bind, addr } => insim::connection::Connection::udp(
                *bind,
                *addr,
                insim::codec::Mode::Compressed,
                true,
                Isi::default(),
            ),
        }
    }
}
