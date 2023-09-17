use std::marker::PhantomData;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time;

use bytes::BytesMut;
use if_chain::if_chain;
use crate::codec::Init;
use crate::{error::Error, result::Result, codec::Packets};

use crate::{
    network::Network, codec::Codec
};

use super::websocket::TungsteniteWebSocket;

pub struct Framed<N, P>
where
    N: Network,
    P: Packets
{
    inner: N,
    codec: Codec<P>,
    buffer: BytesMut,

    verify_version: bool,

    marker: PhantomData<P>,
}

impl<N, P> Framed<N, P>
where
    N: Network,
    P: Packets
{
    pub fn new(inner: N, codec: Codec<P>) -> Self {
        let buffer = BytesMut::new();

        Self {
            inner,
            codec,
            buffer,
            verify_version: false,
            marker: PhantomData,
        }
    }

    pub fn set_verify_version(&mut self, verify_version: bool) {
        self.verify_version = verify_version;
    }

    // async fn handshake<I: Into<P> + Init + Send>(
    //     &mut self, 
    //     isi: I,
    //     timeout: Duration,
    // ) -> Result<()> {
    //     time::timeout(
    //         timeout, 
    //         self.write(isi)
    //     ).await?
    // }

    pub async fn read(&mut self) -> Result<P> {
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
                    if packet.is_ping() {
                        tracing::debug!("ping? pong!");
                        self.write(P::pong(None)).await?;
                    }

                    return Ok(packet);
                }
            }

            match self.inner.try_read_bytes(&mut self.buffer).await {
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
                    return Err(e.into());
                }
            }

        }
    }

    pub async fn write(&mut self, packet: P) -> Result<()> {
        let mut buf = BytesMut::new();

        self.codec.encode(&packet.into(), &mut buf)?;
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
pub enum FramedWrapped<P: Packets> {
    Tcp(Framed<TcpStream, P>),
    Udp(Framed<UdpSocket, P>),
    WebSocket(Framed<TungsteniteWebSocket, P>),
}

impl<P: Packets> FramedWrapped<P> {
    pub async fn read(&mut self) -> Result<P> {
        match self {
            Self::Tcp(i) => i.read().await,
            Self::Udp(i) => i.read().await,
            Self::WebSocket(i) => i.read().await,
        }
    }

    pub async fn write<I: Into<P> + Send + Sync>(&mut self, packet: I) -> Result<()> {
        match self {
            Self::Tcp(i) => i.write(packet.into()).await,
            Self::Udp(i) => i.write(packet.into()).await,
            Self::WebSocket(i) => i.write(packet.into()).await,
        }
    }
}

