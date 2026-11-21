//! Requests: a packet sent with a reserved ReqI, and a tap for the replies that
//! echo it.

use std::{
    future::Future,
    marker::PhantomData,
    ops::RangeInclusive,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

use super::tap::{self, Filter, Next, Policy, Sink, Tap};
use crate::{Packet, Result, WithRequestId, error::Error, identifiers::RequestId};

/// ReqI values `request` and `request_all` use unless changed with
/// `Session::set_request_ids`. Packets you send yourself should use ReqIs outside
/// the session's range, or replies to them may be mistaken for replies to a request.
pub const DEFAULT_REQUEST_IDS: RangeInclusive<u8> = 128..=255;

/// How long `request_all` waits after the last reply, or after sending when
/// nothing replies, before deciding that every reply has arrived.
pub const REQUEST_ALL_QUIET: Duration = Duration::from_secs(1);

/// A packet type that a request can resolve to. Implemented for every packet
/// struct (`Ism`, `Ncn`, ...) and for [`Packet`] itself, which accepts any reply.
pub trait Response: Sized + Send + 'static {
    #[doc(hidden)]
    fn matches(packet: &Packet) -> bool;
    #[doc(hidden)]
    fn from_packet(packet: Packet) -> Option<Self>;
}
impl<T> Response for T
where
    T: TryFrom<Packet> + Send + 'static,
    for<'a> &'a T: TryFrom<&'a Packet>,
{
    fn matches(packet: &Packet) -> bool {
        <&T>::try_from(packet).is_ok()
    }
    fn from_packet(packet: Packet) -> Option<Self> {
        T::try_from(packet).ok()
    }
}

/// Cycles through a nonempty range of nonzero ReqIs.
#[derive(Debug)]
pub(crate) struct RequestIds {
    range: RangeInclusive<u8>,
    next: u8,
}
impl Default for RequestIds {
    fn default() -> Self {
        Self::new(DEFAULT_REQUEST_IDS)
    }
}
impl RequestIds {
    /// # Panics
    /// If `range` is empty or includes 0, which LFS does not correlate.
    pub(crate) fn new(range: RangeInclusive<u8>) -> Self {
        assert!(
            !range.is_empty() && *range.start() != 0,
            "request ids must be a nonempty range within 1..=255, got {range:?}"
        );
        Self {
            next: *range.start(),
            range,
        }
    }
    pub(crate) fn next(&mut self) -> RequestId {
        let id = self.next;
        self.next = if id == *self.range.end() {
            *self.range.start()
        } else {
            id + 1
        };
        RequestId(id)
    }
    pub(crate) fn range(&self) -> &RangeInclusive<u8> {
        &self.range
    }
}

/// A request whose ReqI has not been allocated yet.
#[derive(Debug)]
pub(crate) struct Pending {
    packet: Packet,
    sink: Sink,
    matches: fn(&Packet) -> bool,
}
impl Pending {
    /// Stamp the packet and build the matching filter.
    pub(crate) fn start(self, reqi: RequestId) -> (Packet, Sink, Filter) {
        let matches = self.matches;
        let filter = Box::new(move |p: &Packet| p.request_id() == Some(reqi) && matches(p));
        (self.packet.with_request_id(reqi).into(), self.sink, filter)
    }
}

pub(crate) fn reply<T: Response>(packet: Packet) -> (Pending, Reply<T>) {
    let (sink, tap) = tap::channel(Policy::Once);
    (
        Pending {
            packet,
            sink,
            matches: T::matches,
        },
        Reply {
            tap,
            marker: PhantomData,
        },
    )
}

pub(crate) fn replies<T: Response>(packet: Packet, quiet: Duration) -> (Pending, Replies<T>) {
    let (sink, tap) = tap::channel(Policy::UntilQuiet(quiet));
    (
        Pending {
            packet,
            sink,
            matches: T::matches,
        },
        Replies {
            tap,
            replies: Vec::new(),
        },
    )
}

fn poll_now<F: Future + Unpin>(future: &mut F) -> Option<F::Output> {
    match Pin::new(future).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(output) => Some(output),
        Poll::Pending => None,
    }
}

