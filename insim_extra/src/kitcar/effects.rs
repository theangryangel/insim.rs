//! [`Effects`]: what a [`Service`](crate::kitcar::Service) asks the runtime to do.
//!
//! A service never performs I/O itself. It returns effects, and the [`Host`](crate::kitcar::Host)
//! carries them out: packets are sent in order, timers are kept until they fire,
//! and tasks run off the dispatch loop with their results delivered back as
//! messages.

use std::{
    any::{Any, TypeId},
    fmt,
    future::Future,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{FutureExt, Stream, StreamExt, future::BoxFuture, stream::BoxStream};
use insim::{Packet, identifiers::ConnectionId};

use crate::ui::Key;

/// Identifies a timer so that scheduling it again replaces it, and so it can be
/// cancelled. Implemented for any `Debug + PartialEq + Send + 'static` type; a
/// small newtype per purpose (`struct Boom(PlayerId)`) keeps keys distinct.
pub trait TimerKey: Any + Send + fmt::Debug {
    #[doc(hidden)]
    fn same(&self, other: &dyn TimerKey) -> bool;
    #[doc(hidden)]
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any + Send + fmt::Debug + PartialEq> TimerKey for T {
    fn same(&self, other: &dyn TimerKey) -> bool {
        other.as_any().downcast_ref::<T>() == Some(self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// When a one-shot timer fires.
#[derive(Debug, Clone, Copy)]
pub(crate) enum When {
    At(Instant),
    After(Duration),
}
impl When {
    pub(crate) fn resolve(self, now: Instant) -> Instant {
        match self {
            Self::At(at) => at,
            Self::After(delay) => now + delay,
        }
    }
}

/// Which timers a cancellation removes.
#[derive(Debug)]
pub(crate) enum Cancel {
    Key(Box<dyn TimerKey>),
    Type(TypeId),
}
impl Cancel {
    pub(crate) fn matches(&self, key: &dyn TimerKey) -> bool {
        match self {
            Self::Key(cancel) => cancel.same(key),
            Self::Type(type_id) => key.as_any().type_id() == *type_id,
        }
    }
}

pub(crate) enum Effect<M> {
    Packet(Packet),
    Timer {
        key: Option<Box<dyn TimerKey>>,
        when: When,
        msg: M,
    },
    Every {
        key: Box<dyn TimerKey>,
        period: Duration,
        make: Arc<dyn Fn() -> M + Send + Sync>,
    },
    Cancel(Cancel),
    Task(BoxFuture<'static, M>),
    Stream(BoxStream<'static, M>),
    Send {
        ucid: Option<ConnectionId>,
        key: Box<dyn Key>,
        msg: Box<dyn Any + Send + Sync>,
    },
    Stop,
}

/// A list of things for the runtime to do, in order.
///
/// Built with chained calls and returned from [`Service::update`](crate::kitcar::Service::update):
///
/// ```ignore
/// Effects::new()
///     .message("Run started!", ucid)
///     .schedule(Boom(plid), deadline, Msg::Boom(plid))
///     .task(async move { Msg::Standings(board.standings().await) })
/// ```
#[must_use = "effects do nothing unless returned to the runtime"]
pub struct Effects<M> {
    pub(crate) items: Vec<Effect<M>>,
}

impl<M> Default for Effects<M> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<M> fmt::Debug for Effects<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for item in &self.items {
            let _ = match item {
                Effect::Packet(packet) => list.entry(packet),
                Effect::Timer { key, when, .. } => list.entry(&("timer", key, when)),
                Effect::Every { key, period, .. } => list.entry(&("every", key, period)),
                Effect::Cancel(cancel) => list.entry(&("cancel", cancel)),
                Effect::Task(_) => list.entry(&"task"),
                Effect::Stream(_) => list.entry(&"stream"),
                Effect::Send { ucid, key, .. } => list.entry(&("send", ucid, key)),
                Effect::Stop => list.entry(&"stop"),
            };
        }
        list.finish()
    }
}

impl<M: Send + 'static> Effects<M> {
    /// Nothing to do.
    pub fn none() -> Self {
        Self::default()
    }
    /// An empty list to build on.
    pub fn new() -> Self {
        Self::default()
    }
    fn push(mut self, effect: Effect<M>) -> Self {
        self.items.push(effect);
        self
    }

    /// Send a packet to LFS.
    pub fn packet(self, packet: impl Into<Packet>) -> Self {
        self.push(Effect::Packet(packet.into()))
    }
    /// Send packets to LFS, in order.
    pub fn packets<P: Into<Packet>>(mut self, packets: impl IntoIterator<Item = P>) -> Self {
        self.items
            .extend(packets.into_iter().map(|p| Effect::Packet(p.into())));
        self
    }
    /// Send a chat message to one connection, or everyone with
    /// [`ConnectionId::ALL`]. Long text is split into several messages.
    pub fn message(self, text: impl Into<String>, ucid: ConnectionId) -> Self {
        self.packets(crate::util::mtc(text, Some(ucid)))
    }

    /// Deliver `msg` after `delay`.
    pub fn after(self, delay: Duration, msg: M) -> Self {
        self.push(Effect::Timer {
            key: None,
            when: When::After(delay),
            msg,
        })
    }
    /// Deliver `msg` at `at`.
    pub fn at(self, at: Instant, msg: M) -> Self {
        self.push(Effect::Timer {
            key: None,
            when: When::At(at),
            msg,
        })
    }
    /// Deliver `msg` at `at`, replacing any timer with the same key.
    pub fn schedule(self, key: impl TimerKey, at: Instant, msg: M) -> Self {
        self.push(Effect::Timer {
            key: Some(Box::new(key)),
            when: When::At(at),
            msg,
        })
    }
    /// Deliver `msg` after `delay`, replacing any timer with the same key.
    pub fn schedule_after(self, key: impl TimerKey, delay: Duration, msg: M) -> Self {
        self.push(Effect::Timer {
            key: Some(Box::new(key)),
            when: When::After(delay),
            msg,
        })
    }
    /// Deliver `msg` every `period` until cancelled, replacing any timer with the
    /// same key. Missed ticks are skipped rather than delivered in a burst.
    pub fn every(self, key: impl TimerKey, period: Duration, msg: M) -> Self
    where
        M: Clone + Sync,
    {
        self.push(Effect::Every {
            key: Box::new(key),
            period,
            make: Arc::new(move || msg.clone()),
        })
    }
    /// Cancel the timer with this key, if any.
    pub fn cancel(self, key: impl TimerKey) -> Self {
        self.push(Effect::Cancel(Cancel::Key(Box::new(key))))
    }
    /// Cancel every timer whose key has type `K`.
    pub fn cancel_all<K: TimerKey>(self) -> Self {
        self.push(Effect::Cancel(Cancel::Type(TypeId::of::<K>())))
    }

    /// Run a future off the dispatch loop and deliver its output as a message.
    /// Use this for database and network work. Aborted if the service stops first.
    pub fn task(self, future: impl Future<Output = M> + Send + 'static) -> Self {
        self.push(Effect::Task(future.boxed()))
    }
    /// Deliver every item of a stream as a message, such as UI clicks from
    /// `Ui::messages()`. Dropped when the service stops.
    pub fn stream(self, stream: impl Stream<Item = M> + Send + 'static) -> Self {
        self.push(Effect::Stream(stream.boxed()))
    }
    /// Send `msg` to one player's component placed with `key`
    /// ([`Child::key`](crate::ui::Child::key)), as if it came from one of
    /// its buttons. For example, open a player's help dialog from a chat command.
    pub fn send(self, ucid: ConnectionId, key: impl Key, msg: impl Any + Send + Sync) -> Self {
        self.push(Effect::Send {
            ucid: Some(ucid),
            key: Box::new(key),
            msg: Box::new(msg),
        })
    }
    /// Like [`send`](Self::send), to the component with `key` for every player.
    pub fn send_all(self, key: impl Key, msg: impl Any + Send + Sync) -> Self {
        self.push(Effect::Send {
            ucid: None,
            key: Box::new(key),
            msg: Box::new(msg),
        })
    }
    /// Stop this service once these effects have been carried out.
    pub fn stop(self) -> Self {
        self.push(Effect::Stop)
    }

    /// Append another list of effects.
    pub fn and(mut self, other: Self) -> Self {
        self.items.extend(other.items);
        self
    }
    /// Convert the messages of a component's effects into its parent's message
    /// type, typically an enum variant: `self.clearer.update(..).map(Msg::Penalty)`.
    pub fn map<N: Send + 'static>(self, f: impl Fn(M) -> N + Send + Sync + 'static) -> Effects<N> {
        let f = Arc::new(f);
        Effects {
            items: self
                .items
                .into_iter()
                .map(|item| match item {
                    Effect::Packet(packet) => Effect::Packet(packet),
                    Effect::Timer { key, when, msg } => Effect::Timer {
                        key,
                        when,
                        msg: f(msg),
                    },
                    Effect::Every { key, period, make } => {
                        let f = f.clone();
                        Effect::Every {
                            key,
                            period,
                            make: Arc::new(move || f(make())),
                        }
                    },
                    Effect::Cancel(cancel) => Effect::Cancel(cancel),
                    Effect::Task(task) => {
                        let f = f.clone();
                        Effect::Task(task.map(move |m| f(m)).boxed())
                    },
                    Effect::Stream(stream) => {
                        let f = f.clone();
                        Effect::Stream(stream.map(move |m| f(m)).boxed())
                    },
                    Effect::Send { ucid, key, msg } => Effect::Send { ucid, key, msg },
                    Effect::Stop => Effect::Stop,
                })
                .collect(),
        }
    }

    /// The packets these effects would send, in order. Useful in tests.
    pub fn packets_sent(&self) -> impl Iterator<Item = &Packet> {
        self.items.iter().filter_map(|item| match item {
            Effect::Packet(packet) => Some(packet),
            _ => None,
        })
    }
    /// The messages of one-shot timers these effects would schedule, with their
    /// keys. Useful in tests.
    pub fn timers(&self) -> impl Iterator<Item = (Option<&dyn TimerKey>, &M)> {
        self.items.iter().filter_map(|item| match item {
            Effect::Timer { key, msg, .. } => Some((key.as_deref(), msg)),
            _ => None,
        })
    }
    /// Whether these effects stop the service. Useful in tests.
    pub fn stops(&self) -> bool {
        self.items.iter().any(|item| matches!(item, Effect::Stop))
    }
    /// Whether there is nothing to do.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct KeyA(u8);
    #[derive(Debug, PartialEq)]
    struct KeyB(u8);

    #[test]
    fn keys_compare_by_type_and_value() {
        assert!(KeyA(1).same(&KeyA(1)));
        assert!(!KeyA(1).same(&KeyA(2)));
        assert!(!KeyA(1).same(&KeyB(1)));
        assert!(Cancel::Type(TypeId::of::<KeyA>()).matches(&KeyA(9)));
        assert!(!Cancel::Type(TypeId::of::<KeyA>()).matches(&KeyB(9)));
    }

    #[test]
    fn map_converts_every_message_and_keeps_order() {
        let effects = Effects::<u8>::new()
            .message("hi", ConnectionId(1))
            .schedule(KeyA(1), Instant::now(), 5)
            .after(Duration::from_secs(1), 6)
            .stop()
            .map(|n| u16::from(n) * 10);
        assert_eq!(effects.packets_sent().count(), 1);
        let msgs: Vec<u16> = effects.timers().map(|(_, m)| *m).collect();
        assert_eq!(msgs, [50, 60]);
        assert!(effects.stops());
    }
}
