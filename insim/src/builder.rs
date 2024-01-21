use std::{fmt::Debug, net::SocketAddr, time::Duration};

use insim_core::identifiers::RequestId;
use tokio::{io::BufWriter, time::timeout};

use crate::{
    insim::{Isi, IsiFlags},
    net::{Codec, Framed, FramedInner, Mode},
    relay::HostSelect,
    result::Result,
};

#[derive(Clone, Debug, Default)]
pub enum Proto {
    #[default]
    Tcp,
    Udp,
    Relay,
}

#[derive(Debug)]
pub struct Builder {
    proto: Proto,

    connect_timeout: Duration,
    handshake_timeout: Duration,

    remote: SocketAddr,
    verify_version: bool,
    mode: Mode,

    isi_admin_password: Option<String>,
    isi_flags: IsiFlags,
    isi_prefix: Option<char>,
    isi_interval: Option<Duration>,
    isi_iname: Option<String>,
    isi_reqi: RequestId,

    // Choosing to use separate fields with a prefix, rather than an enum because if you were to do
    // something like this:
    //  Builder::new().relay().relay_admin_password("123").udp(None).relay()
    // the user's expectation would not be to loose all the previous relay configuration.
    // Why would they do this? Absolutely no idea. However, by separating out the fields, it
    // massively simplifies things for us.
    tcp_nodelay: bool,
    udp_local_address: Option<SocketAddr>,

    relay_select_host: Option<String>,
    relay_spectator_password: Option<String>,
    relay_admin_password: Option<String>,

    #[cfg(feature = "websocket")]
    relay_websocket: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            handshake_timeout: Duration::from_secs(30),

            proto: Proto::Tcp,
            remote: "127.0.0.1:29999".parse().unwrap(),
            verify_version: true,
            mode: Mode::Compressed,

            tcp_nodelay: true,
            udp_local_address: None,

            relay_select_host: None,
            relay_spectator_password: None,
            relay_admin_password: None,

            #[cfg(feature = "websocket")]
            relay_websocket: false,

            isi_admin_password: None,

            isi_flags: IsiFlags::default(),
            isi_prefix: None,
            isi_iname: None,
            isi_interval: None,
            isi_reqi: RequestId(0),
        }
    }
}

impl Builder {
    /// Constructs a new `Builder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a TCP connection
    pub fn tcp<R: Into<SocketAddr>>(mut self, remote_addr: R) -> Self {
        self.proto = Proto::Tcp;
        self.remote = remote_addr.into();
        self
    }

    /// Use a UDP connection
    pub fn udp<L: Into<Option<SocketAddr>>, R: Into<SocketAddr>>(
        mut self,
        remote_addr: R,
        local_addr: L,
    ) -> Self {
        self.proto = Proto::Udp;
        self.remote = remote_addr.into();
        self.udp_local_address = local_addr.into();
        self
    }

    /// Use the LFS World Relay over TCP
    pub fn relay(mut self) -> Self {
        self.proto = Proto::Relay;
        self
    }

    /// Use the LFS World Relay over Websockets.
    #[cfg(feature = "websocket")]
    pub fn relay_websocket(mut self, ws: bool) -> Self {
        self.relay_websocket = ws;
        self
    }

