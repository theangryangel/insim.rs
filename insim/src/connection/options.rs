use std::net::SocketAddr;
use std::time::Duration;

use crate::codec::Mode;
use crate::packets::insim::{Isi, IsiFlags};

use super::network_options::NetworkOptions;

#[derive(Clone, Default)]
pub struct ConnectionOptions {
    pub name: String,
    pub password: String,
    pub flags: IsiFlags,
    pub prefix: Option<char>,
    pub interval: Duration,

    pub network_options: NetworkOptions,
}

impl ConnectionOptions {
    /// Name of the client, passed to Insim [Isi](crate::packets::insim::Isi).
    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Set any [IsiFlags](crate::packets::insim::IsiFlags)
    pub fn set_flags(mut self, flags: IsiFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set a flag to be used in the [Isi](crate::packets::insim::Isi).
    pub fn set_flag(mut self, flag: IsiFlags) -> Self {
        self.flags |= flag;
        self
    }

    /// Remove all flags from the [Isi](crate::packets::insim::Isi).
    pub fn clear_flags(mut self) -> Self {
        self.flags.clear();
        self
    }

    /// Set the prefix to be used in the [Isi](crate::packets::insim::Isi).
    pub fn password(mut self, pwd: String) -> Self {
        self.password = pwd;
        self
    }

    /// Set the prefix to be used in the [Isi](crate::packets::insim::Isi).
    pub fn prefix(mut self, prefix: char) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// Set the interval between MCI or NLP packets, in milliseconds.
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Create an [Isi](crate::packets::insim::Isi) packet.
    pub fn as_isi(&self) -> Isi {
        Isi {
            iname: self.name.to_owned(),
            admin: self.password.to_owned(),
            prefix: self.prefix.unwrap_or(0 as char),
            version: crate::packets::VERSION,
            interval: self.interval,
            flags: self.flags,
            ..Default::default()
        }
    }

    pub fn relay<H: Into<Option<String>>>(mut self, select_host: H) -> Self {
        self.network_options = NetworkOptions::Relay {
            select_host: select_host.into(),
        };
        self
    }

    pub fn tcp<R: Into<SocketAddr>>(
        mut self,
        remote: R,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    ) -> Self {
        self.network_options = NetworkOptions::Tcp {
            remote: remote.into(),
            codec_mode,
            verify_version,
            wait_for_initial_pong,
        };
        self
    }

    pub fn udp<L: Into<Option<SocketAddr>>, R: Into<SocketAddr>>(
        mut self,
        local: L,
        remote: R,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    ) -> Self {
        self.network_options = NetworkOptions::Udp {
            local: local.into(),
            remote: remote.into(),
            codec_mode,
            verify_version,
            wait_for_initial_pong,
        };
        self
    }
}
