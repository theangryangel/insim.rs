mod event;
mod identifier;
mod inner;
mod network_options;
mod options;

use std::{net::{SocketAddr}, time::Duration, marker::PhantomData};

pub use event::Event;
pub use identifier::ConnectionIdentifier;
pub use options::ConnectionOptions;
use tokio::{time::timeout, net::{TcpStream, UdpSocket}};

use crate::{
    connection::inner::ConnectionInner,
    error::Error,
    result::Result, codec::{Codec, Mode}, network::{Network, Framed}, relay::HostSelect,
};

use self::network_options::NetworkOptions;

pub struct Connection<C: Codec> {
    id: Option<ConnectionIdentifier>,

    network_options: NetworkOptions,
    inner: Option<ConnectionInner<C>>,
    shutdown: bool,

    shutdown_notify: tokio::sync::Notify,
}

impl<C: Codec> Connection<C> {
    pub fn tcp<
        R: Into<SocketAddr>
    >(
        mode: Mode,
        remote: R,
        verify_version: bool,
    ) -> Self {
        Connection {
            id: None,
            inner: None,
            network_options: NetworkOptions::Tcp {
                remote: remote.into(),
                verify_version,
                mode,
            },
            shutdown: false,
            shutdown_notify: tokio::sync::Notify::new(),
        }
    }

    pub fn udp<
        L: Into<Option<SocketAddr>>,
        R: Into<SocketAddr>
    >(
        local: L,
        remote: R,
        mode: Mode,
        verify_version: bool,
    ) -> Self {
        Connection {
            id: None,
            inner: None,
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

    pub fn relay<
        H: Into<Option<String>>, 
        P: Into<Option<String>>
    > (
        select_host: H,
        websocket: bool,
        spectator_password: P,
    ) -> Self {
        Connection {
            id: None,
            inner: None,
            network_options: NetworkOptions::Relay { 
                select_host: select_host.into(), 
                spectator_password: spectator_password.into(), 
                websocket
            },
            shutdown: false,
            shutdown_notify: tokio::sync::Notify::new(),
        }
    }


    pub(crate) async fn connect(
        &mut self,
    ) -> Result<ConnectionInner<C>> {
        let timeout_duration = Duration::from_secs(90);

        match &self.network_options {
            NetworkOptions::Tcp {
                remote, mode, ..
            } => {
                let stream = timeout(timeout_duration, tokio::net::TcpStream::connect(remote)).await??;
                stream.set_nodelay(true)?;
                let codec = C::new(*mode);
                return Ok(
                    ConnectionInner::Tcp(Framed::new(stream, codec))
                );
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
                return Ok(ConnectionInner::Udp(
                    Framed::new(stream, C::new(*mode))
                ));
            }
            NetworkOptions::Relay {
                select_host,
                spectator_password,
                websocket: false,
            } => {
                let stream = timeout(
                    timeout_duration,
                    tokio::net::TcpStream::connect("isrelay.lfs.net:47474"),
                )
                .await??;
                stream.set_nodelay(true)?;

                return Ok(
                    ConnectionInner::Tcp(
                        Framed::new(stream, C::new(Mode::Uncompressed))
                    )
                );

                // if let Some(hostname) = select_host {
                //     stream
                //         .write(
                //             HostSelect {
                //                 hname: hostname.to_string(),
                //                 admin: self.password.clone(),
                //                 spec: match spectator_password {
                //                     None => "".into(),
                //                     Some(i) => i.clone(),
                //                 },
                //                 ..Default::default()
                //             }
                //             .into(),
                //         )
                //         .await?;
                // }
                //
                // Ok(ConnectionInner::Tcp(stream))
            }

            NetworkOptions::Relay {
                select_host,
                spectator_password,
                websocket: true,
            } => {

                let stream = crate::network::websocket::connect_to_relay().await?;

                let stream = ConnectionInner::WebSocket(
                    Framed::new(stream, C::new(Mode::Uncompressed))
                );

                if let Some(hostname) = select_host {

                    let packet: C::Item = HostSelect {
                        hname: hostname.to_string(),
                        admin: "".into(),
                        spec: match spectator_password {
                            None => "".into(),
                            Some(i) => i.clone(),
                        },
                        ..Default::default()
                    }.into();

                    stream.write(packet).await?;
                }

                Ok(stream)
            }
        }
    }

    pub async fn send<P: Into<C::Item>>(&mut self, packet: P) -> Result<()> {
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
    pub async fn poll(&mut self) -> Result<Event<C::Item>> {
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

    async fn poll_inner(&mut self) -> Result<Event<C::Item>> {
        let stream = self.inner.as_mut().unwrap();

        tokio::select! {
            _ = self.shutdown_notify.notified() => {
                Ok(Event::Shutdown(self.id))
            },

            packet = stream.read() => {
                Ok(Event::Data(packet?, self.id))
            },
        }
    }
}
