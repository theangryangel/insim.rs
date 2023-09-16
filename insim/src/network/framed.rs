use std::time::Duration;
use tokio::time;

use bytes::BytesMut;
use if_chain::if_chain;
use crate::codec::Init;
use crate::{error::Error, result::Result, codec::Packets};

use crate::{
    network::Network, codec::Codec
};

pub struct Framed<C, N>
where
    C: Codec,
    N: Network,
{
    inner: N,
    codec: C,
    buffer: BytesMut,

    verify_version: bool,
}

impl<C, N> Framed<C, N>
where
    C: Codec,
    N: Network,
{
    pub fn new(inner: N, codec: C) -> Self {
        let buffer = BytesMut::new();

        Self {
            inner,
            codec,
            buffer,
            verify_version: false
        }
    }

    pub fn set_verify_version(&mut self, verify_version: bool) {
        self.verify_version = verify_version;
    }

    pub async fn handshake<I: Into<C::Item> + Init>(
        &mut self, 
        isi: I,
        timeout: Duration,
    ) -> Result<()> {
        time::timeout(
            timeout, 
            self.write(isi)
        ).await?
    }

    pub async fn read(&mut self) -> Result<C::Item> {
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
                        self.write(<C::Item>::pong(None)).await?;
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

    pub async fn write<P: Into<C::Item>>(&mut self, packet: P) -> Result<()> {
        let mut buf = BytesMut::new();

        self.codec.encode(&packet.into(), &mut buf)?;
        if !buf.is_empty() {
            self.inner.try_write_bytes(&buf).await?;
        }

        Ok(())
    }
}
