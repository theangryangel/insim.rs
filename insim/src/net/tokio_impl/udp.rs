//! UDPStream

use std::{
    io::{Read, Write},
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::UdpSocket,
};

use crate::DEFAULT_BUFFER_CAPACITY;

/// Tokio UDPSocket wrapper for AsyncRead, AsyncWrite, Read and Write
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

impl From<UdpSocket> for UdpStream {
    fn from(value: UdpSocket) -> Self {
        Self {
            inner: value,
            buffer: BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY),
        }
    }
}

impl Read for UdpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        // lets clear out our internal buffer first
        if !self.buffer.is_empty() {
            return self.read_into_buffer(buf);
        }

        let mut rx_bytes = [0u8; crate::MAX_SIZE_PACKET];
        let size = self.inner.try_recv(&mut rx_bytes)?;

        self.buffer.extend_from_slice(&rx_bytes[..size]);
        self.read_into_buffer(buf)
    }
}

impl Write for UdpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.inner.try_send(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl AsyncRead for UdpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.inner.poll_recv(cx, buf) {
            Poll::Ready(Ok(_n)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for UdpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.inner.poll_send(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
