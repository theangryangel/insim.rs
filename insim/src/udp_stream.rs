use std::io::{Read, Write};
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{ToSocketAddrs, UdpSocket},
};

/// A wrapper around tokio::net::UdpSocket to make it easier to work with.
pub struct UdpStream {
    inner: UdpSocket,
}

impl UdpStream {
    pub async fn connect<A: ToSocketAddrs, B: ToSocketAddrs>(
        local_addr: A,
        remote_addr: B,
    ) -> tokio::io::Result<Self> {
        let inner = UdpSocket::bind(local_addr).await?;
        inner.connect(remote_addr).await?;

        Ok(Self { inner })
    }

    pub fn peer_addr(&self) -> tokio::io::Result<SocketAddr> {
        self.inner.peer_addr()
    }

    pub fn local_addr(&self) -> tokio::io::Result<SocketAddr> {
        self.inner.local_addr()
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
