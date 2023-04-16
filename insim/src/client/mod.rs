//! Insim Client that maintains a connection and provides a Stream and Sink of
//! [Packets](crate::packets::Packet).

pub mod builder;
pub use builder::ClientBuilder;

#[cfg(test)]
mod tests;

use crate::{
    codec::Codec,
    error,
    packets::{insim, Packet},
    result::Result,
    udp_stream::UdpStream,
};
use futures::Future;
use futures::{Sink, Stream};
use pin_project::pin_project;

use futures::{SinkExt, StreamExt};
use insim_core::identifiers::RequestId;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{fmt::Display, pin::Pin};
use tokio::time;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_util::codec::Framed;

const TIMEOUT_SECS: u64 = 90;

pub trait ClientTransport:
    Sink<Packet, Error = error::Error> + Stream<Item = Result<Packet>> + std::marker::Unpin
{
}

impl<T> ClientTransport for Framed<T, Codec> where T: AsyncRead + AsyncWrite + std::marker::Unpin {}

#[cfg(feature = "tcp")]
pub type TcpClientTransport = Framed<TcpStream, Codec>;

#[cfg(feature = "udp")]
pub type UdpClientTransport = Framed<UdpStream, Codec>;

/// Internal Client State.
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub enum ClientState {
    #[default]
    Disconnected,
    Connected,
    Shutdown,
}

impl Display for ClientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connected => write!(f, "Connected"),
            Self::Shutdown => write!(f, "Shutdown"),
        }
    }
}

/// A Stream and Sink based client for the Insim protocol.
/// Given something that implements the [ClientTransport] trait, Client will handle
/// encoding and decoding of [Packets](Packet), and ensure that the connection
/// is maintained through [insim::Tiny] keepalive packets, and handling any timeout.
#[pin_project]
pub struct Client<T> {
    #[pin]
    inner: T,
    // Cant use pin_project on tokio::time::Sleep because it's !Unpin
    // meaning we can't then use tokio::select! later. So it needs to be boxed.
    deadline: Pin<Box<time::Sleep>>,
    duration: Duration,
    poll_deadline: bool,
    state: ClientState,
    pong: bool,
}

impl<T> Client<T>
where
    T: ClientTransport,
{
    pub fn new(inner: T) -> Client<T> {
        let duration = time::Duration::new(TIMEOUT_SECS, 0);
        let next = time::Instant::now() + duration;
        let deadline = Box::pin(tokio::time::sleep_until(next));

        Client {
            inner,
            deadline,
            duration,
            poll_deadline: true,
            state: ClientState::Connected,
            pong: false,
        }
    }

    pub fn state(&self) -> ClientState {
        self.state
    }

    /// Handle the verification of a Transport.
    /// Is Insim server responding the correct version?
    /// Have we received an initial ping response?
    async fn verify(
        mut self: Pin<&mut Self>,
        verify_version: bool,
        wait_for_pong: bool,
    ) -> Result<()> {
        if wait_for_pong {
            // send a ping!
            self.as_mut()
                .project()
                .inner
                .send(
                    crate::packets::insim::Tiny {
                        reqi: RequestId(2),
                        subtype: crate::packets::insim::TinyType::Ping,
                    }
                    .into(),
                )
                .await?;
        }

        let mut received_vers = !verify_version;
        let mut received_tiny = !wait_for_pong;

        while !received_tiny && !received_vers {
            match self.as_mut().project().inner.next().await {
                None => {
                    return Err(error::Error::Disconnected);
                }
                Some(Err(e)) => {
                    return Err(e);
                }
                Some(Ok(Packet::Tiny(_))) => {
                    received_tiny = true;
                }
                Some(Ok(Packet::Version(insim::Version { insimver, .. }))) => {
                    if insimver != crate::packets::VERSION {
                        return Err(error::Error::IncompatibleVersion(insimver));
                    }

                    received_vers = true;
                }
                Some(Ok(m)) => {
                    /* not the droids we're looking for */
                    tracing::info!("received packet whilst waiting for version and/or ping: {m:?}");
                }
            }
        }
        Ok(())
    }

    /// Send an Insim Init (ISI) packet, and attempt verification of the connection
    /// live-ness.
    pub async fn handshake(
        mut self: Pin<&mut Self>,
        timeout: Duration,
        isi: crate::packets::insim::Isi,
        wait_for_pong: bool,
        verify_version: bool,
    ) -> Result<()> {
        self.send(isi).await?;

        time::timeout(timeout, self.verify(wait_for_pong, verify_version)).await??;

        Ok(())
    }

    // Convenience method to call handshake on an unpinned Client.
    pub async fn handshake_unpin(
        &mut self,
        timeout: Duration,
        isi: crate::packets::insim::Isi,
        wait_for_pong: bool,
        verify_version: bool,
    ) -> Result<()> {
        Pin::new(self)
            .handshake(timeout, isi, wait_for_pong, verify_version)
            .await
    }

    pub fn shutdown(&mut self) {
        self.state = ClientState::Shutdown;
    }

    fn poll_pong(mut self: Pin<&mut Self>, cx: &mut Context) {
        if !*self.as_mut().project().pong {
            return;
        }

        tracing::debug!("ping? pong!");

        let res = self.as_mut().project().inner.poll_ready(cx);
        if !res.is_ready() {
            cx.waker().wake_by_ref();
            return;
        }

        let res = self.as_mut().project().inner.start_send(
            insim::Tiny {
                subtype: insim::TinyType::None,
                ..Default::default()
            }
            .into(),
        );
        if res.is_err() {
            // TODO: Probably need to fix this.
            cx.waker().wake_by_ref();
            return;
        }

        // TODO: docs suggest we don't need to call poll_flush, but if we don't then nothing gets
        // sent?
        let res = self.as_mut().project().inner.poll_flush(cx);
        if res.is_ready() {
            *self.as_mut().project().pong = false;
        }
    }
}

