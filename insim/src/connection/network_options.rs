use std::net::SocketAddr;

use crate::codec::Mode;

#[derive(Clone)]
pub enum NetworkOptions {
    Tcp {
        remote: SocketAddr,
        codec_mode: Mode,
        verify_version: bool,
    },
    Udp {
        local: Option<SocketAddr>,
        remote: SocketAddr,
        codec_mode: Mode,
        verify_version: bool,
    },
    Relay {
        select_host: Option<String>,
        spectator_password: Option<String>,
        websocket: bool,
    },
}

impl Default for NetworkOptions {
    fn default() -> Self {
        Self::Tcp {
            remote: "127.0.0.1:29999".parse().unwrap(),
            codec_mode: Mode::Compressed,
            verify_version: true,
        }
    }
}
