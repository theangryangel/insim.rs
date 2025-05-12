use std::{fmt::Debug, time::Duration};

use bytes::{BufMut, BytesMut};
use if_chain::if_chain;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::{self, timeout},
};

/// Read Write super trait
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Debug + Unpin + Send + Sync {}

impl<T: AsyncRead + AsyncWrite + Debug + Unpin + Send + Sync> AsyncReadWrite for T {}

use crate::{
    error::Error,
    insim::Isi,
    net::{Codec, DEFAULT_TIMEOUT_SECS},
    packet::Packet,
    result::Result,
    DEFAULT_BUFFER_CAPACITY,
};

/// A unified wrapper around anything that implements [AsyncReadWrite].
/// You probably really want to look at [Framed].
#[derive(Debug)]
pub struct Framed {
    inner: Box<dyn AsyncReadWrite>,
    codec: Codec,
    buffer: BytesMut,
    verify_version: bool,
}

impl Framed {
    /// Create a new FramedInner, which wraps some kind of network transport.
    pub fn new(inner: Box<dyn AsyncReadWrite>, codec: Codec) -> Self {
        let buffer = BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY);

        Self {
            inner,
            codec,
            buffer,
            verify_version: false,
        }
    }

    /// Modifies whether or not to verify the Insim version
    pub fn verify_version(&mut self, verify_version: bool) {
        self.verify_version = verify_version;
    }

    /// Performs the Insim handshake by sending a [Isi] packet.
    /// If the handshake does not complete within the given timeout, it will fail and the
    /// connection should be considered invalid.
    pub async fn handshake(&mut self, isi: Isi, timeout: Duration) -> Result<()> {
        time::timeout(timeout, self.write(isi)).await??;

        Ok(())
    }

    #[allow(unsafe_code)]
    async fn read_buf(&mut self) -> Result<usize> {
        // We're making the assumption that the internal stream will not discard data if there is
        // not enough space in the chunk.
        // This is how TcpStream works, and how we've made UdpStream work.
        let size = {
            let chunk = self.buffer.chunk_mut();
            let len = chunk.len();

            let ptr = chunk.as_mut_ptr();
            // SAFETY: The inner stream is not going to read from the uninitialized data.
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
            self.inner.read(slice).await?
        };

        // SAFETY: The inner stream just initialized that many bytes, we need to trust that
        unsafe {
            self.buffer.advance_mut(size);
        }

        Ok(size)
    }

    /// Asynchronously wait for a packet from the inner network.
    pub async fn read(&mut self) -> Result<Packet> {
        loop {
            if_chain! {
                if !self.buffer.is_empty();
                if let Some(packet) = self.codec.decode(&mut self.buffer)?;
                then {
                    if self.verify_version {
                        // maybe verify version
                        let _ = packet.maybe_verify_version()?;
                    }

                    // keepalive
                    if let Some(pong) = packet.maybe_pong() {
                        tracing::debug!("Ping? Pong!");
                        self.write(pong).await?;
                    }

                    return Ok(packet);
                }
            }

            match timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), self.read_buf()).await? {
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
                },
                Ok(_) => {
                    continue;
                },
                Err(e) => {
                    return Err(e);
                },
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
}
