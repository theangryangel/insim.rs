use std::{net::SocketAddr, time::Duration};

use insim_core::identifiers::RequestId;
use tokio::{
    net::{TcpStream, UdpSocket},
    time::timeout,
};

use crate::{
    codec::Mode, connection::inner::ConnectionInner, packets::insim::Isi, result::Result,
    traits::WritePacket,
};

#[derive(Clone)]
pub enum NetworkOptions {
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
    },
}

impl Default for NetworkOptions {
    fn default() -> Self {
        Self::Tcp {
            remote: "127.0.0.1:29999".parse().unwrap(),
            codec_mode: Mode::Compressed,
            verify_version: true,
            wait_for_initial_pong: true,
        }
    }
}

impl NetworkOptions {
    pub(crate) async fn connect(
        &self,
        isi: Isi,
        timeout_duration: Duration,
    ) -> Result<ConnectionInner> {
        match self {
            NetworkOptions::Tcp {
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

                Ok(ConnectionInner::Udp(stream))
            }
            NetworkOptions::Relay { select_host } => {
                use crate::packets::relay::HostSelect;

                let stream = timeout(
                    timeout_duration,
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

                Ok(ConnectionInner::Tcp(stream))
            }
        }
    }
}
