use futures::Future;
use futures::Stream;
use insim_core::identifiers::RequestId;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time;

use super::{Connection, PacketSinkStream, State};
use crate::error::Error;
use crate::packets::insim::{Tiny, TinyType};
use crate::packets::Packet;
use crate::result::Result;

impl<T> Stream for Connection<T>
where
    T: PacketSinkStream,
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
        if *self.as_mut().project().state == State::Shutdown {
            *self.as_mut().project().state = State::Disconnected;
            return Poll::Ready(None);
        }

        if *self.as_mut().project().state == State::Disconnected {
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
                        if let Packet::Tiny(Tiny {
                            reqi: RequestId(0),
                            subt: TinyType::None,
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
            *self.as_mut().project().state = State::Disconnected;
            return Poll::Ready(Some(Err(Error::Timeout("Keepalive (ping) timeout".into()))));
        }

        Poll::Pending
    }
}
