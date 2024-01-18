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
    admin_password: Option<String>,

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

            admin_password: None,

            isi_flags: IsiFlags::default(),
            isi_prefix: None,
            isi_iname: None,
            isi_interval: None,
            isi_reqi: RequestId(0),
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tcp<R: Into<SocketAddr>>(mut self, remote_addr: R) -> Self {
        self.proto = Proto::Tcp;
        self.remote = remote_addr.into();
        self
    }

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

    pub fn relay(mut self) -> Self {
        self.proto = Proto::Relay;
        self
    }

    #[cfg(feature = "websocket")]
    pub fn relay_websocket(mut self, ws: bool) -> Self {
        self.relay_websocket = ws;
        self
    }

    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = duration;
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn compressed(self) -> Self {
        self.mode(Mode::Compressed)
    }

    pub fn uncompressed(self) -> Self {
        self.mode(Mode::Uncompressed)
    }

    pub fn verify_version(mut self, verify: bool) -> Self {
        self.verify_version = verify;
        self
    }

    pub fn tcp_nodelay(mut self, no_delay: bool) -> Self {
        self.tcp_nodelay = no_delay;
        self
    }

    pub fn relay_select_host<H: Into<Option<String>>>(mut self, host: H) -> Self {
        self.relay_select_host = host.into();
        self
    }

    pub fn relay_spectator_password<P: Into<Option<String>>>(mut self, password: P) -> Self {
        self.relay_spectator_password = password.into();
        self
    }

    pub fn relay_admin_password<P: Into<Option<String>>>(mut self, password: P) -> Self {
        self.relay_admin_password = password.into();
        self
    }

    pub fn admin_password<P: Into<Option<String>>>(mut self, password: P) -> Self {
        self.admin_password = password.into();
        self
    }

    pub fn isi_reqi(mut self, i: RequestId) -> Self {
        self.isi_reqi = i;
        self
    }

    pub fn isi_flags(mut self, flags: IsiFlags) -> Self {
        self.isi_flags = flags;
        self
    }

    pub fn isi_prefix<C: Into<Option<char>>>(mut self, c: C) -> Self {
        self.isi_prefix = c.into();
        self
    }

    pub fn isi_iname<N: Into<Option<String>>>(mut self, iname: N) -> Self {
        self.isi_iname = iname.into();
        self
    }

    pub fn isi_interval<D: Into<Option<Duration>>>(mut self, duration: D) -> Self {
        self.isi_interval = duration.into();
        self
    }

    pub fn isi(&self) -> Isi {
        let udpport = match self.proto {
            Proto::Udp => self.udp_local_address.unwrap().port(),
            _ => 0,
        };

        let admin = if let Some(admin) = &self.admin_password {
            admin.clone()
        } else {
            "".into()
        };

        let iname = if let Some(iname) = &self.isi_iname {
            iname.clone()
        } else {
            "".into()
        };

        Isi {
            reqi: self.isi_reqi,
            udpport,
            flags: self.isi_flags,
            admin,
            iname,
            prefix: self.isi_prefix.unwrap_or(0 as char),
            interval: self.isi_interval.unwrap_or(Duration::ZERO),
            ..Default::default()
        }
    }

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
                        admin: match &self.relay_admin_password {
                            None => "".into(),
                            Some(i) => i.clone(),
                        },
                        spec: match &self.relay_spectator_password {
                            None => "".into(),
                            Some(i) => i.clone(),
                        },
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
                crate::net::websocket::connect_to_relay(),
            )
            .await??;

            let mut inner = FramedInner::new(stream, Codec::new(Mode::Uncompressed));
            inner.verify_version(false);
            return Ok(Framed::WebSocket(inner));
        }

        let stream = timeout(
            self.connect_timeout,
            tokio::net::TcpStream::connect("isrelay.lfs.net:47474"),
        )
        .await??;
        stream.set_nodelay(self.tcp_nodelay)?;

        let mut inner = FramedInner::new(stream, Codec::new(Mode::Uncompressed));
        inner.verify_version(false);
        Ok(Framed::Tcp(inner))
    }
}
