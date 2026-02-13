use std::{
    fmt::Debug,
    io::{Read, Write},
};

use super::ReadWrite;
use crate::{Error, MAX_SIZE_PACKET, Packet, insim::TinyType, net::Codec, result::Result};

/// A unified wrapper around anything that implements Read + Write.
#[derive(Debug)]
pub struct Framed {
    inner: Box<dyn ReadWrite>,
    codec: Codec,
}

impl Framed {
    /// Create a new FramedInner, which wraps some kind of network transport.
    pub fn new(inner: Box<dyn ReadWrite>, codec: Codec) -> Self {
        Self { inner, codec }
    }

    /// Wait for a packet from the inner network.
    pub fn read(&mut self) -> Result<Packet> {
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
                self.write(keepalive)?;
            }

            if let Some(packet) = packet {
                return Ok(packet);
            }

            let mut buf = [0u8; MAX_SIZE_PACKET];
            match self.inner.read(&mut buf)? {
                0 => {
                    // The remote closed the connection. For this to be a clean
                    // shutdown, there should be no data in the read buffer. If
                    // there is, this means that the peer closed the socket while
                    // sending a frame.
                    return Err(Error::Disconnected);
                },
                amt => {
                    // data
                    self.codec.feed(&buf[..amt]);
                },
            }
        }
    }

    /// Write a packet to the inner network.
    pub fn write<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        let buf = self.codec.encode(&packet.into())?;
        if !buf.is_empty() {
            let _ = self.inner.write_all(&buf)?;
        }

        Ok(())
    }

    /// Flush the inner network
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()?;
        Ok(())
    }

    /// Shutdown the inner network. For blocking this currently just a flush
    pub fn shutdown(&mut self) -> Result<()> {
        self.write(TinyType::Close)?;
        while self.read().is_ok() {}
        self.inner.flush()?;

        Ok(())
    }
}