    /// Set the connection timeout.
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = duration;
        self
    }

    /// Insim 9+ allows for a "compressed" and "uncompressed" packet size mode.
    /// When "compressed" the size on the wire is indicated as "true size / 4".
    /// The LFS World relay does not currently appear to support compressed mode.
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Use "compressed" mode.
    pub fn compressed(self) -> Self {
        self.mode(Mode::Compressed)
    }

    /// Use "uncompressed" mode. Force enabled for LFS World relay.
    pub fn uncompressed(self) -> Self {
        self.mode(Mode::Uncompressed)
    }

    /// Enable the verification of the Insim version within the library. If a [crate::Packet::Version] is received with a differing version, [crate::Error::IncompatibleVersion] is returned and the connection is lost.
    pub fn verify_version(mut self, verify: bool) -> Self {
        self.verify_version = verify;
        self
    }

    /// Set whether sockets have `TCP_NODELAY` enabled.
    /// Default is `true`
    pub fn tcp_nodelay(mut self, no_delay: bool) -> Self {
        self.tcp_nodelay = no_delay;
        self
    }

    /// Automatically select a host after connection to the LFS World relay.
    /// This is not verified. If the host is not online, or registered with the LFS World relay, it
    /// is currently your responsibility to handle this.
    pub fn relay_select_host<H: Into<Option<String>>>(mut self, host: H) -> Self {
        self.relay_select_host = host.into();
        self
    }

    /// Set the spectator password to use when connecting to the host via the LFS World Relay.
    pub fn relay_spectator_password<P: Into<Option<String>>>(mut self, password: P) -> Self {
        self.relay_spectator_password = password.into();
        self
    }

    /// Set the admin password to use when connecting to the host via the LFS World Relay.
    pub fn relay_admin_password<P: Into<Option<String>>>(mut self, password: P) -> Self {
        self.relay_admin_password = password.into();
        self
    }

    /// Set the admin password to be used in the [crate::Packet::Init] packet during connection
    /// handshake.
    pub fn isi_admin_password<P: Into<Option<String>>>(mut self, password: P) -> Self {
        self.isi_admin_password = password.into();
        self
    }

    /// Set the [crate::core::identifiers::RequestId] to be used in the [crate::Packet::Init] packet during connection
    /// handshake.
    pub fn isi_reqi(mut self, i: RequestId) -> Self {
        self.isi_reqi = i;
        self
    }

    /// Set the [crate::insim::IsiFlags] to be used in the [crate::Packet::Init] packet during connection
    /// handshake.
    pub fn isi_flags(mut self, flags: IsiFlags) -> Self {
        self.isi_flags = flags;
        self
    }

    /// Set the prefix to be used in the [crate::Packet::Init] packet during connection
    /// handshake.
    pub fn isi_prefix<C: Into<Option<char>>>(mut self, c: C) -> Self {
        self.isi_prefix = c.into();
        self
    }

    /// Set the iname to be used in the [crate::Packet::Init] packet during connection
    /// handshake.
    pub fn isi_iname<N: Into<Option<String>>>(mut self, iname: N) -> Self {
        self.isi_iname = iname.into();
        self
    }

    /// Set the interval to be used in the [crate::Packet::Init] packet during connection
    /// handshake.
    /// This governs the time between [crate::Packet::MultiCarInfo] or [crate::Packet::NodeLap]
    /// packets.
    pub fn isi_interval<D: Into<Option<Duration>>>(mut self, duration: D) -> Self {
        self.isi_interval = duration.into();
        self
    }

    /// Create a [crate::insim::Isi] from this configuration.
    pub fn isi(&self) -> Isi {
        let udpport = match self.proto {
            Proto::Udp => self.udp_local_address.unwrap().port(),
            _ => 0,
        };

        Isi {
            reqi: self.isi_reqi,
            udpport,
            flags: self.isi_flags,
            admin: self.isi_admin_password.as_deref().unwrap_or("").to_owned(),
            iname: self.isi_iname.as_deref().unwrap_or("").to_owned(),
            prefix: self.isi_prefix.unwrap_or(0 as char),
            interval: self.isi_interval.unwrap_or(Duration::ZERO),
            ..Default::default()
        }
    }

    /// Attempt to establish (connect and handshake) a valid Insim connection using this
    /// configuration.
    /// The `Builder` is not consumed and may be reused.
    pub async fn connect(&self) -> Result<Framed> {
        match self.proto {
            Proto::Tcp => {
                let stream = timeout(
                    self.connect_timeout,
                    tokio::net::TcpStream::connect(self.remote),
                )
                .await??;
                stream.set_nodelay(self.tcp_nodelay)?;

                let stream = BufWriter::new(stream);

                let mut stream = FramedInner::new(stream, Codec::new(self.mode.clone()));
                stream.verify_version(self.verify_version);
                stream.handshake(self.isi(), self.handshake_timeout).await?;

                Ok(Framed::BufferedTcp(stream))
            }
            Proto::Udp => {
                let local = self.udp_local_address.unwrap_or("0.0.0.0:0".parse()?);

                let stream = tokio::net::UdpSocket::bind(local).await?;
                stream.connect(self.remote).await.unwrap();

                let mut isi = self.isi();
                if self.udp_local_address.is_none() {
                    isi.udpport = local.port();
                }

                let mut stream = FramedInner::new(stream, Codec::new(self.mode.clone()));
                stream.verify_version(self.verify_version);
                stream.handshake(isi, self.handshake_timeout).await?;

                Ok(Framed::Udp(stream))
            }
            Proto::Relay => {
                let mut stream = self._connect_relay().await?;

                if let Some(hostname) = &self.relay_select_host {
                    let packet = HostSelect {
                        hname: hostname.to_string(),
                        admin: self
                            .relay_admin_password
                            .as_deref()
                            .unwrap_or("")
                            .to_owned(),
                        spec: self
                            .relay_spectator_password
                            .as_deref()
                            .unwrap_or("")
                            .to_owned(),
                        ..Default::default()
                    };

                    stream.write(packet).await?;
                }

                Ok(stream)
            }
        }
    }

    async fn _connect_relay(&self) -> Result<Framed> {
        #[cfg(feature = "websocket")]
        if self.relay_websocket {
            let stream = timeout(
                self.connect_timeout,
                crate::net::websocket::connect_to_relay(self.tcp_nodelay),
            )
            .await??;

            let mut inner = FramedInner::new(stream, Codec::new(Mode::Uncompressed));
            inner.verify_version(false);
            return Ok(Framed::WebSocket(inner));
        }

        let stream = timeout(
            self.connect_timeout,
            tokio::net::TcpStream::connect(crate::LFSW_RELAY_ADDR),
        )
        .await??;
        stream.set_nodelay(self.tcp_nodelay)?;

        let mut inner = FramedInner::new(stream, Codec::new(Mode::Uncompressed));
        inner.verify_version(false);
        Ok(Framed::Tcp(inner))
    }
}
