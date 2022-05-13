use futures_core::{ready, stream::TryStream};
use futures_sink::Sink;
use futures_util::stream::FuturesOrdered;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{error, fmt};
use tokio::io::{AsyncRead, AsyncWrite};
//use tower_service::Service;

const YIELD_EVERY: usize = 24;

// TODO: Create a wrapper around Transport called "ReconnectingTransport", or "Connection" or
// something. We had this in a previous version of the crate. We can pull it out of history.
use crate::error::Error;
use crate::protocol::transport::Transport;

/// This type provides a Tower-like implementation.
/// It heavily borrows from the `tower_service` crate.
/// Unfortunately tower caters for a request-response style of communication and Insim is more
/// stream-based.
#[pin_project]
pub struct Client<T, S>
where
    T: AsyncRead + AsyncWrite + TryStream,
    S: Service,
{
    #[pin]
    pending: FuturesOrdered<dyn futures::Stream<Item = crate::protocol::Packet>>,

    #[pin]
    transport: Transport<T>,
    service: S,

    in_flight: usize,
    finish: bool,
}

impl<T, S> Client<T, S>
where
    T: AsyncRead + AsyncWrite + TryStream,
    S: Service,
{
    pub fn new(transport: Transport<T>, service: S) -> Self {
        Self {
            pending: FuturesOrdered::new(),
            transport,
            service,
            in_flight: 0,
            finish: false,
        }
    }
}

impl<T, S> Future for Client<T, S>
where
    T: AsyncRead + AsyncWrite + TryStream,
    S: Service,
{
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let span = tracing::trace_span!("poll");
        let _guard = span.enter();
        tracing::trace!("poll");

        // go through the deref so we can do partial borrows
        let this = self.project();

        // we never move transport or pending, nor do we ever hand out &mut to it
        let mut transport: Pin<_> = this.transport;
        let mut pending: Pin<_> = this.pending;

        // track how many times we have iterated
        let mut i = 0;

        loop {
            // first, poll pending futures to see if any have produced responses
            // note that we only poll for completed service futures if we can send the response
            while let Poll::Ready(r) = transport.as_mut().poll_ready(cx) {
                if let Err(e) = r {
                    return Poll::Ready(Err(e.into()));
                }

                tracing::trace!(
                    in_flight = *this.in_flight,
                    pending = pending.len(),
                    "transport.ready"
                );
                match pending.as_mut().try_poll_next(cx) {
                    Poll::Ready(Some(Err(e))) => {
                        return Poll::Ready(Err(e.into()));
                    }
                    Poll::Ready(Some(Ok(Some(rsp)))) => {
                        tracing::trace!("transport.start_send");
                        // try to send the response!
                        transport.as_mut().start_send(rsp).map_err(|e| e.into())?;
                        *this.in_flight -= 1;
                    }
                    Poll::Ready(Some(Ok(None))) => {
                        // no response but everything is OK
                    }
                    _ => {
                        // XXX: should we "release" the poll_ready we got from the Sink?
                        break;
                    }
                }
            }

            // also try to make progress on sending
            tracing::trace!(finish = *this.finish, "transport.poll_flush");
            if let Poll::Ready(()) = transport.as_mut().poll_flush(cx).map_err(|e| e.into())? {
                if *this.finish && pending.as_mut().is_empty() {
                    // there are no more requests
                    // and we've finished all the work!
                    return Poll::Ready(Ok(()));
                }
            }

            if *this.finish {
                // there's still work to be done, but there are no more requests
                // so no need to check the incoming transport
                return Poll::Pending;
            }

            // if we have run for a while without yielding, yield back so other tasks can run
            i += 1;
            if i == YIELD_EVERY {
                // we're forcing a yield, so need to ensure we get woken up again
                tracing::trace!("forced yield");
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }

            // is the service ready?
            tracing::trace!("service.poll_ready");
            ready!(this.service.poll_ready(cx)).map_err(|e| e.into())?;

            tracing::trace!("transport.poll_next");
            let rq = ready!(transport.as_mut().try_poll_next(cx))
                .transpose()
                .map_err(|e| e.into())?;
            if let Some(rq) = rq {
                // the service is ready, and we have another request!
                // you know what that means:
                pending.push(this.service.call(rq));
                *this.in_flight += 1;
            } else {
                // there are no more requests coming
                // check one more time for responses, and then yield
                assert!(!*this.finish);
                *this.finish = true;
            }
        }
    }
}

pub trait Service {
    /// Returns `Poll::Ready(Ok(()))` when the service is able to process requests.
    ///
    /// If the service is at capacity, then `Poll::Pending` is returned and the task
    /// is notified when the service becomes ready again. This function is
    /// expected to be called while on a task. Generally, this can be done with
    /// a simple `futures::future::poll_fn` call.
    ///
    /// If `Poll::Ready(Err(_))` is returned, the service is no longer able to service requests
    /// and the caller should discard the service instance.
    ///
    /// Once `poll_ready` returns `Poll::Ready(Ok(()))`, a request may be dispatched to the
    /// service using `call`. Until a request is dispatched, repeated calls to
    /// `poll_ready` must return either `Poll::Ready(Ok(()))` or `Poll::Ready(Err(_))`.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), StdError>>;

    /// Process the request and return the response asynchronously.
    ///
    /// This function is expected to be callable off task. As such,
    /// implementations should take care to not call `poll_ready`.
    ///
    /// Before dispatching a request, `poll_ready` must be called and return
    /// `Poll::Ready(Ok(()))`.
    ///
    /// # Panics
    ///
    /// Implementations are permitted to panic if `call` is invoked without
    /// obtaining `Poll::Ready(Ok(()))` from `poll_ready`.
    fn call(
        &mut self,
        req: crate::protocol::Packet,
    ) -> dyn futures::stream::Stream<Item = dyn std::convert::Into<crate::protocol::Packet>>;
}

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A service that tokio-tower should serve over the transport.
/// This one just echoes whatever it gets.
pub struct Echo;

impl Service for Echo {
    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), StdError>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        req: crate::protocol::Packet,
    ) -> dyn futures::stream::Stream<Item = dyn std::convert::Into<crate::protocol::Packet>> {
        println!("DEBUG! GOT A PACKET! {:?}", req);

        futures::stream::empty()
    }
}
