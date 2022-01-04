//! Lower level transport interface.
use crate::{
    error,
    protocol::{codec, insim, Packet},
};
use futures::Future;
use futures::{Sink, Stream};
use pin_project::pin_project;

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

/// A lower-level Stream and Sink based transport layer for the insim protocol.
/// Given a `AsyncRead` and `AsyncWrite`, this struct will handle encoding and decoding of
/// [Packets](Packet), and ensure that the connection is maintained through
/// [insim::Tiny] keepalive packets.
#[pin_project]
pub struct Transport<T> {
    #[pin]
    inner: Framed<T, codec::Codec>,
    // Cant use pin_project on tokio::time::Sleep because it's !Unpin
    // meaning we can't then use tokio::select! later. So it needs to be boxed.
    deadline: Pin<Box<time::Sleep>>,
    duration: Duration,
    poll_deadline: bool,
    state: TransportState,
    pong: bool,
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
            pong: false,
        }
    }

    pub fn shutdown(&mut self) {
        self.state = TransportState::Shutdown;
    }

    fn poll_pong(mut self: Pin<&mut Self>, cx: &mut Context) {
        if !*self.as_mut().project().pong {
            return;
        }

        tracing::debug!("ping? pong!");

        let res = self.as_mut().project().inner.poll_ready(cx);
        if !res.is_ready() {
            return;
        }

        let res = self.as_mut().project().inner.start_send(
            insim::Tiny {
                reqi: 0,
                subtype: insim::TinyType::None,
            }
            .into(),
        );
        if res.is_err() {
            return;
        }

        // TODO: docs suggest we don't need to call poll_flush, but if we don't then nothing gets
        // sent?
        //
        let res = self.as_mut().project().inner.poll_flush(cx);
        if res.is_ready() {
            *self.as_mut().project().pong = false;
        }
    }
}

impl<T> Stream for Transport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = Result<Packet, error::Error>;

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
        if *self.as_mut().project().state == TransportState::Shutdown {
            *self.as_mut().project().state = TransportState::Disconnected;
            return Poll::Ready(None);
        }

        if *self.as_mut().project().state == TransportState::Disconnected {
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
                            reqi: 0,
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
                        return Poll::Ready(Some(Err(e.into())));
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
            *self.as_mut().project().state = TransportState::Disconnected;
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
