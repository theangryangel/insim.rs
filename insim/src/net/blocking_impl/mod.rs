use std::{
    fmt::Debug,
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
    time::Duration,
};

use bytes::{BufMut, BytesMut};

use super::{Codec, DEFAULT_TIMEOUT_SECS};
use crate::{insim::Isi, result::Result, Error, Packet, DEFAULT_BUFFER_CAPACITY};

/// TryReadWriteBytes
pub trait TryReadWriteBytes {
    /// Set the timeout
    fn try_read_write_timeout(&self, timeout: Duration);

    /// Try to read bytes.
    fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize>;

    /// Try to write bytes.
    fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize>;
}

impl TryReadWriteBytes for TcpStream {
    fn try_read_write_timeout(&self, timeout: Duration) {
        self.set_read_timeout(Some(timeout))
            .expect("set_read_timeout failed");
        self.set_write_timeout(Some(timeout))
            .expect("set_write_timeout failed");
    }

    #[allow(unsafe_code)]
    fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize> {
        // TODO: Remove when read_buf becomes stable.
        // See https://users.rust-lang.org/t/how-to-read-from-tcpstream-and-append-to-vec-u8-efficiently/89059/4
        // for why this is "safe"

        let size = {
            let chunk = buf.chunk_mut();
            let len = chunk.len();

            let ptr = chunk.as_mut_ptr();
            // SAFETY: The file is not going to read from the uninitialized data.
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
            self.read(slice)?
        };

        // SAFETY: The tcpstream just initialized that many bytes.
        unsafe {
            buf.advance_mut(size);
        }
        Ok(size)
    }

    fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize> {
        Ok(self.write(src)?)
    }
}

impl TryReadWriteBytes for UdpSocket {
    fn try_read_write_timeout(&self, timeout: Duration) {
        self.set_read_timeout(Some(timeout))
            .expect("set_read_timeout failed");
        self.set_write_timeout(Some(timeout))
            .expect("set_write_timeout failed");
    }

    fn try_read_bytes(&mut self, buf: &mut BytesMut) -> Result<usize> {
        let mut rx_bytes = [0u8; crate::MAX_SIZE_PACKET];
        let size = self.recv(&mut rx_bytes)?;
        buf.extend_from_slice(&rx_bytes[..size]);
        Ok(size)
    }

    fn try_write_bytes(&mut self, src: &[u8]) -> Result<usize> {
        Ok(self.send(src)?)
    }
}

/// A unified wrapper around anything that implements Read + Write.
/// You probably really want to look at [Framed].
#[derive(Debug)]
pub struct FramedInner<N>
where
    N: TryReadWriteBytes,
{
    inner: N,
    codec: Codec,
    buffer: BytesMut,
    verify_version: bool,
}

impl<N> FramedInner<N>
where
    N: TryReadWriteBytes,
{
    /// Create a new FramedInner, which wraps some kind of network transport.
    pub fn new(inner: N, codec: Codec) -> Self {
        let buffer = BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY);
        inner.try_read_write_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS));

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
        self.write(isi.into())?;

        Ok(())
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

            match self.inner.try_read_bytes(&mut self.buffer) {
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
    pub fn write(&mut self, packet: Packet) -> Result<()> {
        let buf = self.codec.encode(&packet)?;
        if !buf.is_empty() {
            let _ = self.inner.try_write_bytes(&buf)?;
        }

        Ok(())
    }
}

/// Concrete enum of connection types, to avoid Box'ing. Wraps [FramedInner].
// The "Inner" connection for Connection, so that we can avoid Box'ing
// Since the ConnectionOptions is all very hard coded, for "high level" API usage,
// I think this fine.
// i.e. if we add a Websocket option down the line, then ConnectionOptions needs to understand it
// therefore we cannot just box stuff magically anyway.
pub enum Framed {
    /// Tcp
    Tcp(FramedInner<TcpStream>),

    /// Udp
    Udp(FramedInner<UdpSocket>),
}

impl Framed {
    #[tracing::instrument]
    /// Wait for a packet from the inner network.
    pub fn read(&mut self) -> Result<Packet> {
        let res = match self {
            Self::Tcp(i) => i.read(),
            Self::Udp(i) => i.read(),
        };
        tracing::debug!("read result {:?}", res);
        res
    }

    #[tracing::instrument]
    /// Write a packet to the inner network.
    pub fn write<I: Into<Packet> + Send + Sync + Debug>(&mut self, packet: I) -> Result<()> {
        tracing::debug!("writing packet {:?}", &packet);
        match self {
            Self::Tcp(i) => i.write(packet.into()),
            Self::Udp(i) => i.write(packet.into()),
        }
    }
}

impl Debug for Framed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Framed::Tcp(i) => write!(
                f,
                "Framed::Tcp {{ codec: {:?}, verify_version: {:?} }}",
                i.codec, i.verify_version
            ),
            Framed::Udp(i) => write!(
                f,
                "Framed::Tcp {{ codec: {:?}, verify_version: {:?} }}",
                i.codec, i.verify_version
            ),
        }
    }
}
