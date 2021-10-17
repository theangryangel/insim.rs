use crate::client::{Client, TransportType};

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) ctype: TransportType,
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) password: String,
    pub(crate) flags: u16,
    pub(crate) prefix: u8,
    pub(crate) interval_ms: u16,
    pub(crate) reconnect: bool,
    pub(crate) max_reconnect_attempts: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config::new()
    }
}

impl Config {
    // Builder functions
    pub fn new() -> Self {
        Self {
            ctype: TransportType::Tcp,
            name: "insim.rs".into(),
            host: "127.0.0.1:29999".into(),
            password: "".into(),
            flags: (1 << 5), // TODO make a builder
            prefix: 0,
            interval_ms: 1000,
            reconnect: true,
            max_reconnect_attempts: 1,
        }
    }

    pub fn tcp(mut self, host: String) -> Self {
        self.ctype = TransportType::Tcp;
        self.host = host;
        self
    }

    pub fn relay(mut self) -> Self {
        self.ctype = TransportType::Tcp;
        self.host = "isrelay.lfs.net:47474".into();
        self
    }

    pub fn udp(mut self, host: String) -> Self {
        self.ctype = TransportType::Udp;
        self.host = host;
        self
    }

    pub fn named(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn set_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    pub fn clear_flags(mut self) -> Self {
        self.flags = 0;
        self
    }

    pub fn password(mut self, pwd: String) -> Self {
        self.password = pwd;
        self
    }

    pub fn prefix(mut self, prefix: u8) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn interval(mut self, interval: u16) -> Self {
        // TODO take a Duration and automatically convert it
        self.interval_ms = interval;
        self
    }

    pub async fn build(self) -> Client {
        Client::from_config(self)
    }
}
