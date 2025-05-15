use std::{
    fmt::Debug,
    io::{Read, Write},
};

use bytes::{BufMut, BytesMut};

use super::ReadWrite;
use crate::{insim::Isi, net::Codec, result::Result, Error, Packet, DEFAULT_BUFFER_CAPACITY};

/// A unified wrapper around anything that implements Read + Write.
#[derive(Debug)]
pub struct Framed {
    inner: Box<dyn ReadWrite>,
    codec: Codec,
    buffer: BytesMut,
    verify_version: bool,
}

impl Framed {
    /// Create a new FramedInner, which wraps some kind of network transport.
    pub fn new(inner: Box<dyn ReadWrite>, codec: Codec) -> Self {
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
    pub fn handshake(&mut self, isi: Isi) -> Result<()> {
        self.write(Into::<Packet>::into(isi))?;

        Ok(())
    }

    #[allow(unsafe_code)]
    fn read_buf(&mut self) -> Result<usize> {
        // We're making the assumption that the internal stream will not discard data if there is
        // not enough space in the chunk.
        // This is how TcpStream works, and how we've made UdpStream work.
        let size = {
            let chunk = self.buffer.chunk_mut();
            let len = chunk.len();

            let ptr = chunk.as_mut_ptr();
            // SAFETY: The inner stream is not going to read from the uninitialized data.
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
            self.inner.read(slice)?
        };

        // SAFETY: The inner stream just initialized that many bytes, we need to trust that
        unsafe {
            self.buffer.advance_mut(size);
        }

        Ok(size)
    }

    /// Wait for a packet from the inner network.
    pub fn read(&mut self) -> Result<Packet> {
        loop {
            if self.codec.reached_timeout() {
                return Err(Error::Timeout(
                    "Timeout exceeded, no keepalive or packet received".into(),
                ));
            }

            let packet = if !self.buffer.is_empty() {
                self.codec.decode(&mut self.buffer)?
            } else {
                None
            };

            if let Some(keepalive) = self.codec.keepalive() {
                tracing::debug!("Ping? Pong!");
                self.write(keepalive)?;
            }

            if let Some(packet) = packet {
                return Ok(packet);
            }

            match self.read_buf() {
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
                    tracing::info!("did get err={:?}", e);
                    return Err(e);
                },
            }
        }
    }

    /// Write a packet to the inner network.
    pub fn write<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        let buf = self.codec.encode(&packet.into())?;
        if !buf.is_empty() {
            let _ = self.inner.write(&buf)?;
        }

        Ok(())
    }
}
