use std::{net::SocketAddr, time::Duration};

use insim_core::identifiers::RequestId;
use tokio::{
    net::{TcpStream, UdpSocket},
    time::timeout,
};

use crate::{
    codec::Mode,
    connection::inner::ConnectionInner,
    packets::insim::{Isi, IsiFlags},
    result::Result,
    traits::WritePacket,
};

use super::network_options::NetworkOptions;

#[derive(Clone, Default)]
pub struct ConnectionOptions {
    pub name: String,
    pub password: String,
    pub flags: IsiFlags,
    pub prefix: Option<char>,
    pub interval: Option<Duration>,

    pub network_options: NetworkOptions,
    pub connection_timeout: Option<Duration>,
}

impl ConnectionOptions {
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
        self.interval = Some(interval);
        self
    }

    // Clear any set interval
    pub fn clear_interval(mut self) -> Self {
        self.interval = None;
        self
    }

    /// Create an [Isi](crate::packets::insim::Isi) packet.
    pub fn as_isi(&self) -> Isi {
        Isi {
            iname: self.name.to_owned(),
            admin: self.password.to_owned(),
            prefix: self.prefix.unwrap_or(0 as char),
            version: crate::packets::VERSION,
            interval: self.interval.unwrap_or(Duration::from_secs(0)),
            flags: self.flags,
            ..Default::default()
        }
    }

    pub fn relay<H: Into<Option<String>>, P: Into<Option<String>>>(
        mut self,
        select_host: H,
        websocket: bool,
        spectator_password: P,
    ) -> Self {
        self.network_options = NetworkOptions::Relay {
            select_host: select_host.into(),
            spectator_password: spectator_password.into(),
            websocket,
        };
        self
    }

    pub fn tcp<R: Into<SocketAddr>>(
        mut self,
        remote: R,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    ) -> Self {
        self.network_options = NetworkOptions::Tcp {
            remote: remote.into(),
            codec_mode,
            verify_version,
            wait_for_initial_pong,
        };
        self
    }

    pub fn udp<L: Into<Option<SocketAddr>>, R: Into<SocketAddr>>(
        mut self,
        local: L,
        remote: R,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    ) -> Self {
        self.network_options = NetworkOptions::Udp {
            local: local.into(),
            remote: remote.into(),
            codec_mode,
            verify_version,
            wait_for_initial_pong,
        };
        self
    }

    pub(crate) async fn connect(&self) -> Result<ConnectionInner> {
        let timeout_duration = self.connection_timeout.unwrap_or(Duration::from_secs(90));

        match &self.network_options {
            NetworkOptions::Tcp {
                remote,
                codec_mode,
                verify_version,
                wait_for_initial_pong,
            } => {
                let stream = timeout(timeout_duration, TcpStream::connect(remote)).await??;
                stream.set_nodelay(true)?;

                let mut stream = crate::tcp::Tcp::new(stream, *codec_mode);
                let mut isi = self.as_isi();
                if *verify_version {
                    isi.reqi = RequestId(1);
                }
                super::handshake(
                    &mut stream,
                    timeout_duration,
                    isi,
                    *wait_for_initial_pong,
                    *verify_version,
                )
                .await?;

                Ok(ConnectionInner::Tcp(stream))
            }
            NetworkOptions::Udp {
                local,
                remote,
                codec_mode,
                verify_version,
                wait_for_initial_pong,
            } => {
                let local = local.unwrap_or("0.0.0.0:0".parse()?);

                let stream = UdpSocket::bind(local).await?;
                stream.connect(remote).await.unwrap();
                let mut isi = self.as_isi();
                if *verify_version {
                    isi.reqi = RequestId(1);
                }
                isi.udpport = stream.local_addr().unwrap().port();
                let mut stream = crate::udp::Udp::new(stream, *codec_mode);

                super::handshake(
                    &mut stream,
                    timeout_duration,
                    isi,
                    *wait_for_initial_pong,
                    *verify_version,
                )
                .await?;

                Ok(ConnectionInner::Udp(stream))
            }
            NetworkOptions::Relay {
                select_host,
                spectator_password,
                websocket: false,
            } => {
                use crate::packets::relay::HostSelect;

                let stream = timeout(
                    timeout_duration,
                    TcpStream::connect("isrelay.lfs.net:47474"),
                )
                .await??;
                stream.set_nodelay(true)?;

                let mut stream = crate::tcp::Tcp::new(
                    stream,
                    // TODO: Talk to LFS devs, find out if/when relay gets compressed support?
                    Mode::Uncompressed,
                );

                if let Some(hostname) = select_host {
                    stream
                        .write(
                            HostSelect {
                                hname: hostname.to_string(),
                                admin: self.password.clone(),
                                spec: match spectator_password {
                                    None => "".into(),
                                    Some(i) => i.clone(),
                                },
                                ..Default::default()
                            }
                            .into(),
                        )
                        .await?;
                }

                Ok(ConnectionInner::Tcp(stream))
            }

            NetworkOptions::Relay {
                select_host,
                spectator_password,
                websocket: true,
            } => {
                use crate::packets::relay::HostSelect;

                use tokio_tungstenite::{
                    connect_async, tungstenite::handshake::client::generate_key, tungstenite::http,
                };

                let uri = "ws://isrelay.lfs.net:47474/connect"
                    .parse::<http::Uri>()
                    .expect("Failed to parse relay URI");

                let req = http::Request::builder()
                    .method("GET")
                    .header("Host", uri.host().expect("Failed to get host from uri"))
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "websocket")
                    .header("Sec-WebSocket-Version", "13")
                    .header("Sec-WebSocket-Key", generate_key())
                    // It appears that isrelay.lfs.net requires an Origin header
                    // Without this it does not allow us to connect.
                    .header("Origin", "null")
                    .uri(uri)
                    .body(())
                    .unwrap();

                let (stream, _response) = connect_async(req).await?;

                let mut stream = crate::websocket::WebSocket::new(
                    stream,
                    // TODO: Talk to LFS devs, find out if/when relay gets compressed support?
                    Mode::Uncompressed,
                );

                if let Some(hostname) = select_host {
                    stream
                        .write(
                            HostSelect {
                                hname: hostname.to_string(),
                                admin: self.password.clone(),
                                spec: match spectator_password {
                                    None => "".into(),
                                    Some(i) => i.clone(),
                                },
                                ..Default::default()
                            }
                            .into(),
                        )
                        .await?;
                }

                Ok(ConnectionInner::WebSocket(stream))
            }
        }
    }
}
