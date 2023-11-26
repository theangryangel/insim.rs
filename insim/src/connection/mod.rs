mod event;
mod identifier;
mod network_options;

use std::{fmt::Debug, net::SocketAddr, time::Duration};

pub use event::Event;
pub use identifier::ConnectionIdentifier;
use tokio::{io::BufWriter, time::timeout};

use crate::{
    codec::{Codec, Mode},
    error::Error,
    insim::Isi,
    network::{Framed, FramedInner},
    packet::Packet,
    relay::HostSelect,
    result::Result,
};

use self::network_options::NetworkOptions;

#[derive(Debug)]
pub struct Connection {
    pub id: Option<ConnectionIdentifier>,

    isi: Isi,
    network_options: NetworkOptions,
    inner: Option<Framed>,
    shutdown: bool,

    shutdown_notify: tokio::sync::Notify,
}

impl Connection {
    #[tracing::instrument]
    pub fn tcp<R: Into<SocketAddr> + Debug>(
        mode: Mode,
        remote: R,
        verify_version: bool,
        options: Isi,
    ) -> Self {
        tracing::debug!("Building Tcp connection");
        Connection {
            id: None,
            inner: None,
            isi: options,
            network_options: NetworkOptions::Tcp {
                remote: remote.into(),
                verify_version,
                mode,
            },
            shutdown: false,
            shutdown_notify: tokio::sync::Notify::new(),
        }
    }

    #[tracing::instrument]
    pub fn udp<L: Into<Option<SocketAddr>> + Debug, R: Into<SocketAddr> + Debug>(
        local: L,
        remote: R,
        mode: Mode,
        verify_version: bool,
        options: Isi,
    ) -> Self {
        tracing::debug!("Building Udp connection");
        Connection {
            id: None,
            inner: None,
            isi: options,
            network_options: NetworkOptions::Udp {
                local: local.into(),
                remote: remote.into(),
                verify_version,
                mode,
            },
            shutdown: false,
            shutdown_notify: tokio::sync::Notify::new(),
        }
    }

    #[tracing::instrument]
    pub fn relay<H: Into<Option<String>> + Debug, S: Into<Option<String>> + Debug>(
        select_host: H,
        websocket: bool,
        spectator_password: S,
        admin_password: S,
        options: Isi,
    ) -> Self {
        tracing::debug!("Building Relay connection");
        Connection {
            id: None,
            inner: None,
            isi: options,
            network_options: NetworkOptions::Relay {
                select_host: select_host.into(),
                spectator_password: spectator_password.into(),
                admin_password: admin_password.into(),
                websocket,
            },
            shutdown: false,
            shutdown_notify: tokio::sync::Notify::new(),
        }
    }

    pub fn isi(&self) -> &Isi {
        &self.isi
    }

    pub fn isi_mut(&mut self) -> &mut Isi {
        &mut self.isi
    }

    #[tracing::instrument(skip(self), fields(self = ?self.id))]
    pub(crate) async fn connect(&mut self) -> Result<Framed> {
        let timeout_duration = Duration::from_secs(5);

        tracing::debug!("Connecting...");

        match &self.network_options {
            NetworkOptions::Tcp { remote, mode, .. } => {
                let stream =
                    timeout(timeout_duration, tokio::net::TcpStream::connect(remote)).await??;
                stream.set_nodelay(true)?;

                let stream = BufWriter::new(stream);

                let mut stream = FramedInner::new(stream, Codec::new(mode.clone()));
                stream
                    .handshake(self.isi.clone(), Duration::from_secs(30))
                    .await?;

                Ok(Framed::BufferedTcp(stream))
            }
            NetworkOptions::Udp {
                local,
                remote,
                mode,
                ..
            } => {
                let local = local.unwrap_or("0.0.0.0:0".parse()?);

                let stream = tokio::net::UdpSocket::bind(local).await?;
                stream.connect(remote).await.unwrap();

                let mut stream = FramedInner::new(stream, Codec::new(mode.clone()));
                stream
                    .handshake(self.isi.clone(), Duration::from_secs(30))
                    .await?;

                Ok(Framed::Udp(stream))
            }
            NetworkOptions::Relay {
                select_host,
                spectator_password,
                admin_password,
                websocket,
            } => {
                let mut stream = if *websocket {
                    let stream = timeout(
                        timeout_duration,
                        crate::network::websocket::connect_to_relay(),
                    )
                    .await??;

                    Framed::WebSocket(FramedInner::new(stream, Codec::new(Mode::Uncompressed)))
                } else {
                    tracing::debug!("Attempting connection...");

                    let stream = timeout(
                        Duration::from_secs(5),
                        tokio::net::TcpStream::connect("isrelay.lfs.net:47474"),
                    )
                    .await??;

                    tracing::debug!("Finished Connecting");

                    Framed::Tcp(FramedInner::new(stream, Codec::new(Mode::Uncompressed)))
                };

                if let Some(hostname) = select_host {
                    let packet = HostSelect {
                        hname: hostname.to_string(),
                        admin: match admin_password {
                            None => "".into(),
                            Some(i) => i.clone(),
                        },
                        spec: match spectator_password {
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

    #[tracing::instrument(skip(self), fields(self = ?self.id))]
    pub async fn send<I: Into<Packet> + Debug>(&mut self, packet: I) -> Result<()> {
        if self.shutdown {
            return Err(Error::Shutdown);
        }

        match self.inner.as_mut() {
            None => Err(Error::Disconnected),
            Some(ref mut inner) => inner.write(packet.into()).await,
        }
    }

    pub fn shutdown(&mut self) {
        self.shutdown = true;
        self.shutdown_notify.notify_one();
    }

    /// Handles establishing the connection, managing the keepalive (ping) and returns the next
    /// [Event].
    /// On error, calling again will result in a reconnection attempt.
    /// Failure to call poll will result in a timeout.
    #[tracing::instrument(skip(self), fields(self = ?self.id))]
    pub async fn poll(&mut self) -> Result<Event> {
        if self.shutdown {
            return Err(Error::Shutdown);
        }

        if self.inner.is_none() {
            self.inner = Some(self.connect().await?);
            return Ok(Event::Connected(self.id));
        }

        match self.poll_inner().await {
            Ok(inner) => Ok(inner),
            Err(inner) => {
                self.inner = None;
                Err(inner)
            }
        }
    }

    async fn poll_inner(&mut self) -> Result<Event> {
        let stream = self.inner.as_mut().unwrap();

        tokio::select! {
            _ = self.shutdown_notify.notified() => {
                Ok(Event::Shutdown(self.id))
            },

            packet = stream.read() => {
                let packet = packet?;

                Ok(Event::Data(packet, self.id))
            },
        }
    }

    pub fn is_connected(&self) -> bool {
        self.inner.is_some()
    }
}
