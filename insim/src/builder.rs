//! Tools to build a connection to LFS using Insim
use std::{
    fmt::Debug,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

#[cfg(feature = "tokio")]
use crate::Packet;
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
    net::Codec,
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
}

#[derive(Debug)]
/// Builder to help you connect to Insim
pub struct Builder {
    proto: Proto,

    connect_timeout: Duration,

    remote: Addr,

    isi_admin_password: Option<String>,
    isi_flags: IsiFlags,
    isi_prefix: Option<char>,
    isi_interval: Option<Duration>,
    isi_iname: Option<String>,
    isi_reqi: RequestId,

    // Choosing to use separate fields with a prefix, rather than an enum because when this was
    // originally implemented it supported LFSW Relay, which no longer exists, and if you were to do
    // something like this:
    //  Builder::new().relay().relay_admin_password("123").udp(None).relay()
    // the user's expectation would not be to loose all the previous relay configuration.
    // Why would they do this? Absolutely no idea. However, by separating out the fields, it
    // massively simplifies things for us.
    tcp_nodelay: bool,
    non_blocking: bool,
    udp_local_address: Option<SocketAddr>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),

            proto: Proto::Tcp,
            remote: ("127.0.0.1", 29999_u16).into(),

            tcp_nodelay: true,
            non_blocking: false,
            udp_local_address: None,

            isi_admin_password: None,

            isi_flags: IsiFlags::default(),
            isi_prefix: None,
            isi_iname: None,
            isi_interval: None,
            isi_reqi: RequestId(1),
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

    /// Set the connection timeout.
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = duration;
        self
    }

    /// Set whether sockets are non-blocking.
    /// Default is `false` if blocking.
    /// Always forced for tokio implementation.
    pub fn set_non_blocking(mut self, non_blocking: bool) -> Self {
        self.non_blocking = non_blocking;
        self
    }

    /// Set whether sockets have `TCP_NODELAY` enabled.
    /// Default is `true`
    pub fn tcp_nodelay(mut self, no_delay: bool) -> Self {
        self.tcp_nodelay = no_delay;
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
    /// If not provided 1 is used by default. Any non-zero value will result in LFS responding with
    /// a Ver packet, which will be verified by the library.
    /// Set to 0 if you wish to circumvent this.
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
    pub fn isi(&self, udpport: Option<u16>) -> Isi {
        Isi {
            reqi: self.isi_reqi,
            udpport: udpport.unwrap_or(0),
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
        use crate::net::{DEFAULT_TIMEOUT_SECS, blocking_impl::UdpStream};

        match self.proto {
            Proto::Tcp => {
                let stream = tcpstream_connect_to_any(&self.remote, self.connect_timeout)?;
                stream.set_nodelay(self.tcp_nodelay)?;
                stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                if self.non_blocking {
                    stream.set_nonblocking(true)?;
                }
                let mut stream = BlockingFramed::new(Box::new(stream), Codec::new());
                stream.write(self.isi(None))?;

                Ok(stream)
            },
            Proto::Udp => {
                let local = self.udp_local_address.unwrap_or("0.0.0.0:0".parse()?);

                let stream = std::net::UdpSocket::bind(local)?;
                stream.connect(&self.remote)?;
                stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))?;
                if self.non_blocking {
                    stream.set_nonblocking(true)?;
                }

                let isi = self.isi(Some(local.port()));

                let mut stream =
                    BlockingFramed::new(Box::new(UdpStream::from(stream)), Codec::new());
                stream.write(isi)?;

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
        use crate::net::tokio_impl::udp::UdpStream;

        match self.proto {
            Proto::Tcp => {
                let stream = tcpstream_connect_to_any(&self.remote, self.connect_timeout)?;
                stream.set_nodelay(self.tcp_nodelay)?;
                stream.set_nonblocking(true)?;
                let stream = tokio::net::TcpStream::from_std(stream)?;

                let mut stream = AsyncFramed::new(Box::new(stream), Codec::new());
                stream.write(self.isi(None)).await?;

                Ok(stream)
            },
            Proto::Udp => {
                let local = self.udp_local_address.unwrap_or("0.0.0.0:0".parse()?);

                let stream = std::net::UdpSocket::bind(local)?;
                stream.connect(&self.remote)?;
                stream.set_nonblocking(true)?;

                let stream = tokio::net::UdpSocket::from_std(stream)?;

                let isi = self.isi(Some(local.port()));

                let mut stream = AsyncFramed::new(Box::new(UdpStream::from(stream)), Codec::new());
                stream.write(isi).await?;

                Ok(stream)
            },
        }
    }

    /// Connect and spawn a background Tokio task to manage the insim connection.
    /// A [SpawnedHandle] is returned to allow you to interact with the background task to send and
    /// receive packets.
    /// Automatic reconnection is not handled at this time.
    #[cfg(feature = "tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub async fn spawn<C: Into<Option<usize>>>(
        self,
        capacity: C,
    ) -> Result<(SpawnedHandle, tokio::task::JoinHandle<crate::Result<()>>)> {
        let mut net = self.connect_async().await?;

        let cap = capacity.into().unwrap_or(100);

        let (event_sender, _) = tokio::sync::broadcast::channel::<crate::Packet>(cap);
        let (command_sender, mut command_receiver) =
            tokio::sync::mpsc::channel::<crate::Packet>(cap);

        let token = tokio_util::sync::CancellationToken::new();

        let cloned_event_sender = event_sender.clone();
        let cloned_command_sender = command_sender.clone();
        let cloned_token = token.clone();

        let join_handle: tokio::task::JoinHandle<crate::Result<()>> = tokio::spawn(async move {
            // main event loop
            loop {
                tokio::select! {
                    // shutdown / abort / cancellation
                    _ = token.cancelled() => {
                        break;
                    },

                    // packet from LFS
                    packet = net.read() => {
                        let packet = packet?;

                        if event_sender.receiver_count() > 0 {
                            let _ = event_sender.send(packet);
                        }
                    },

                    // commands
                    Some(packet) = command_receiver.recv() => {
                        net.write(packet).await?;
                    }
                }
            }

            Ok(())
        });

        let handle = SpawnedHandle {
            events: cloned_event_sender,
            commands: cloned_command_sender,
            cancellation_token: cloned_token,
        };

        Ok((handle, join_handle))
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[derive(Debug, Clone)]
/// Handle for a spawned insim connection
pub struct SpawnedHandle {
    /// Receiver for packets
    events: tokio::sync::broadcast::Sender<Packet>,
    // Sender for packets
    commands: tokio::sync::mpsc::Sender<Packet>,
    // Cancellation token for shutdown handling
    cancellation_token: tokio_util::sync::CancellationToken,
}

#[cfg(feature = "tokio")]
impl SpawnedHandle {
    /// Subscribe to a stream of Packets
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<crate::Packet> {
        self.events.subscribe()
    }

    /// Send an insim packet
    pub async fn send<P: Into<crate::Packet> + Send + Sync>(&self, packet: P) -> crate::Result<()> {
        self.commands
            .send(packet.into())
            .await
            .map_err(|_| crate::Error::SpawnedDead)
    }

    /// Sends an iterator of packets concurrently and stops on the first error
    pub async fn send_all<I, P>(&self, packets: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = P> + Send,
        P: Into<crate::Packet> + Send + Debug,
    {
        let futures = packets.into_iter().map(|p| self.send(p.into()));
        let _ = futures::future::try_join_all(futures).await?;
        Ok(())
    }

    /// Shortcut to send a command
    pub async fn send_command<S: Into<String>>(&self, command: S) -> crate::Result<()> {
        let command: String = command.into();
        self.send(crate::insim::Mst {
            msg: command,
            ..Default::default()
        })
        .await
        .map_err(|_| crate::Error::SpawnedDead)
    }

    /// Shortcut to send a message. This will automatically detect what type of packet to send for
    /// you.
    pub async fn send_message<
        U: Into<Option<crate::identifiers::ConnectionId>>,
        S: Into<String>,
    >(
        &self,
        msg: S,
        ucid: U,
    ) -> crate::Result<()> {
        let msg: String = msg.into();

        let packet: crate::Packet = if let Some(ucid) = ucid.into() {
            crate::insim::Mtc {
                ucid,
                text: msg,
                ..Default::default()
            }
            .into()
        } else if msg.len() > 63 {
            crate::insim::Mst {
                msg,
                ..Default::default()
            }
            .into()
        } else {
            crate::insim::Msx {
                msg,
                ..Default::default()
            }
            .into()
        };

        self.send(packet)
            .await
            .map_err(|_| crate::Error::SpawnedDead)
    }

    /// Request cancellation / shutdown
    pub async fn shutdown(&self) {
        self.cancellation_token.cancelled().await
    }
}
