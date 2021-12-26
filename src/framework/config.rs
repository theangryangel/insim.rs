use super::protocol::codec::Mode;
use super::protocol::insim::InitFlags;
use super::Client;

/// Configuration and [Client] builder.
pub struct Config {
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) password: String,
    pub(crate) flags: InitFlags,
    pub(crate) prefix: u8,
    pub(crate) interval_ms: u16,
    pub(crate) verify_version: bool,
    //pub(crate) reconnect: bool,
    //pub(crate) max_reconnect_attempts: u16,
    pub(crate) codec_mode: Mode,
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
            // TODO: Readd support for reconnection attempts
            //reconnect: true,
            //max_reconnect_attempts: 1,
            codec_mode: Mode::Compressed,
        }
    }

    /// Use a TCP connection, to a given "host:port".
    pub fn tcp(mut self, host: String) -> Self {
        self.host = host;
        self
    }

    /// Use the Insim Relay.
    pub fn relay(mut self) -> Self {
        self.host = "isrelay.lfs.net:47474".into();
        // TODO: Talk to LFS devs, find out if/when relay gets compressed support?
        self.codec_mode = Mode::Uncompressed;
        self.verify_version = false;
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
    pub fn prefix(mut self, prefix: u8) -> Self {
        self.prefix = prefix;
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

    pub fn build(self) -> Client<()> {
        Client::new(self)
    }

    pub fn build_with_state<State>(self, state: State) -> Client<State>
    where
        State: Clone + Send + Sync + 'static,
    {
        Client::with_state(self, state)
    }
}