// FIXME: requests have no built-in timeout. LFS never answers some requests
// (e.g. TINY_ISM outside multiplayer), so a `Reply` can wait forever unless the
// caller wraps it in a timeout. Consider a deadline in the tap registry (like
// `UntilQuiet`) that ends the tap with `Error::Timeout`, with a default and a
// `*_within` override. This applies to `Session::request` and `Sender::request`.
/// The first reply to a request. Await it, or check with [`Reply::try_get`].
///
/// It fills in while the session is driven, so keep calling `read()` (for example in
/// `tokio::select!`) or use a split session. The reply is still delivered by
/// `read()` too. LFS does not answer every request, so consider a timeout.
#[derive(Debug)]
#[must_use = "a reply does nothing unless awaited or checked"]
pub struct Reply<T> {
    tap: Tap,
    marker: PhantomData<fn() -> T>,
}
impl<T: Response> Reply<T> {
    /// The reply, if it has arrived, without waiting.
    pub fn try_get(&mut self) -> Option<Result<T>> {
        poll_now(self)
    }
}
impl<T: Response> Future for Reply<T> {
    type Output = Result<T>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<T>> {
        let this = self.get_mut();
        loop {
            return Poll::Ready(match std::task::ready!(this.tap.poll_next(cx)) {
                Next::Packet(packet) => match T::from_packet(packet) {
                    Some(reply) => Ok(reply),
                    None => continue,
                },
                Next::End(Ok(())) => Err(Error::Disconnected),
                Next::End(Err(error)) => Err(error),
            });
        }
    }
}

/// Every reply to a request, collected until the replies go quiet.
///
/// Like [`Reply`], it fills in only while the session is driven.
#[derive(Debug)]
#[must_use = "replies do nothing unless awaited or checked"]
pub struct Replies<T> {
    tap: Tap,
    replies: Vec<T>,
}
impl<T: Response> Replies<T> {
    /// All replies, if they have finished arriving, without waiting.
    pub fn try_get(&mut self) -> Option<Result<Vec<T>>> {
        poll_now(self)
    }
}
impl<T: Response> Future for Replies<T> {
    type Output = Result<Vec<T>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Vec<T>>> {
        let this = self.get_mut();
        loop {
            match std::task::ready!(this.tap.poll_next(cx)) {
                Next::Packet(packet) => this.replies.extend(T::from_packet(packet)),
                Next::End(Ok(())) => return Poll::Ready(Ok(std::mem::take(&mut this.replies))),
                Next::End(Err(error)) => return Poll::Ready(Err(error)),
            }
        }
    }
}
// Neither type is self-referential.
impl<T> Unpin for Reply<T> {}
impl<T> Unpin for Replies<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::insim::{Ism, Ncn, TinyType};

    #[test]
    fn request_ids_cycle_through_the_configured_range() {
        let mut ids = RequestIds::default();
        let first: Vec<_> = (0..129).map(|_| ids.next().0).collect();
        assert_eq!(first[0], 128);
        assert_eq!(first[127], 255);
        assert_eq!(first[128], 128);
        let mut ids = RequestIds::new(1..=2);
        assert_eq!([(); 3].map(|_| ids.next().0), [1, 2, 1]);
        let mut ids = RequestIds::new(255..=255);
        assert_eq!([(); 2].map(|_| ids.next().0), [255, 255]);
    }

    #[test]
    #[should_panic(expected = "nonempty range within 1..=255")]
    fn request_ids_reject_zero() {
        let _ = RequestIds::new(0..=10);
    }

    #[test]
    #[should_panic(expected = "nonempty range within 1..=255")]
    #[allow(clippy::reversed_empty_ranges)]
    fn request_ids_reject_an_empty_range() {
        let _ = RequestIds::new(10..=5);
    }

    #[test]
    fn filter_matches_only_the_echoed_reqi_and_type() {
        let (pending, _reply) = reply::<Ism>(TinyType::Ism.into());
        let (packet, _sink, mut filter) = pending.start(RequestId(130));
        assert_eq!(packet.request_id(), Some(RequestId(130)));
        let ism = |reqi| -> Packet {
            Ism {
                reqi: RequestId(reqi),
                ..Default::default()
            }
            .into()
        };
        assert!(filter(&ism(130)));
        assert!(!filter(&ism(131)));
        assert!(!filter(
            &Ncn {
                reqi: RequestId(130),
                ..Default::default()
            }
            .into()
        ));
        // `Packet` accepts any reply type with the right ReqI.
        let (pending, _reply) = reply::<Packet>(TinyType::Ism.into());
        let (_, _, mut any) = pending.start(RequestId(130));
        assert!(any(&ism(130)));
    }
}
