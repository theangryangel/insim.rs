use std::{net::SocketAddr, time::Duration};

use insim_core::identifiers::RequestId;
use tokio::{
    net::{TcpStream, UdpSocket},
    time::timeout,
};

use crate::{
    codec::Mode,
    packets::insim::Isi,
    result::Result,
    traits::{ReadWritePacket, WritePacket},
};

#[derive(Clone)]
pub enum ConnectionType {
    Tcp {
        remote: SocketAddr,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    },
    Udp {
        local: Option<SocketAddr>,
        remote: SocketAddr,
        codec_mode: Mode,
        verify_version: bool,
        wait_for_initial_pong: bool,
    },
    Relay {
        select_host: Option<String>,
        connect_timeout: Duration,
    },
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Tcp {
            remote: "127.0.0.1:29999".parse().unwrap(),
            codec_mode: Mode::Compressed,
            verify_version: true,
            wait_for_initial_pong: true,
        }
    }
}

impl ConnectionType {
    pub async fn connect(
        &self,
        isi: Isi,
        timeout_duration: Duration,
    ) -> Result<Box<dyn ReadWritePacket>> {
        match self {
            ConnectionType::Tcp {
                remote,
                codec_mode,
                verify_version,
                wait_for_initial_pong,
            } => {
                let stream = timeout(timeout_duration, TcpStream::connect(remote)).await??;

                let mut stream = crate::tcp::Tcp::new(stream, *codec_mode);
                let mut isi = isi.clone();
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

                Ok(stream.boxed())
            }
            ConnectionType::Udp {
                local,
                remote,
                codec_mode,
                verify_version,
                wait_for_initial_pong,
            } => {
                let local = local.unwrap_or("0.0.0.0:0".parse()?);

                let stream = UdpSocket::bind(local).await?;
                stream.connect(remote).await.unwrap();
                let mut isi = isi.clone();
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

                Ok(stream.boxed())
            }
            ConnectionType::Relay {
                select_host,
                connect_timeout,
            } => {
                use crate::packets::relay::HostSelect;

                // Why not call connect_tcp? purely so we won't call request_game_state multiple times
                // until we select a host there is no game state to request
                let stream = timeout(
                    *connect_timeout,
                    TcpStream::connect("isrelay.lfs.net:47474"),
                )
                .await??;

                let mut stream = crate::tcp::Tcp::new(
                    stream,
                    // TODO: Talk to LFS devs, find out if/when relay gets compressed support?
                    Mode::Uncompressed,
                );

                super::handshake(
                    &mut stream,
                    timeout_duration,
                    isi,
                    false, // Relay does not respond to ping requests
                    false, // Relay does not respond to version requests until after the host is selected
                )
                .await?;

                if let Some(hostname) = select_host {
                    stream
                        .write(
                            HostSelect {
                                hname: hostname.to_string(),
                                ..Default::default()
                            }
                            .into(),
                        )
                        .await?;
                }

                Ok(stream.boxed())
            }
        }
    }
}
