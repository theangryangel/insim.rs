use std::time::Duration;
use tokio::{
    io::BufWriter,
    net::{TcpStream, UdpSocket},
    time::{self, timeout},
};

use crate::{error::Error, insim::Isi, packet::Packet, result::Result};
use bytes::BytesMut;
use if_chain::if_chain;

use crate::{codec::Codec, network::Network};

use super::websocket::TungsteniteWebSocket;
use super::DEFAULT_TIMEOUT_SECS;

pub struct Framed<N>
where
    N: Network,
{
    inner: N,
    codec: Codec,
    buffer: BytesMut,
    verify_version: bool,
}

impl<N> Framed<N>
where
    N: Network,
{
    pub fn new(inner: N, codec: Codec) -> Self {
        let buffer = BytesMut::new();

        Self {
            inner,
            codec,
            buffer,
            verify_version: false,
        }
    }

    pub fn set_verify_version(&mut self, verify_version: bool) {
        self.verify_version = verify_version;
    }

    pub async fn handshake(&mut self, isi: Isi, timeout: Duration) -> Result<()> {
        time::timeout(timeout, self.write(isi.into())).await??;

        Ok(())
    }

    pub async fn read(&mut self) -> Result<Packet> {
        loop {
            if_chain! {
                if !self.buffer.is_empty();
                if let Some(packet) = self.codec.decode(&mut self.buffer)?;
                then {
                    if self.verify_version {
                        // maybe verify version
                        packet.maybe_verify_version()?;
                    }

                    // keepalive
                    if let Some(pong) = packet.maybe_pong() {
                        self.write(pong).await?;
                    }

                    return Ok(packet);
                }
            }

            match timeout(
                Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                self.inner.try_read_bytes(&mut self.buffer),
            )
            .await?
            {
                Ok(0) => {
                    // The remote closed the connection. For this to be a clean
                    // shutdown, there should be no data in the read buffer. If
                    // there is, this means that the peer closed the socket while
                    // sending a frame.
                    if !self.buffer.is_empty() {
                        tracing::debug!(
                            "Buffer was not empty when disconnected: {:?}",
                            self.buffer
                        );
                    }

                    return Err(Error::Disconnected);
                }
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    pub async fn write(&mut self, packet: Packet) -> Result<()> {
        let mut buf = BytesMut::new();

        self.codec.encode(&packet, &mut buf)?;
        if !buf.is_empty() {
            self.inner.try_write_bytes(&buf).await?;
        }

        Ok(())
    }
}

// The "Inner" connection for Connection, so that we can avoid Box'ing
// Since the ConnectionOptions is all very hard coded, for "high level" API usage,
// I think this fine.
// i.e. if we add a Websocket option down the line, then ConnectionOptions needs to understand it
// therefore we cannot just box stuff magically anyway.
pub enum FramedWrapped {
    Tcp(Framed<TcpStream>),
    BufferedTcp(Framed<BufWriter<TcpStream>>),
    Udp(Framed<UdpSocket>),
    WebSocket(Framed<TungsteniteWebSocket>),
}

impl FramedWrapped {
    pub async fn handshake(&mut self, isi: Isi, timeout: Duration) -> Result<()> {
        match self {
            Self::Tcp(i) => i.handshake(isi, timeout).await,
            Self::Udp(i) => i.handshake(isi, timeout).await,
            Self::WebSocket(i) => i.handshake(isi, timeout).await,
            Self::BufferedTcp(i) => i.handshake(isi, timeout).await,
        }
    }

    pub async fn read(&mut self) -> Result<Packet> {
        match self {
            Self::Tcp(i) => i.read().await,
            Self::Udp(i) => i.read().await,
            Self::WebSocket(i) => i.read().await,
            Self::BufferedTcp(i) => i.read().await,
        }
    }

    pub async fn write<I: Into<Packet> + Send + Sync>(&mut self, packet: I) -> Result<()> {
        match self {
            Self::Tcp(i) => i.write(packet.into()).await,
            Self::Udp(i) => i.write(packet.into()).await,
            Self::WebSocket(i) => i.write(packet.into()).await,
            Self::BufferedTcp(i) => i.write(packet.into()).await,
        }
    }
}
