//! UDPStream

use std::{
    io::{Read, Write},
    pin::Pin,
    task::{Context, Poll},
};

use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::UdpSocket,
};

/// Tokio UDPSocket wrapper for AsyncRead, AsyncWrite, Read and Write
#[derive(Debug)]
pub struct UdpStream {
    // FIXME: Add internal buf like we've done with WebSocketStream
    inner: UdpSocket,
}

impl From<UdpSocket> for UdpStream {
    fn from(value: UdpSocket) -> Self {
        Self { inner: value }
    }
}

impl Read for UdpStream {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, std::io::Error> {
        self.inner.try_recv(buff)
    }
}

impl Write for UdpStream {
    fn write(&mut self, buff: &[u8]) -> Result<usize, std::io::Error> {
        self.inner.try_send(buff)
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
