mod event;
mod identifier;
mod network_options;

use std::{net::SocketAddr, time::Duration};

pub use event::Event;
pub use identifier::ConnectionIdentifier;
use tokio::{io::BufWriter, time::timeout};

use crate::{
    codec::{Codec, Frame, Mode},
    error::Error,
    network::{Framed, FramedWrapped},
    relay::HostSelect,
    result::Result,
};

use self::network_options::NetworkOptions;

pub struct Connection<P: Frame + std::convert::From<HostSelect>> {
    pub id: Option<ConnectionIdentifier>,

    isi: P::Isi,
    network_options: NetworkOptions,
    inner: Option<FramedWrapped<P>>,
    shutdown: bool,

    shutdown_notify: tokio::sync::Notify,
}

impl<P: Frame + std::convert::From<HostSelect>> Connection<P> {
    pub fn tcp<R: Into<SocketAddr>>(
        mode: Mode,
        remote: R,
        verify_version: bool,
        options: P::Isi,
    ) -> Self {
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

    pub fn udp<L: Into<Option<SocketAddr>>, R: Into<SocketAddr>>(
        local: L,
        remote: R,
        mode: Mode,
        verify_version: bool,
        options: P::Isi,
    ) -> Self {
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

    pub fn relay<H: Into<Option<String>>, S: Into<Option<String>>>(
        select_host: H,
        websocket: bool,
        spectator_password: S,
        admin_password: S,
        options: P::Isi,
    ) -> Self {
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

    pub fn isi(&self) -> &P::Isi {
        &self.isi
    }

    pub fn isi_mut(&mut self) -> &mut P::Isi {
        &mut self.isi
    }

    pub(crate) async fn connect(&mut self) -> Result<FramedWrapped<P>> {
        let timeout_duration = Duration::from_secs(5);

        tracing::debug!("Connecting...");

        match &self.network_options {
            NetworkOptions::Tcp { remote, mode, .. } => {
                let stream =
                    timeout(timeout_duration, tokio::net::TcpStream::connect(remote)).await??;
                stream.set_nodelay(true)?;

                let stream = BufWriter::new(stream);

                let mut stream = Framed::new(stream, Codec::new(*mode));
                stream
                    .handshake(self.isi.clone(), Duration::from_secs(30))
                    .await?;

                Ok(FramedWrapped::BufferedTcp(stream))
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

                let mut stream = Framed::new(stream, Codec::new(*mode));
                stream
                    .handshake(self.isi.clone(), Duration::from_secs(30))
                    .await?;

                Ok(FramedWrapped::Udp(stream))
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

                    FramedWrapped::WebSocket(Framed::new(stream, Codec::new(Mode::Uncompressed)))
                } else {
                    tracing::debug!("Attempting connection...");

                    let stream = timeout(
                        Duration::from_secs(5),
                        tokio::net::TcpStream::connect("isrelay.lfs.net:47474"),
                    )
                    .await??;

                    tracing::debug!("Finished Connecting");

                    FramedWrapped::Tcp(Framed::new(stream, Codec::new(Mode::Uncompressed)))
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

    pub async fn send<I: Into<P>>(&mut self, packet: I) -> Result<()> {
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
    pub async fn poll(&mut self) -> Result<Event<P>> {
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

    async fn poll_inner(&mut self) -> Result<Event<P>> {
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
