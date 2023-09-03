use bytes::BytesMut;
use if_chain::if_chain;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::codec::{Codec, Mode};
use crate::error::Error;
use crate::packets::Packet;
use crate::result::Result;

use crate::traits::{ReadPacket, ReadWritePacket, WritePacket};

pub struct Tcp {
    inner: TcpStream,
    codec: Codec,
    buffer: BytesMut,
}

impl Tcp {
    pub fn new(stream: TcpStream, mode: Mode) -> Self {
        let buffer = BytesMut::with_capacity(512);
        let codec = Codec { mode };

        Self {
            inner: stream,
            codec,
            buffer,
        }
    }
}

impl ReadWritePacket for Tcp {}

#[async_trait::async_trait]
impl ReadPacket for Tcp {
    async fn read(&mut self) -> Result<Option<Packet>> {
        loop {
            if_chain! {
                if !self.buffer.is_empty();
                if let Some(packet) = self.codec.decode(&mut self.buffer)?;
                then {
                    return Ok(Some(packet));
                }
            }

            // Wait for the socket to be readable.
            // This is cancel safe.
            self.inner.readable().await?;

            match self.inner.try_read_buf(&mut self.buffer) {
                Ok(0) => {
                    // The remote closed the connection. For this to be a clean
                    // shutdown, there should be no data in the read buffer. If
                    // there is, this means that the peer closed the socket while
                    // sending a frame.
                    if self.buffer.is_empty() {
                        return Ok(None);
                    }

                    return Err(Error::Disconnected);
                }
                Ok(_) => {
                    continue;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl WritePacket for Tcp {
    async fn write(&mut self, packet: Packet) -> Result<()> {
        let mut buf = BytesMut::new();

        self.codec.encode(packet, &mut buf)?;
        if !buf.is_empty() {
            self.inner.write_all(&buf).await?;
        }

        Ok(())
    }
}
