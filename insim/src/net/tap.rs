//! Taps: copies of selected packets from a session's input, taken without
//! removing anything from `read()`.
//!
//! The session owns a [`Taps`] registry and offers it every decoded packet before
//! that packet is queued for `read()`. Each [`Tap`] handle is filled while the
//! session is driven, whether by the caller's `read()` loop or by a split driver.
//! Nothing here touches a socket or reads the clock.

use std::{
    collections::VecDeque,
    fmt,
    future::poll_fn,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use crate::{Error, Packet, Result};

/// How a [`Tap`] keeps the packets it matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Keep every match, in order. The queue is unbounded, so drain the tap or drop it.
    Queue,
    /// Keep only the newest match that has not been received yet.
    Latest,
    /// Keep the first match, then finish.
    Once,
    /// Keep every match until none has arrived for this long, then finish. The
    /// quiet period starts when the tap is registered and restarts with each match.
    UntilQuiet(Duration),
}

/// Filter deciding which packets a tap copies.
pub(crate) type Filter = Box<dyn FnMut(&Packet) -> bool + Send>;

#[derive(Debug, Default)]
struct Slot {
    packets: VecDeque<Packet>,
    // `Some(Ok)` once the policy is satisfied, `Some(Err)` if the session ended first.
    end: Option<Result<()>>,
    waker: Option<Waker>,
}
impl Slot {
    fn wake(&mut self) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

fn lock(slot: &Mutex<Slot>) -> MutexGuard<'_, Slot> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What a tap yields next.
#[derive(Debug)]
pub(crate) enum Next {
    Packet(Packet),
    End(Result<()>),
}

/// Create a connected registry half and handle.
pub(crate) fn channel(policy: Policy) -> (Sink, Tap) {
    let slot = Arc::new(Mutex::new(Slot::default()));
    (
        Sink {
            slot: slot.clone(),
            policy,
        },
        Tap { slot },
    )
}

/// A copy of selected packets from a session.
///
/// Created by `Session::tap`, or by `Sender::tap` after an `into_split`. A tap
/// never takes packets away from `read()`: every packet it copies is still
/// delivered there as well. It fills only while the session is being driven.
///
/// Matches are copied when they are decoded, so a tap can see a packet before
/// `read()` returns it. A tap finishes when its [`Policy`] is satisfied or the
/// session ends. Dropping the handle unregisters it.
#[derive(Debug)]
pub struct Tap {
    slot: Arc<Mutex<Slot>>,
}
impl Tap {
    fn next(slot: &mut Slot) -> Option<Next> {
        if let Some(packet) = slot.packets.pop_front() {
            return Some(Next::Packet(packet));
        }
        match &mut slot.end {
            None => None,
            Some(Ok(())) => Some(Next::End(Ok(()))),
            // Report the cause once; later polls see a plain disconnection.
            Some(end) => Some(Next::End(std::mem::replace(end, Err(Error::Disconnected)))),
        }
    }
    pub(crate) fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Next> {
        let mut slot = lock(&self.slot);
        match Self::next(&mut slot) {
            Some(next) => Poll::Ready(next),
            None => {
                match &mut slot.waker {
                    Some(waker) => waker.clone_from(cx.waker()),
                    waker @ None => *waker = Some(cx.waker().clone()),
                }
                Poll::Pending
            },
        }
    }
    /// Wait for the next match. Returns `None` once the tap has finished.
    ///
    /// Works on any executor; the session must be driven elsewhere for it to fill.
    pub async fn recv(&mut self) -> Option<Packet> {
        match poll_fn(|cx| self.poll_next(cx)).await {
            Next::Packet(packet) => Some(packet),
            Next::End(_) => None,
        }
    }
    /// Take the next match without waiting.
    pub fn try_recv(&mut self) -> Option<Packet> {
        match Self::next(&mut lock(&self.slot)) {
            Some(Next::Packet(packet)) => Some(packet),
            _ => None,
        }
    }
    /// Whether the tap has finished and every match has been received.
    pub fn is_finished(&self) -> bool {
        let slot = lock(&self.slot);
        slot.end.is_some() && slot.packets.is_empty()
    }
}

/// The registry half of a tap. Dropping it before the policy is satisfied ends the
/// tap with [`Error::Disconnected`].
pub(crate) struct Sink {
    slot: Arc<Mutex<Slot>>,
    policy: Policy,
}
impl Sink {
    fn abandoned(&self) -> bool {
        Arc::strong_count(&self.slot) == 1
    }
    fn push(&self, packet: Packet) {
        let mut slot = lock(&self.slot);
        if self.policy == Policy::Latest {
            slot.packets.clear();
        }
        slot.packets.push_back(packet);
        if self.policy == Policy::Once {
            slot.end = Some(Ok(()));
        }
        slot.wake();
    }
    fn finish(&self, end: Result<()>) {
        let mut slot = lock(&self.slot);
        if slot.end.is_none() {
            slot.end = Some(end);
            slot.wake();
        }
    }
    /// End the tap with the error that prevented it from being registered.
    #[cfg(feature = "tokio")]
    pub(crate) fn fail(self, error: Error) {
        self.finish(Err(error));
    }
}
impl Drop for Sink {
    fn drop(&mut self) {
        self.finish(Err(Error::Disconnected));
    }
}
impl fmt::Debug for Sink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sink")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

struct Entry {
    sink: Sink,
    filter: Filter,
    quiet_at: Option<Instant>,
}

/// Registered taps. Driven by supplied packets and time.
#[derive(Default)]
pub(crate) struct Taps {
    entries: Vec<Entry>,
}
impl Taps {
    pub(crate) fn insert(&mut self, sink: Sink, filter: Filter, now: Instant) {
        let quiet_at = match sink.policy {
            Policy::UntilQuiet(quiet) => Some(now + quiet),
            _ => None,
        };
        self.entries.push(Entry {
            sink,
            filter,
            quiet_at,
        });
    }
    /// Copy a received packet to every tap whose filter accepts it.
    pub(crate) fn offer(&mut self, packet: &Packet, now: Instant) {
        self.entries.retain_mut(|entry| {
            if entry.sink.abandoned() {
                return false;
            }
            if !(entry.filter)(packet) {
                return true;
            }
            entry.sink.push(packet.clone());
            match entry.sink.policy {
                Policy::Once => false,
                Policy::UntilQuiet(quiet) => {
                    entry.quiet_at = Some(now + quiet);
                    true
                },
                Policy::Queue | Policy::Latest => true,
            }
        });
    }
    /// Finish quiet taps and forget abandoned ones.
    pub(crate) fn advance(&mut self, now: Instant) {
        self.entries.retain(|entry| {
            if entry.quiet_at.is_some_and(|at| now >= at) {
                entry.sink.finish(Ok(()));
                return false;
            }
            !entry.sink.abandoned()
        });
    }
    /// Earliest quiet deadline.
    #[cfg(any(feature = "tokio", feature = "blocking"))]
    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.entries.iter().filter_map(|e| e.quiet_at).min()
    }
    /// End every unfinished tap because the session has ended.
    pub(crate) fn close(&mut self) {
        self.entries.clear();
    }
}
impl fmt::Debug for Taps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Taps")
            .field("len", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{identifiers::RequestId, insim::Tiny};

    fn tiny(reqi: u8) -> Packet {
        Tiny {
            reqi: RequestId(reqi),
            ..Default::default()
        }
        .into()
    }
    fn reqi(packet: Option<Packet>) -> Option<u8> {
        packet.and_then(|p| p.request_id()).map(|r| r.0)
    }
    fn tap(taps: &mut Taps, policy: Policy, now: Instant) -> Tap {
        let (sink, tap) = channel(policy);
        taps.insert(
            sink,
            Box::new(|p| p.request_id() != Some(RequestId(0))),
            now,
        );
        tap
    }

    #[test]
    fn policies_keep_the_documented_matches() {
        let now = Instant::now();
        let mut taps = Taps::default();
        let mut queue = tap(&mut taps, Policy::Queue, now);
        let mut latest = tap(&mut taps, Policy::Latest, now);
        let mut once = tap(&mut taps, Policy::Once, now);
        for id in [0, 1, 2, 3] {
            taps.offer(&tiny(id), now);
        }
        assert_eq!(
            [(); 4].map(|_| reqi(queue.try_recv())),
            [Some(1), Some(2), Some(3), None]
        );
        assert!(!queue.is_finished());
        assert_eq!(reqi(latest.try_recv()), Some(3));
        assert_eq!(reqi(latest.try_recv()), None);
        assert_eq!(reqi(once.try_recv()), Some(1));
        assert!(once.is_finished());
        // Once is removed after its match; queue and latest remain.
        assert_eq!(taps.entries.len(), 2);
    }

    #[test]
    fn until_quiet_restarts_on_each_match_and_finishes_cleanly() {
        let now = Instant::now();
        let quiet = Duration::from_millis(100);
        let mut taps = Taps::default();
        let tap = tap(&mut taps, Policy::UntilQuiet(quiet), now);
        assert_eq!(taps.deadline(), Some(now + quiet));
        taps.offer(&tiny(1), now + Duration::from_millis(90));
        taps.advance(now + quiet);
        assert!(!tap.is_finished());
        assert_eq!(taps.deadline(), Some(now + Duration::from_millis(190)));
        taps.advance(now + Duration::from_millis(190));
        assert_eq!(taps.deadline(), None);
        let mut slot = lock(&tap.slot);
        assert!(matches!(Tap::next(&mut slot), Some(Next::Packet(_))));
        assert!(matches!(Tap::next(&mut slot), Some(Next::End(Ok(())))));
        drop(slot);
        assert!(tap.is_finished());
    }

    #[test]
    fn closing_or_dropping_ends_taps_with_disconnected() {
        let now = Instant::now();
        let mut taps = Taps::default();
        let mut open = tap(&mut taps, Policy::Queue, now);
        let dropped = tap(&mut taps, Policy::Queue, now);
        drop(dropped);
        taps.offer(&tiny(1), now);
        assert_eq!(taps.entries.len(), 1);
        taps.close();
        assert_eq!(reqi(open.try_recv()), Some(1));
        let mut slot = lock(&open.slot);
        assert!(matches!(
            Tap::next(&mut slot),
            Some(Next::End(Err(Error::Disconnected)))
        ));
    }
}
