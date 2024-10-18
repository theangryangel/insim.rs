use std::{
    fmt::Debug,
    io::{Read, Write},
    net::UdpSocket,
};

use bytes::BytesMut;

use super::Codec;
use crate::{insim::Isi, result::Result, Error, Packet, DEFAULT_BUFFER_CAPACITY};

/// Read Write super trait
pub trait ReadWrite: Read + Write + Debug {}

impl<T: Read + Write + Debug> ReadWrite for T {}

impl From<UdpSocket> for UdpStream {
    fn from(value: UdpSocket) -> Self {
        Self { inner: value }
    }
}

/// adsasd
#[derive(Debug)]
pub struct UdpStream {
    inner: UdpSocket,
}

impl Read for UdpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // FIXME: Add an internal buffer

        // XXX: This is "safe" because we ensure that the buffer has a minimum size anywhere we
        // need where call this
        // We ensure that the buffer can accept the max size packet and as such nothing can be
        // discarded.
        self.inner.recv(buf)
    }
}

impl Write for UdpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.send(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

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

    fn try_read_bytes(&mut self) -> Result<usize> {
        // FIXME: Get rid of this when we fix the internal buffer for UDPStream

        // we allocate a temporary buffer of MAX_SIZE_PACKET to ensure that we don't run into the
        // issue where UDPSocket may truncate the data.
        let mut rx_bytes = [0u8; crate::MAX_SIZE_PACKET];
        let size = self.inner.read(&mut rx_bytes)?;
        self.buffer.extend_from_slice(&rx_bytes[..size]);
        Ok(size)
    }

    /// Wait for a packet from the inner network.
    pub fn read(&mut self) -> Result<Packet> {
        loop {
            if_chain::if_chain! {
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
                        self.write(pong)?;
                    }

                    return Ok(packet);
                }
            }

            match self.try_read_bytes() {
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
