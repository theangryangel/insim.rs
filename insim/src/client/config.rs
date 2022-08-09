use super::connection::Client;
use crate::protocol::{codec::Mode, insim::InitFlags};

/// Configuration and [Client] builder.
pub struct Config {
    pub name: String,
    pub host: String,
    pub password: String,
    pub flags: InitFlags,
    pub prefix: u8,
    pub interval_ms: u16,
    pub verify_version: bool,
    pub reconnect: bool,
    pub max_reconnect_attempts: usize,
    pub codec_mode: Mode,
    pub select_relay_host: Option<String>,
    pub connect_timeout_ms: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config::new()
    }
}

impl Config {
    /// Create a default configuration instance.
    pub fn new() -> Self {
        Self {
            name: "insim.rs".into(),
            host: "127.0.0.1:29999".into(),
            password: "".into(),
            flags: InitFlags::MCI | InitFlags::CON | InitFlags::OBH,
            prefix: 0,
            interval_ms: 1000,
            verify_version: true,
            reconnect: true,
            max_reconnect_attempts: 2,
            codec_mode: Mode::Compressed,
            select_relay_host: None,
            connect_timeout_ms: 10000,
        }
    }
}

impl Config {
    /// Use a TCP connection, to a given "host:port".
    pub fn tcp(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    /// Use the Insim Relay.
    pub fn relay(mut self, host: Option<String>) -> Self {
        self.host = "isrelay.lfs.net:47474".into();
        // TODO: Talk to LFS devs, find out if/when relay gets compressed support?
        self.codec_mode = Mode::Uncompressed;
        self.verify_version = false;
        self.select_relay_host = host;
        self
    }

    /// Use a UDP connection.
    pub fn udp(self, _host: String) -> Self {
        unimplemented!("UDP support is not yet available.");
    }

    /// Name of the client, passed to Insim [Init](super::protocol::insim::Init).
    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn set_flags(mut self, flags: InitFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set a flag to be used in the [Init](super::protocol::insim::Init).
    pub fn set_flag(mut self, flag: InitFlags) -> Self {
        self.flags |= flag;
        self
    }

    /// Remove all flags from the [Init](super::protocol::insim::Init).
    pub fn clear_flags(mut self) -> Self {
        self.flags.clear();
        self
    }

    /// Set the prefix to be used in the [Init](super::protocol::insim::Init).
    pub fn password(mut self, pwd: String) -> Self {
        self.password = pwd;
        self
    }

    /// Set the prefix to be used in the [Init](super::protocol::insim::Init).
    pub fn prefix(mut self, prefix: char) -> Self {
        self.prefix = prefix as u8;
        self
    }

    /// Set the interval between MCI or NLP packets, in milliseconds.
    pub fn interval(mut self, interval: u16) -> Self {
        // TODO take a Duration and automatically convert it
        self.interval_ms = interval;
        self
    }

    /// Set the codec mode to use Insim v9 "compressed" packet lengths.
    pub fn use_compressed_header_byte(mut self) -> Self {
        self.codec_mode = Mode::Compressed;
        self
    }

    /// Set the codec mode to use Insim <= v8 "uncompressed" packet lengths.
    pub fn use_uncompressed_header_byte(mut self) -> Self {
        self.codec_mode = Mode::Uncompressed;
        self
    }

    /// Should the client try to reconnect?
    pub fn try_reconnect(mut self, value: bool) -> Self {
        self.reconnect = value;
        self
    }

    /// Set the maximum number of reconnection attempts.
    pub fn try_reconnect_attempts(mut self, value: usize) -> Self {
        if !self.reconnect {
            self.reconnect = true;
        }
        self.max_reconnect_attempts = value;
        self
    }

    /// Should the Client verify the version of the server?
    pub fn verify_version(mut self, value: bool) -> Self {
        self.verify_version = value;
        self
    }

    /// Build a [Client] instance.
    pub fn into_client(self) -> Client {
        Client::new(self)
    }
}
