use std::time::Duration;

use futures::SinkExt;

use tokio::net::ToSocketAddrs;
use tokio::{net::TcpStream, time::timeout};
use tokio_util::codec::Framed;

use crate::client::Client;
use crate::codec::{Codec, Mode};
use crate::core::identifiers::RequestId;
use crate::packets::insim::{Isi, IsiFlags};
use crate::result::Result;
use crate::udp_stream::UdpStream;

use super::{TcpClientTransport, UdpClientTransport};

#[derive(Debug)]
/// Configuration and [Client] builder.
pub struct ClientBuilder {
    pub name: String,
    pub password: String,
    pub flags: IsiFlags,
    pub prefix: Option<char>,
    pub interval: Duration,
    pub verify_version: bool,
    pub wait_for_initial_pong: bool,
    pub codec_mode: Mode,
    pub connect_timeout: Duration,
    pub udp_port: Option<u16>,
}

impl Default for ClientBuilder {
    fn default() -> ClientBuilder {
        ClientBuilder::new()
    }
}

impl ClientBuilder {
    /// Create a default configuration instance.
    pub fn new() -> Self {
        Self {
            name: "insim.rs".into(),
            password: "".into(),
            flags: IsiFlags::MCI | IsiFlags::CON | IsiFlags::OBH,
            prefix: None,
            interval: Duration::from_millis(1000),
            verify_version: true,
            wait_for_initial_pong: true,
            codec_mode: Mode::Compressed,
            connect_timeout: Duration::from_secs(10),
            udp_port: None,
        }
    }
}

impl ClientBuilder {
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

    /// Set the codec mode to use Insim v9 "compressed" packet lengths.
    /// If you select to connect to the LFS World Relay this will be overridden for compatibility.
    pub fn use_compressed_header_byte(mut self) -> Self {
        self.codec_mode = Mode::Compressed;
        self
    }

    /// Set the codec mode to use Insim <= v8 "uncompressed" packet lengths.
    /// If you select to connect to the LFS World Relay this will be overridden for compatibility.
    pub fn use_uncompressed_header_byte(mut self) -> Self {
        self.codec_mode = Mode::Uncompressed;
        self
    }

    /// Should the Client verify the version of the server?
    /// If you select to connect to the LFS World Relay this will be overridden for compatibility.
    pub fn verify_version(mut self, value: bool) -> Self {
        self.verify_version = value;
        self
    }

    /// Should the Client wait for the initial pong?
    /// If you select to connect to the LFS World Relay this will be overridden for compatibility.
    pub fn wait_for_initial_pong(mut self, value: bool) -> Self {
        self.wait_for_initial_pong = value;
        self
    }

    /// Create an [Isi](crate::packets::insim::Isi) packet.
    pub fn as_isi(&self) -> Isi {
        Isi {
            name: self.name.to_owned(),
            password: self.password.to_owned(),
            prefix: self.prefix.unwrap_or(0 as char),
            version: crate::packets::VERSION,
            interval: self.interval,
            flags: self.flags,
            reqi: if self.verify_version {
                RequestId(1)
            } else {
                RequestId(0)
            },
            udpport: self.udp_port.unwrap_or(0),
        }
    }

    /// Connect to Insim via TCP and return a new [Client](crate::client::Client).
    #[cfg(feature = "tcp")]
    pub async fn connect_tcp<A: ToSocketAddrs>(
        &mut self,
        remote: A,
    ) -> Result<Client<TcpClientTransport>> {
        let stream = timeout(self.connect_timeout, TcpStream::connect(remote)).await??;

        let stream = Framed::new(stream, Codec::new(self.codec_mode));

        let mut stream = Client::new(stream);
        stream
            .handshake_unpin(
                self.connect_timeout,
                self.as_isi(),
                self.wait_for_initial_pong,
                self.verify_version,
            )
            .await?;
        Ok(stream)
    }

    /// Connect to Insim via UDP and return a new [Client](crate::client::Client).
    #[cfg(feature = "udp")]
    pub async fn connect_udp<A: ToSocketAddrs, B: ToSocketAddrs>(
        &mut self,
        local: A,
        remote: B,
    ) -> Result<Client<UdpClientTransport>> {
        let stream = UdpStream::connect(local, remote).await?;

        self.udp_port = stream.local_addr()?.port().into();

        let stream = Framed::new(stream, Codec::new(self.codec_mode));

        let mut stream = Client::new(stream);
        stream
            .handshake_unpin(
                self.connect_timeout,
                self.as_isi(),
                self.wait_for_initial_pong,
                self.verify_version,
            )
            .await?;
        Ok(stream)
    }

    /// Connect to Insim via LFS World Relay and return a new [Client](crate::client::Client).
    /// Optionally automatically select a host.
    /// Warning: Several options will be automatically set to maintain compatibility with LFS World.
    #[cfg(feature = "relay")]
    pub async fn connect_relay<'a, H>(
        &mut self,
        auto_select_host: H,
    ) -> Result<Client<TcpClientTransport>>
    where
        H: Into<Option<&'a str>>,
    {
        // TODO: Talk to LFS devs, find out if/when relay gets compressed support?
        self.codec_mode = Mode::Uncompressed;

        // Relay does not respond to version requests until after the host is selected
        self.verify_version = false;

        // Relay does not respond to ping requests
        self.wait_for_initial_pong = false;

        let mut stream = self.connect_tcp("isrelay.lfs.net:47474").await?;

        if let Some(hostname) = auto_select_host.into() {
            stream
                .send(crate::packets::relay::HostSelect {
                    hname: hostname.to_string(),
                    ..Default::default()
                })
                .await?;
        }

        Ok(stream)
    }
}