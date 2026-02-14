use std::{io, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
    time::timeout,
};

use super::{Codec, DEFAULT_TIMEOUT_SECS};
use crate::{Error, Packet, Result, insim::TinyType};

#[derive(Debug)]
enum Transport {
    Tcp(TcpStream),
    Udp(UdpSocket),
}

/// A convenience wrapper around Udp and Tcp, using Codec to drive the IO
#[derive(Debug)]
pub struct Framed {
    inner: Transport,
    codec: Codec,
}

impl Framed {
    /// Create a new Framed from a TcpStream
    pub fn from_tcp(stream: TcpStream, codec: Codec) -> Self {
        Self {
            inner: Transport::Tcp(stream),
            codec,
        }
    }

    /// Create a new Framed from a UdpSocket
    pub fn from_udp(socket: UdpSocket, codec: Codec) -> Self {
        Self {
            inner: Transport::Udp(socket),
            codec,
        }
    }

    async fn read_inner(&mut self) -> io::Result<usize> {
        let (inner, codec) = (&mut self.inner, &mut self.codec);
        let dst = codec.buf_mut();
        match inner {
            Transport::Tcp(stream) => {
                let n = stream.read_buf(dst).await?;
                Ok(n)
            },
            Transport::Udp(socket) => {
                let n = socket.recv_buf(dst).await?;
                Ok(n)
            },
        }
    }

    /// Asynchronously wait for a packet from the inner network.
    pub async fn read(&mut self) -> Result<Packet> {
        loop {
            if self.codec.reached_timeout() {
                return Err(Error::Timeout(
                    "Timeout exceeded, no keepalive or packet received".into(),
                ));
            }
            if let Some(packet) = self.codec.decode()? {
                return Ok(packet);
            }
            if let Some(keepalive) = self.codec.keepalive() {
                self.write(keepalive).await?;
            }
            let outcome =
                timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), self.read_inner()).await??;
            if outcome == 0 {
                return Err(Error::Disconnected);
            }
        }
    }

    /// Asynchronously write a packet to the inner network.
    pub async fn write<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        let buf = self.codec.encode(&packet.into())?;
        if buf.is_empty() {
            return Ok(());
        }
        match &mut self.inner {
            Transport::Tcp(stream) => {
                stream.write_all(buf.as_ref()).await?;
            },
            Transport::Udp(socket) => {
                // UDP should be whole datagram or error, but we still guard against short sends.
                let n = socket.send(buf.as_ref()).await?;
                if n != buf.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        format!("short UDP send: {n} < {}", buf.len()),
                    )
                    .into());
                }
            },
        }
        Ok(())
    }

    /// Asynchronously flush the inner network
    pub async fn flush(&mut self) -> Result<()> {
        if let Transport::Tcp(stream) = &mut self.inner {
            stream.flush().await?;
        }
        Ok(())
    }

    /// Asynchronously flush the inner network and shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        self.write(TinyType::Close).await?;
        if let Transport::Tcp(stream) = &mut self.inner {
            stream.shutdown().await?;
        }
        Ok(())
    }
}

impl From<TcpStream> for Framed {
    fn from(value: TcpStream) -> Self {
        Self::from_tcp(value, Codec::new())
    }
}

impl From<UdpSocket> for Framed {
    fn from(value: UdpSocket) -> Self {
        Self::from_udp(value, Codec::new())
    }
}
