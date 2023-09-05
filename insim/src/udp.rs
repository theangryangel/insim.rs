use bytes::{Buf, BytesMut};
use insim_core::Decodable;
use tokio::net::UdpSocket;

use crate::codec::{Codec, Mode};
use crate::error::Error;
use crate::packets::Packet;
use crate::result::Result;

use crate::traits::{ReadPacket, ReadWritePacket, WritePacket};

pub struct Udp {
    inner: UdpSocket,
    codec: Codec,
}

impl Udp {
    pub fn new(socket: UdpSocket, mode: Mode) -> Self {
        let codec = Codec { mode };

        Self {
            inner: socket,
            codec,
        }
    }
}

impl ReadWritePacket for Udp {}

#[async_trait::async_trait]
impl ReadPacket for Udp {
    async fn read(&mut self) -> Result<Packet> {
        // UDP packets from Insim are never fragmented
        // so we can just skip over using the codec to encode/decode.
        // Tokio docs indicates that the buffer must be large enough for any packet.
        // I've picked 1492 because its the effectively a common MTU size across the internet
        // still, and should give some future proofing if any packets insim increase
        // in size

        loop {
            let ready = self.inner.ready(tokio::io::Interest::READABLE).await?;

            if ready.is_readable() {
                let mut buffer = BytesMut::with_capacity(1492);

                match self.inner.try_recv_buf(&mut buffer) {
                    Ok(_) => {
                        if buffer.is_empty() {
                            return Err(Error::Disconnected);
                        }

                        // skip over the size, we always know we have a full packet
                        // TODO: We should probably verify the length matches
                        buffer.advance(1);

                        return Ok(Packet::decode(&mut buffer, None)?);
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
}

#[async_trait::async_trait]
impl WritePacket for Udp {
    async fn write(&mut self, packet: Packet) -> Result<()> {
        let mut buf = BytesMut::new();

        self.codec.encode(packet, &mut buf)?;
        if !buf.is_empty() {
            self.inner.send(&buf).await?;
        }

        Ok(())
    }
}