impl<T> Stream for Client<T>
where
    T: ClientTransport,
{
    type Item = Result<Packet>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        // This is cribbed from tokio_stream::StreamExt::Timeout.

        let (lower, upper) = self.inner.size_hint();

        // The timeout stream may insert an error before and after each message
        // from the underlying stream, but no more than one error between each
        // message. Hence the upper bound is computed as 2x+1.

        // Using a helper function to enable use of question mark operator.
        fn twice_plus_one(value: Option<usize>) -> Option<usize> {
            value?.checked_mul(2)?.checked_add(1)
        }

        (lower, twice_plus_one(upper))
    }

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if *self.as_mut().project().state == ClientState::Shutdown {
            *self.as_mut().project().state = ClientState::Disconnected;
            return Poll::Ready(None);
        }

        if *self.as_mut().project().state == ClientState::Disconnected {
            tracing::error!("polled after disconnect");
            return Poll::Ready(None);
        }

        if *self.as_mut().project().pong {
            // do we have a pre-existing ping request that couldn't be previously sent for some
            // reason?
            self.as_mut().poll_pong(cx);
        }

        match self.as_mut().project().inner.poll_next(cx) {
            Poll::Pending => {}
            Poll::Ready(v) => {
                let next = time::Instant::now() + *self.as_mut().project().duration;
                self.as_mut().project().deadline.as_mut().reset(next);
                *self.as_mut().project().poll_deadline = true;

                match v {
                    Some(Ok(frame)) => {
                        if let Packet::Tiny(insim::Tiny {
                            reqi: RequestId(0),
                            subtype: insim::TinyType::None,
                        }) = frame
                        {
                            // attempt to send a ping response immediately.
                            *self.as_mut().project().pong = true;
                            self.as_mut().poll_pong(cx);
                        }

                        return Poll::Ready(Some(Ok(frame)));
                    }
                    Some(Err(e)) => {
                        return Poll::Ready(Some(Err(e)));
                    }
                    None => {}
                }
            }
        };

        if *self.as_mut().project().poll_deadline {
            match self.as_mut().project().deadline.as_mut().poll(cx) {
                Poll::Ready(t) => t,
                Poll::Pending => return Poll::Pending,
            };
            *self.as_mut().project().poll_deadline = false;
            *self.as_mut().project().state = ClientState::Disconnected;
            return Poll::Ready(Some(Err(error::Error::Timeout(
                "Keepalive (ping) timeout".into(),
            ))));
        }

        Poll::Pending
    }
}

impl<T, P> Sink<P> for Client<T>
where
    T: ClientTransport,
    P: std::fmt::Debug + Into<Packet>,
{
    type Error = <T as futures::Sink<Packet>>::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, value: P) -> Result<()> {
        tracing::info!("asked to send {value:?}");
        self.project().inner.start_send(value.into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_close(cx)
    }
}