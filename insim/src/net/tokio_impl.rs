use std::{io, time::Duration};

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncWriteExt, Interest},
    net::{TcpStream, UdpSocket},
};

use super::{Codec, DEFAULT_TIMEOUT_SECS};
use crate::{Error, Packet, Result, insim::TinyType};

#[derive(Debug)]
enum Transport {
    Tcp(TcpStream, BytesMut),
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
            inner: Transport::Tcp(stream, BytesMut::new()),
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

    /// Asynchronously wait for a packet from the inner network.
    ///
    /// Cancel-safe: every await point inside this function is itself cancel-safe
    /// (`stream.ready()`, `socket.recv_buf()`, `socket.send()`), and all
    /// in-flight state (decoded bytes, pending outbound bytes, keepalive flag)
    /// lives on `self`. Dropping the returned future from a `tokio::select!`
    /// loses no progress.
    pub async fn read(&mut self) -> Result<Packet> {
        // XXX: Why does this look complicated? Because I was daft enough to make the library
        // automatically send keepalive packets, which made the original version of this cancel
        // unsafe!
        // This is about as good as we can do without breaking userspace completely.
        loop {
            // Split the parent borrows so we can use codec and inner independently.
            let Framed { inner, codec } = self;

            if codec.reached_timeout() {
                return Err(Error::Timeout(
                    "Timeout exceeded, no keepalive or packet received".into(),
                ));
            }
            if let Some(packet) = codec.decode()? {
                return Ok(packet);
            }

            match inner {
                Transport::Tcp(stream, write_buf) => {
                    if let Some(ka) = codec.keepalive() {
                        let buf = codec.encode(&ka)?;
                        write_buf.extend_from_slice(buf.as_ref());
                    }

                    let interest = if write_buf.is_empty() {
                        Interest::READABLE
                    } else {
                        Interest::READABLE | Interest::WRITABLE
                    };

                    let ready = tokio::time::timeout(
                        Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                        stream.ready(interest),
                    )
                    .await??;

                    if ready.is_readable() {
                        match stream.try_read_buf(codec.buf_mut()) {
                            Ok(0) => return Err(Error::Disconnected),
                            Ok(_) => {},
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {},
                            Err(e) => return Err(e.into()),
                        }
                    }
                    if ready.is_writable() {
                        while write_buf.has_remaining() {
                            match stream.try_write(write_buf) {
                                Ok(n) => write_buf.advance(n),
                                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => return Err(e.into()),
                            }
                        }
                    }
                },

                Transport::Udp(socket) => {
                    if let Some(ka) = codec.keepalive() {
                        let buf = codec.encode(&ka)?;
                        if !buf.is_empty() {
                            let _ = socket.send(buf.as_ref()).await?;
                        }
                    }
                    let outcome = tokio::time::timeout(
                        Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                        socket.recv_buf(codec.buf_mut()),
                    )
                    .await??;
                    if outcome == 0 {
                        return Err(Error::Disconnected);
                    }
                },
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
            Transport::Tcp(stream, write_buf) => {
                write_buf.extend_from_slice(buf.as_ref());
                while write_buf.has_remaining() {
                    let n = stream.write_buf(write_buf).await?;
                    if n == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            format!(
                                "short TCP send: 0 bytes written, {} pending",
                                write_buf.len()
                            ),
                        )
                        .into());
                    }
                }
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

    /// Asynchronously flush the inner network and shutdown.
    ///
    /// `write(TinyType::Close)` drains `pending_out` end-to-end before returning
    /// (it loops on `write_buf.has_remaining()`), so by the time we reach
    /// `stream.shutdown()` no outbound bytes are still buffered.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.write(TinyType::Close).await?;
        if let Transport::Tcp(stream, _) = &mut self.inner {
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
