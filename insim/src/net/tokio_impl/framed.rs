use std::{fmt::Debug, time::Duration};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::timeout,
};

/// Read Write super trait
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Debug + Unpin + Send + Sync {}

impl<T: AsyncRead + AsyncWrite + Debug + Unpin + Send + Sync> AsyncReadWrite for T {}

use crate::{
    error::Error,
    insim::TinyType,
    net::{Codec, DEFAULT_TIMEOUT_SECS},
    packet::Packet,
    result::Result,
};

/// A unified wrapper around anything that implements [AsyncReadWrite].
/// You probably really want to look at [Framed].
#[derive(Debug)]
pub struct Framed {
    inner: Box<dyn AsyncReadWrite>,
    codec: Codec,
}

impl Framed {
    /// Create a new FramedInner, which wraps some kind of network transport.
    pub fn new(inner: Box<dyn AsyncReadWrite>, codec: Codec) -> Self {
        Self { inner, codec }
    }

    /// Asynchronously wait for a packet from the inner network.
    pub async fn read(&mut self) -> Result<Packet> {
        loop {
            if self.codec.reached_timeout() {
                return Err(Error::Timeout(
                    "Timeout exceeded, no keepalive or packet received".into(),
                ));
            }

            // ensure that we exhaust the buffer first
            let packet = self.codec.decode()?;

            if let Some(keepalive) = self.codec.keepalive() {
                tracing::debug!("Ping? Pong!");
                self.write(keepalive).await?;
            }

            if let Some(packet) = packet {
                return Ok(packet);
            }

            let amt = timeout(
                Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                self.inner.read_buf(self.codec.buf_mut()),
            )
            .await??;
            if amt == 0 {
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                return Err(Error::Disconnected);
            }
        }
    }

    /// Asynchronously write a packet to the inner network.
    pub async fn write<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        let mut buf = self.codec.encode(&packet.into())?;
        if !buf.is_empty() {
            self.inner.write_all_buf(&mut buf).await?;
        }

        Ok(())
    }

    /// Asynchronously flush the inner network
    pub async fn flush(&mut self) -> Result<()> {
        self.inner.flush().await?;
        Ok(())
    }

    /// Asynchronously flush the inner network and shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        self.write(TinyType::Close).await?;
        self.flush().await?;
        self.inner.shutdown().await?;
        while self.read().await.is_ok() {}
        Ok(())
    }
}
