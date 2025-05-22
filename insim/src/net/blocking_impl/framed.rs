use std::{
    fmt::Debug,
    io::{self, Read, Write},
};

use super::ReadWrite;
use crate::{net::Codec, result::Result, Error, Packet, MAX_SIZE_PACKET};

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
            match self.inner.read(&mut buf) {
                Ok(0) => {
                    // The remote closed the connection. For this to be a clean
                    // shutdown, there should be no data in the read buffer. If
                    // there is, this means that the peer closed the socket while
                    // sending a frame.
                    return Err(Error::Disconnected);
                },
                Ok(amt) => {
                    // data
                    self.codec.feed(&buf[..amt]);
                },
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        continue;
                    }

                    return Err(e.into());
                },
            }
        }
    }

    /// Write a packet to the inner network.
    pub fn write<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        let buf = self.codec.encode(&packet.into())?;
        if !buf.is_empty() {
            let _ = self.inner.write(&buf)?;
            self.inner.flush()?;
        }

        Ok(())
    }
}
