use bytes::{BytesMut, Bytes};
use if_chain::if_chain;
use insim_core::{Encodable, Decodable};
use crate::{error::Error, result::Result};

pub mod codec;
pub mod transport;
pub mod tcp;

pub struct Framed<C, I>
where
    C: codec::Codec,
    I: transport::Transport,
{
    inner: I,
    codec: C,
    buffer: BytesMut,
}

impl<C, I> Framed<C, I>
where
    C: codec::Codec,
    I: transport::Transport,
{
    pub fn new(inner: I, codec: C) -> Self {
        let buffer = BytesMut::new();

        Self {
            inner,
            codec,
            buffer,           
        }
    }

    pub fn version(&self) -> u8 {
        C::VERSION
    }

    pub async fn read(&mut self) -> Result<C::Item> {
        loop {
            if_chain! {
                if !self.buffer.is_empty();
                if let Some(packet) = self.codec.decode(&mut self.buffer)?;
                then {
                    return Ok(packet);
                }
            }

            match self.inner.read(&mut self.buffer).await {
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
            self.inner.write(&mut buf).await?;
        }

        Ok(())
    }
}
