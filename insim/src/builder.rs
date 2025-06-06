//! Tools to build a connection to LFS using Insim
use std::{
    fmt::Debug,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
use crate::net::blocking_impl::Framed as BlockingFramed;
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
use crate::net::tokio_impl::Framed as AsyncFramed;
use crate::{
    address::Addr,
    identifiers::RequestId,
    insim::{Isi, IsiFlags},
    net::{Codec, Mode},
    result::Result,
};

fn tcpstream_connect_to_any<A: ToSocketAddrs>(
    addrs: A,
    timeout: Duration,
) -> std::io::Result<std::net::TcpStream> {
    for addr in addrs.to_socket_addrs()? {
        match std::net::TcpStream::connect_timeout(&addr, timeout) {
            Ok(stream) => return Ok(stream),
            Err(_) => {
                continue;
            },
        }
    }

    Err(std::io::Error::other("All connection attempts failed"))
}

#[derive(Clone, Debug, Default)]
enum Proto {
    #[default]
    Tcp,
    Udp,
    Relay,
}

#[derive(Debug)]
/// Builder to help you connect to Insim
pub struct Builder {
    proto: Proto,

    connect_timeout: Duration,

    remote: Addr,
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

    relay_websocket: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),

            proto: Proto::Tcp,
            remote: ("127.0.0.1", 29999_u16).into(),
            mode: Mode::Compressed,

            tcp_nodelay: true,
            udp_local_address: None,

            relay_select_host: None,
            relay_spectator_password: None,
            relay_admin_password: None,

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
    pub fn tcp<R: Into<Addr>>(mut self, remote_addr: R) -> Self {
        self.proto = Proto::Tcp;
        self.remote = remote_addr.into();
        self
    }

    /// Use a UDP connection
    pub fn udp<L: Into<Option<SocketAddr>>, R: Into<Addr>>(
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

    /// Set the admin password to be used in the [crate::Packet::Isi] packet during connection
    /// handshake.
    pub fn isi_admin_password<P: Into<Option<String>>>(mut self, password: P) -> Self {
        self.isi_admin_password = password.into();
        self
    }

    /// Set the [crate::identifiers::RequestId] to be used in the [crate::Packet::Isi] packet during connection
    /// handshake.
    pub fn isi_reqi(mut self, i: RequestId) -> Self {
        self.isi_reqi = i;
        self
    }

    /// Set the [crate::insim::IsiFlags] to be used in the [crate::Packet::Isi] packet during connection
    /// handshake.
    pub fn isi_flags(mut self, flags: IsiFlags) -> Self {
        self.isi_flags = flags;
        self
    }

    /// Set the [IsiFlags::MCI] flag
    pub fn isi_flag_mci(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::MCI, enabled);
        self
    }

    /// Set the [IsiFlags::LOCAL] flag
    pub fn isi_flag_local(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::LOCAL, enabled);
        self
    }

    /// Set the [IsiFlags::MSO_COLS] flag
    pub fn isi_flag_mso_cols(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::MSO_COLS, enabled);
        self
    }

    /// Set the [IsiFlags::NLP] flag
    pub fn isi_flag_nlp(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::NLP, enabled);
        self
    }

    /// Set the [IsiFlags::CON] flag
    pub fn isi_flag_con(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::CON, enabled);
        self
    }

    /// Set the [IsiFlags::OBH] flag
    pub fn isi_flag_obh(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::OBH, enabled);
        self
    }

    /// Set the [IsiFlags::HLV] flag
    pub fn isi_flag_hlv(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::HLV, enabled);
        self
    }

    /// Set the [IsiFlags::AXM_LOAD] flag
    pub fn isi_flag_axm_load(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::AXM_LOAD, enabled);
        self
    }

    /// Set the [IsiFlags::AXM_EDIT] flag
    pub fn isi_flag_axm_edit(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::AXM_EDIT, enabled);
        self
    }

    /// Set the [IsiFlags::REQ_JOIN] flag
    pub fn isi_flag_req_join(mut self, enabled: bool) -> Self {
        self.isi_flags.set(IsiFlags::REQ_JOIN, enabled);
        self
    }

    /// Set the prefix to be used in the [crate::Packet::Isi] packet during connection
    /// handshake.
    pub fn isi_prefix<C: Into<Option<char>>>(mut self, c: C) -> Self {
        self.isi_prefix = c.into();
        self
    }

    /// Set the iname to be used in the [crate::Packet::Isi] packet during connection
    /// handshake.
    pub fn isi_iname<N: Into<Option<String>>>(mut self, iname: N) -> Self {
        self.isi_iname = iname.into();
        self
    }

    /// Set the interval to be used in the [crate::Packet::Isi] packet during connection
    /// handshake.
    /// This governs the time between [crate::Packet::Mci] or [crate::Packet::Nlp]
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
            iname: self
                .isi_iname
                .as_deref()
                .unwrap_or(Isi::DEFAULT_INAME)
                .to_owned(),
            prefix: self.isi_prefix.unwrap_or(0 as char),
            interval: self.isi_interval.unwrap_or(Duration::ZERO),
            ..Default::default()
        }
    }

    /// Attempt to establish (connect and handshake) a valid Insim connection using this
    /// configuration.
    /// The `Builder` is not consumed and may be reused.
    #[cfg(feature = "blocking")]
    #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
    pub fn connect_blocking(&self) -> Result<BlockingFramed> {
        use tungstenite::stream::NoDelay;

        use crate::net::{blocking_impl::UdpStream, DEFAULT_TIMEOUT_SECS};

        match self.proto {
            Proto::Tcp => {
                let stream = tcpstream_connect_to_any(&self.remote, self.connect_timeout)?;
                stream.set_nodelay(self.tcp_nodelay)?;
                stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;

                let mut stream =
                    BlockingFramed::new(Box::new(stream), Codec::new(self.mode.clone()));
                stream.write(self.isi())?;

                Ok(stream)
            },
            Proto::Udp => {
                let local = self.udp_local_address.unwrap_or("0.0.0.0:0".parse()?);

                let stream = std::net::UdpSocket::bind(local)?;
                stream.connect(&self.remote)?;
                stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;

                let mut isi = self.isi();
                if self.udp_local_address.is_none() {
                    isi.udpport = local.port();
                }

                let mut stream = BlockingFramed::new(
                    Box::new(UdpStream::from(stream)),
                    Codec::new(self.mode.clone()),
                );
                stream.write(isi)?;

                Ok(stream)
            },
            Proto::Relay => {
                use crate::net::blocking_impl::websocket::WebsocketStream;

                let mut stream = if self.relay_websocket {
                    let request = self.relay_websocket_request().body(()).unwrap();
                    let (mut stream, _response) =
                        tungstenite::client::connect_with_config(request, None, 2).unwrap();
                    match stream.get_mut() {
                        tungstenite::stream::MaybeTlsStream::Plain(t) => {
                            t.set_nodelay(true)?;
                            t.set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                            t.set_write_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                        },
                        // handle more cases as necessary, this one only focuses on native-tls
                        _ => panic!("Error: it is not Plain stream?!"),
                    }

                    BlockingFramed::new(
                        Box::new(WebsocketStream::from(stream)),
                        Codec::new(Mode::Uncompressed),
                    )
                } else {
                    let stream =
                        tcpstream_connect_to_any(crate::LFSW_RELAY_ADDR, self.connect_timeout)?;
                    stream.set_nodelay(true)?;
                    stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                    stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                    BlockingFramed::new(Box::new(stream), Codec::new(Mode::Uncompressed))
                };

                if let Some(hostname) = &self.relay_select_host {
                    let packet = crate::relay::Sel {
                        reqi: RequestId(1),
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
                    };

                    stream.write(packet)?;
                }

                Ok(stream)
            },
        }
    }

    /// Attempt to establish (connect and handshake) a valid Insim connection using this
    /// configuration.
    /// The `Builder` is not consumed and may be reused.
    #[cfg(feature = "tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub async fn connect_async(&self) -> Result<AsyncFramed> {
        use crate::{
            net::tokio_impl::{udp::UdpStream, websocket::WebsocketStream},
            LFSW_RELAY_ADDR,
        };

        match self.proto {
            Proto::Tcp => {
                let stream = tcpstream_connect_to_any(&self.remote, self.connect_timeout)?;
                stream.set_nodelay(self.tcp_nodelay)?;
                stream.set_nonblocking(true)?;
                let stream = tokio::net::TcpStream::from_std(stream)?;

                let mut stream = AsyncFramed::new(Box::new(stream), Codec::new(self.mode.clone()));
                stream.write(self.isi()).await?;

                Ok(stream)
            },
            Proto::Udp => {
                let local = self.udp_local_address.unwrap_or("0.0.0.0:0".parse()?);

                let stream = std::net::UdpSocket::bind(local)?;
                stream.connect(&self.remote)?;
                stream.set_nonblocking(true)?;

                let stream = tokio::net::UdpSocket::from_std(stream)?;

                let mut isi = self.isi();
                if self.udp_local_address.is_none() {
                    isi.udpport = local.port();
                }

                let mut stream = AsyncFramed::new(
                    Box::new(UdpStream::from(stream)),
                    Codec::new(self.mode.clone()),
                );
                stream.write(isi).await?;

                Ok(stream)
            },
            Proto::Relay => {
                let mut stream = if self.relay_websocket {
                    let request = self.relay_websocket_request().body(()).unwrap();
                    let (stream, _response) = tokio_tungstenite::connect_async_with_config(
                        request,
                        None,
                        self.tcp_nodelay,
                    )
                    .await
                    .unwrap();

                    AsyncFramed::new(
                        Box::new(WebsocketStream::from(stream)),
                        Codec::new(Mode::Uncompressed),
                    )
                } else {
                    let stream = tcpstream_connect_to_any(LFSW_RELAY_ADDR, self.connect_timeout)?;
                    stream.set_nonblocking(true)?;
                    stream.set_nodelay(self.tcp_nodelay)?;
                    let stream = tokio::net::TcpStream::from_std(stream)?;
                    AsyncFramed::new(Box::new(stream), Codec::new(Mode::Uncompressed))
                };

                if let Some(hostname) = &self.relay_select_host {
                    let packet = crate::relay::Sel {
                        reqi: RequestId(1),
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
                    };

                    stream.write(packet).await?;
                }

                Ok(stream)
            },
        }
    }

    fn relay_websocket_request(&self) -> tungstenite::http::request::Builder {
        let uri = format!("ws://{}/connect", crate::LFSW_RELAY_ADDR)
            .parse::<tungstenite::http::Uri>()
            .expect("Failed to parse relay URI");

        tungstenite::http::Request::builder()
            .method("GET")
            .header("Host", uri.host().expect("Failed to get host from uri"))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            // It appears that isrelay.lfs.net requires an Origin header
            // Without this it does not allow us to connect.
            .header("Origin", "null")
            .uri(uri)
    }
}
