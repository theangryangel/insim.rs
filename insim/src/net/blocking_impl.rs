use std::{
    io::{self, Read, Write},
    net::{TcpStream, UdpSocket},
};

use super::Codec;
use crate::{Error, MAX_SIZE_PACKET, Packet, Result, insim::TinyType};

#[derive(Debug)]
enum Transport {
    Tcp(TcpStream),
    Udp(UdpSocket),
}

#[derive(Debug)]
/// A convenience wrapper around Udp and Tcp, using Codec to drive the IO
pub struct Framed {
    inner: Transport,
    codec: Codec,
    scratch: [u8; MAX_SIZE_PACKET], // reused across reads
}

impl Framed {
    /// Create a new Framed from a TcpStream
    pub fn from_tcp(stream: TcpStream, codec: Codec) -> Self {
        Self {
            inner: Transport::Tcp(stream),
            codec,
            scratch: [0; MAX_SIZE_PACKET],
        }
    }

    /// Create a new Framed from a UdpSocket
    pub fn from_udp(socket: UdpSocket, codec: Codec) -> Self {
        Self {
            inner: Transport::Udp(socket),
            codec,
            scratch: [0; MAX_SIZE_PACKET],
        }
    }

    fn read_inner(&mut self) -> io::Result<usize> {
        let n = match &mut self.inner {
            Transport::Tcp(stream) => stream.read(&mut self.scratch)?,
            Transport::Udp(socket) => socket.recv(&mut self.scratch)?,
        };
        let dst = self.codec.buf_mut();
        dst.reserve(n);
        dst.extend_from_slice(&self.scratch[..n]);
        Ok(n)
    }

    /// Wait for a packet from the inner network.
    pub fn read(&mut self) -> Result<Packet> {
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
                self.write(keepalive)?;
            }
            let n = self.read_inner()?;
            if n == 0 {
                return Err(Error::Disconnected);
            }
        }
    }

    /// Write a packet to the inner network.
    pub fn write<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        let buf = self.codec.encode(&packet.into())?;
        if buf.is_empty() {
            return Ok(());
        }
        match &mut self.inner {
            Transport::Tcp(stream) => stream.write_all(&buf)?,
            Transport::Udp(socket) => {
                let n = socket.send(&buf)?;
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

    /// Flush the inner network
    pub fn flush(&mut self) -> Result<()> {
        if let Transport::Tcp(stream) = &mut self.inner {
            stream.flush()?;
        }
        Ok(())
    }

    /// Shutdown the inner network. For blocking this currently just a flush
    pub fn shutdown(&mut self) -> Result<()> {
        self.write(TinyType::Close)?;
        self.flush()?;
        if let Transport::Tcp(stream) = &mut self.inner {
            let _ = stream.shutdown(std::net::Shutdown::Both);
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
