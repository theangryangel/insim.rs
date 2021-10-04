use crate::packets;
use crate::protocol::codec;

use futures::{Sink, Stream};
use std::convert::Into;
use std::io::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::{TcpStream, UdpSocket};
use tokio_util::codec::Framed;
use tokio_util::udp::UdpFramed;

use pin_project::pin_project;

/*
 * InsimPacketStream papers over the implementation details between UDP and TCP, providing a
 * vaguely unified interface.
 *
 * There's probably a better way to do this, but it'll have to do for now.
 *
 * pin_project and EnumProj are used here to avoid unsafe on the Stream and Sink forwarding.
 * See https://github.com/rust-lang/pin-utils/issues/21
 * and https://internals.rust-lang.org/t/idea-enhance-match-ergonomics-to-match-on-pinned-enums-without-unsafe/9317
 */

#[pin_project(project = EnumProj)]
pub enum InsimPacketStream {
    Tcp {
        #[pin]
        inner: Framed<TcpStream, codec::InsimCodec>,
    },

    Udp {
        #[pin]
        inner: UdpFramed<codec::InsimCodec, UdpSocket>,
        peer: SocketAddr,
        local: SocketAddr,
    },
}

impl InsimPacketStream {
    pub async fn new_tcp(dest: String) -> InsimPacketStream {
        // TODO connection timeout
        // TODO handle error
        let stream = TcpStream::connect(dest).await.unwrap();
        let inner = Framed::new(stream, codec::InsimCodec::new());
        InsimPacketStream::Tcp { inner }
    }

    pub async fn new_udp(dest: String) -> InsimPacketStream {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

        let peer = dest.parse().unwrap();
        let local = socket.local_addr().unwrap();

        // TODO unhandled error
        socket.connect(&peer).await;

        let inner = UdpFramed::new(socket, codec::InsimCodec::new());
        InsimPacketStream::Udp { inner, peer, local }
    }

    pub fn local(&mut self) -> Option<SocketAddr> {
        match *self {
            InsimPacketStream::Udp { local, .. } => Some(local),
            _ => None,
        }
    }
}

impl Stream for InsimPacketStream {
    type Item = Result<packets::Insim, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.project() {
            EnumProj::Tcp { inner, .. } => inner.poll_next(cx),
            EnumProj::Udp { inner, .. } => {
                let next = inner.poll_next(cx);

                // We need to do this to drop the peer from Poll::Ready to maintain compatibility
                match next {
                    Poll::Ready(Some(Ok((frame, _peer)))) => {
                        Poll::Ready(Some(std::result::Result::Ok(frame)))
                    }
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

impl<I: Into<packets::Insim>> Sink<I> for InsimPacketStream {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            EnumProj::Tcp { inner, .. } => inner.poll_ready(cx),
            EnumProj::Udp { inner, .. } => inner.poll_ready(cx),
        }
    }

    fn start_send(self: Pin<&mut Self>, value: I) -> Result<(), Self::Error> {
        match self.project() {
            EnumProj::Tcp { inner, .. } => inner.start_send(value.into()),
            EnumProj::Udp { inner, peer, .. } => inner.start_send((value.into(), *peer)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            EnumProj::Tcp { inner, .. } => inner.poll_flush(cx),
            EnumProj::Udp { inner, .. } => inner.poll_flush(cx),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            EnumProj::Tcp { inner, .. } => inner.poll_close(cx),
            EnumProj::Udp { inner, .. } => inner.poll_close(cx),
        }
    }
}
