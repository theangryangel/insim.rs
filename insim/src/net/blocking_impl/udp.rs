//! UdpStream

use std::{
    io::{Read, Write},
    net::UdpSocket,
};

use bytes::{Buf, BytesMut};

use crate::MAX_SIZE_PACKET;

impl From<UdpSocket> for UdpStream {
    fn from(value: UdpSocket) -> Self {
        Self {
            inner: value,
            buffer: BytesMut::with_capacity(MAX_SIZE_PACKET),
        }
    }
}

/// Udp "stream" wrapper.
/// By default UdpSocket doesnt behave like TcpStream and when calling recv any data that cannot
/// fit inside of the passed buffer is lost. This UdpStream implementation papers over this fact,
/// allowing us to safely implement Read.
#[derive(Debug)]
pub struct UdpStream {
    inner: UdpSocket,
    buffer: BytesMut,
}

impl UdpStream {
    fn read_into_buffer(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let to_copy = buf.len().min(self.buffer.len());
        self.buffer
            .copy_to_bytes(to_copy)
            .copy_to_slice(&mut buf[..to_copy]);
        Ok(to_copy)
    }
}

impl Read for UdpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // lets clear out our internal buffer first
        if !self.buffer.is_empty() {
            return self.read_into_buffer(buf);
        }

        // we need an internal buffer for *this receive call* to ensure that the internal udpsocket
        // does not truncate anything. We cannot guarantee that a chunk from the internal BytesMut
        // buffer will have the space we require.
        let mut rx_bytes = [0u8; crate::MAX_SIZE_PACKET];
        let size = self.inner.recv(&mut rx_bytes)?;
        self.buffer.extend_from_slice(&rx_bytes[..size]);
        self.read_into_buffer(buf)
    }
}

impl Write for UdpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.send(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
