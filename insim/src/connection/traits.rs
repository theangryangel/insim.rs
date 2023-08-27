use std::net::SocketAddr;

use if_chain::if_chain;
use bytes::{BytesMut, Buf, BufMut};
use insim_core::{Decodable, Encodable};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

use crate::codec::Mode;
use crate::packets::Packet;
use crate::result::Result;
use crate::error::Error;

#[async_trait::async_trait]
pub trait ReadPacket {
    /// Read a packet
    async fn read(&mut self) -> Result<Option<Packet>>;
}

#[async_trait::async_trait]
pub trait WritePacket {
    /// Write a packet
    async fn write(&mut self, packet: Packet) -> Result<()>;
}

pub trait ReadWritePacket: ReadPacket + WritePacket {
    fn boxed<'a>(self) -> Box<dyn ReadWritePacket + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }
}

pub struct Codec {
    mode: Mode,
}

impl Codec {
    fn encode(&mut self, msg: Packet, dst: &mut BytesMut) -> Result<()> {
        let mut buf = BytesMut::new();
        msg.encode(&mut buf, None)?;

        let n = self.mode.encode_length(&mut buf)?;

        // Reserve capacity in the destination buffer to fit the frame and
        // length field (plus adjustment).
        dst.reserve(n + 1);

        dst.put_u8(n as u8);

        // Write the frame to the buffer
        dst.extend_from_slice(&buf[..]);

        Ok(())
    }

    pub fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Packet>> {
        if src.is_empty() {
            return Ok(None);
        }

        let n = match self.mode.decode_length(src)? {
            Some(n) => n,
            None => {
                return Ok(None);
            }
        };

        let mut data = src.split_to(n);

        // skip over the size field now that we know we have a full packet
        // none of the packet definitions include the size
        data.advance(1);

        let res = Packet::decode(&mut data, None);

        match res {
            Ok(packet) => {
                tracing::debug!("decoded: {:?}", packet);
                Ok(Some(packet))
            }
            Err(e) => {
                tracing::error!("unhandled error: {:?}, data: {:?}", e, data);
                Err(e.into())
            }
        }

    }
}

pub struct Tcp {
    inner: TcpStream,
    codec: Codec,
    buffer: BytesMut,
}

impl Tcp {
    pub fn new(stream: TcpStream, mode: Mode) -> Self {
        let buffer = BytesMut::with_capacity(512);
        let codec = Codec {
            mode,
        };

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
                },
                Ok(_) => {
                    continue;
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                },
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


pub struct Udp {
    inner: UdpSocket,
    codec: Codec,
}

impl Udp {
    pub fn new(socket: UdpSocket, mode: Mode) -> Self {
        let codec = Codec {
            mode,
        };

        Self {
            inner: socket,
            codec,
        }
    }
}

impl ReadWritePacket for Udp {}

#[async_trait::async_trait]
impl ReadPacket for Udp {
    async fn read(&mut self) -> Result<Option<Packet>> {
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
                            return Ok(None);
                        }

                        // skip over the size, we always know we have a full packet
                        buffer.advance(1);

                        let res = Packet::decode(&mut buffer, None)?;
                        return Ok(Some(res));
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    },
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
