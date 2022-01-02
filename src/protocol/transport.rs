//! Lower level transport interface.
use crate::{
    error,
    protocol::{codec, insim, Packet},
};
use futures::Future;
use futures::{Sink, Stream};
use pin_project_lite::pin_project;

use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time;
use tokio_util::codec::Framed;

use super::codec::Mode;

const TIMEOUT_SECS: u64 = 90;

/// Internal Transport state.
#[derive(Eq, PartialEq)]
pub enum TransportState {
    Disconnected,
    Connected,

    Shutdown,
}

pin_project! {
    /// A lower-level Stream and Sink based transport layer for the insim protocol.
    /// Given a `AsyncRead` and `AsyncWrite`, this struct will handle encoding and decoding of
    /// [Packets](Packet), and ensure that the connection is maintained through keepalive packets.
    pub struct Transport<T>
    {
        #[pin]
        inner: Framed<T, codec::Codec>,
        // Cant use pin_project on tokio::time::Sleep because it's !Unpin
        // meaning we can't then use tokio::select! later.
        // Lovely.
        // TODO: Do we bother keeping pin_project in that case?
        deadline: Pin<Box<time::Sleep>>,
        duration: Duration,
        poll_deadline: bool,
        state: TransportState,
    }
}

impl<T> Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    pub fn new(inner: T, codec_mode: Mode) -> Transport<T> {
        let inner = Framed::new(inner, codec::Codec::new(codec_mode));

        let duration = time::Duration::new(TIMEOUT_SECS, 0);
        let next = time::Instant::now() + duration;
        let deadline = Box::pin(tokio::time::sleep_until(next));

        Transport {
            inner,
            deadline,
            duration,
            poll_deadline: true,
            state: TransportState::Connected,
        }
    }

    pub fn shutdown(&mut self) {
        self.state = TransportState::Shutdown;
    }
}

impl<T> Stream for Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = Result<Packet, error::Error>;

    fn size_hint(&self) -> (usize, Option<usize>) {
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

    #[allow(unused_must_use)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // TODO: Fixup all the dereferencing and make it coherent rather than Just Make It Compile.
        // TODO: remove the allow unused

        let this = self.as_mut().project();

        // TODO: Is this really the right way to do this?
        if *this.state == TransportState::Shutdown {
            *this.state = TransportState::Disconnected;
            return Poll::Ready(None);
        }

        if *this.state == TransportState::Disconnected {
            tracing::error!("polled after disconnect");
            return Poll::Ready(None);
        }

        match this.inner.poll_next(cx) {
            Poll::Pending => {}
            Poll::Ready(v) => {
                let next = time::Instant::now() + *this.duration;
                this.deadline.as_mut().reset(next);
                *this.poll_deadline = true;

                match v {
                    Some(Ok(frame)) => {
                        if let Packet::Tiny(insim::Tiny {
                            reqi: 0,
                            subtype: insim::TinyType::None,
                        }) = frame
                        {
                            tracing::debug!("ping? pong!");

                            // TODO: This is absolutely not the way to do this.
                            // We either using call start_send or we should at least check the result.
                            // FIXME.
                            self.as_mut()
                                .project()
                                .inner
                                .start_send(Packet::from(insim::Tiny {
                                    reqi: 0,
                                    subtype: insim::TinyType::None,
                                }));
                            self.as_mut().project().inner.poll_flush(cx);
                        }

                        return Poll::Ready(Some(Ok(frame)));
                    }
                    Some(Err(e)) => {
                        return Poll::Ready(Some(Err(e.into())));
                    }
                    None => {}
                }
            }
        };

        if *this.poll_deadline {
            match this.deadline.as_mut().poll(cx) {
                Poll::Ready(t) => t,
                Poll::Pending => return Poll::Pending,
            };
            *this.poll_deadline = false;
            *this.state = TransportState::Disconnected;
            return Poll::Ready(Some(Err(error::Error::Timeout)));
        }

        Poll::Pending
    }
}

impl<T, I: Into<Packet>> Sink<I> for Transport<T>
where
    T: AsyncRead + AsyncWrite,
    I: std::fmt::Debug,
{
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, value: I) -> Result<(), Self::Error> {
        self.project().inner.start_send(value.into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}
