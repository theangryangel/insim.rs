//! [`Effects`]: what a [`Service`](crate::kitcar::Service) asks the runtime to do.
//!
//! Services never do I/O themselves. The [`Host`](crate::kitcar::Host) sends
//! packets in order, holds timers until they fire, and runs tasks off the
//! dispatch loop, delivering their results as messages.

use std::{
    any::{Any, TypeId},
    fmt,
    future::Future,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{FutureExt, Stream, StreamExt, future::BoxFuture};
use insim::{Packet, identifiers::ConnectionId, insim::Mtc};

pub use crate::timers::TimerKey;
use crate::{
    kitcar::director::DirectorCommand,
    timers::{Cancel, Job, When},
    ui::Key,
};

pub(crate) enum Effect<M> {
    Packet(Packet),
    Job(Job<M>),
    Send {
        ucid: Option<ConnectionId>,
        key: Box<dyn Key>,
        msg: Box<dyn Any + Send + Sync>,
    },
    Stop,
    Director(DirectorCommand),
}

/// A list of things for the runtime to do, in order.
///
/// Chain calls and return it from [`Service::update`](crate::kitcar::Service::update):
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
                Effect::Job(job) => list.entry(job),
                Effect::Send { ucid, key, .. } => list.entry(&("send", ucid, key)),
                Effect::Stop => list.entry(&"stop"),
                Effect::Director(command) => list.entry(&("director", command)),
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
    /// Start a chain.
    pub fn new() -> Self {
        Self::default()
    }
    fn push(mut self, effect: Effect<M>) -> Self {
        self.items.push(effect);
        self
    }

    /// Send a packet.
    pub fn packet(self, packet: impl Into<Packet>) -> Self {
        self.push(Effect::Packet(packet.into()))
    }
    /// Send packets, in order.
    pub fn packets<P: Into<Packet>>(mut self, packets: impl IntoIterator<Item = P>) -> Self {
        self.items
            .extend(packets.into_iter().map(|p| Effect::Packet(p.into())));
        self
    }
    /// Chat to one connection, or everyone with [`ConnectionId::ALL`]. Long
    /// text is split across messages.
    pub fn message(self, text: impl Into<String>, ucid: ConnectionId) -> Self {
        let text = text.into();
        self.packets(text.lines().map(|line| Mtc::new(ucid, line)))
    }

    fn job(self, job: Job<M>) -> Self {
        self.push(Effect::Job(job))
    }

    /// Deliver `msg` after `delay`.
    pub fn after(self, delay: Duration, msg: M) -> Self {
        self.job(Job::Once {
            key: None,
            when: When::After(delay),
            msg,
        })
    }
    /// Deliver `msg` at `at`.
    pub fn at(self, at: Instant, msg: M) -> Self {
        self.job(Job::Once {
            key: None,
            when: When::At(at),
            msg,
        })
    }
    /// Deliver `msg` at `at`, replacing anything pending under the same key.
    pub fn schedule(self, key: impl TimerKey, at: Instant, msg: M) -> Self {
        self.job(Job::Once {
            key: Some(Box::new(key)),
            when: When::At(at),
            msg,
        })
    }
    /// Deliver `msg` after `delay`, replacing anything pending under the same
    /// key.
    pub fn schedule_after(self, key: impl TimerKey, delay: Duration, msg: M) -> Self {
        self.job(Job::Once {
            key: Some(Box::new(key)),
            when: When::After(delay),
            msg,
        })
    }
    /// Deliver `msg` every `period` until cancelled, replacing anything
    /// pending under the same key. Missed ticks are skipped, not delivered in
    /// a burst.
    pub fn every(self, key: impl TimerKey, period: Duration, msg: M) -> Self
    where
        M: Clone + Sync,
    {
        self.job(Job::Every {
            key: Box::new(key),
            period,
            make: Arc::new(move || msg.clone()),
        })
    }
    /// Cancel whatever is pending under this key: a timer or a keyed task.
    pub fn cancel(self, key: impl TimerKey) -> Self {
        self.job(Job::Cancel(Cancel::Key(Box::new(key))))
    }
    /// Cancel everything pending under a key of type `K`.
    pub fn cancel_all<K: TimerKey>(self) -> Self {
        self.job(Job::Cancel(Cancel::Type(TypeId::of::<K>())))
    }

    /// Run a future off the dispatch loop and deliver its output as a message.
    /// For database and network work. Aborted if the service stops first.
    pub fn task(self, future: impl Future<Output = M> + Send + 'static) -> Self {
        self.job(Job::Task {
            key: None,
            future: future.boxed(),
        })
    }
    /// Like [`task`](Self::task), replacing anything pending under the same
    /// key. A task it replaces is aborted, and its result never arrives: only
    /// the latest of several lookups lands.
    pub fn keyed_task(
        self,
        key: impl TimerKey,
        future: impl Future<Output = M> + Send + 'static,
    ) -> Self {
        self.job(Job::Task {
            key: Some(Box::new(key)),
            future: future.boxed(),
        })
    }
    /// Deliver each item of a stream as a message. Dropped when the service
    /// stops.
    pub fn stream(self, stream: impl Stream<Item = M> + Send + 'static) -> Self {
        self.job(Job::Stream(stream.boxed()))
    }
    /// Send `msg` to one player's component placed with `key`
    /// ([`Child::key`](crate::ui::Child::key)), as if from its own buttons.
    /// Handy for opening a dialog from a chat command.
    pub fn send(self, ucid: ConnectionId, key: impl Key, msg: impl Any + Send + Sync) -> Self {
        self.push(Effect::Send {
            ucid: Some(ucid),
            key: Box::new(key),
            msg: Box::new(msg),
        })
    }
    /// Like [`send`](Self::send), for every player.
    pub fn send_all(self, key: impl Key, msg: impl Any + Send + Sync) -> Self {
        self.push(Effect::Send {
            ucid: None,
            key: Box::new(key),
            msg: Box::new(msg),
        })
    }
    /// Stop this service once these effects are carried out.
    pub fn stop(self) -> Self {
        self.push(Effect::Stop)
    }
    /// Steer the host's [`RaceDirector`](crate::kitcar::director::RaceDirector),
    /// say from an admin's chat command. It acts once these effects are carried
    /// out, before the next service sees the input. Ignored, with a warning,
    /// if the host has no director.
    pub fn director(self, command: DirectorCommand) -> Self {
        self.push(Effect::Director(command))
    }

    /// Append `other`.
    pub fn and(mut self, other: Self) -> Self {
        self.items.extend(other.items);
        self
    }
    /// Lift a component's messages into its parent's:
    /// `self.part.update(..).map(Msg::Part)`.
    pub fn map<N: Send + 'static>(self, f: impl Fn(M) -> N + Send + Sync + 'static) -> Effects<N> {
        let f: Arc<dyn Fn(M) -> N + Send + Sync> = Arc::new(f);
        Effects {
            items: self
                .items
                .into_iter()
                .map(|item| match item {
                    Effect::Packet(packet) => Effect::Packet(packet),
                    Effect::Job(job) => Effect::Job(job.map(f.clone())),
                    Effect::Send { ucid, key, msg } => Effect::Send { ucid, key, msg },
                    Effect::Stop => Effect::Stop,
                    Effect::Director(command) => Effect::Director(command),
                })
                .collect(),
        }
    }

    /// Commands for the director, in order. For tests.
    pub fn director_commands(&self) -> impl Iterator<Item = &DirectorCommand> {
        self.items.iter().filter_map(|item| match item {
            Effect::Director(command) => Some(command),
            _ => None,
        })
    }
    /// Packets to send, in order. For tests.
    pub fn packets_sent(&self) -> impl Iterator<Item = &Packet> {
        self.items.iter().filter_map(|item| match item {
            Effect::Packet(packet) => Some(packet),
            _ => None,
        })
    }
    /// One-shot timers to schedule, with their keys. For tests.
    pub fn timers(&self) -> impl Iterator<Item = (Option<&dyn TimerKey>, &M)> {
        self.items.iter().filter_map(|item| match item {
            Effect::Job(Job::Once { key, msg, .. }) => Some((key.as_deref(), msg)),
            _ => None,
        })
    }
    /// Take the tasks out to run by hand. For tests: await each and pass its
    /// message back to `update`.
    pub fn take_tasks(&mut self) -> Vec<BoxFuture<'static, M>> {
        let mut tasks = Vec::new();
        self.items.retain_mut(|item| match item {
            Effect::Job(Job::Task { .. }) => {
                if let Effect::Job(Job::Task { future, .. }) = std::mem::replace(item, Effect::Stop)
                {
                    tasks.push(future);
                }
                false
            },
            _ => true,
        });
        tasks
    }
    /// Whether these effects stop the service. For tests.
    pub fn stops(&self) -> bool {
        self.items.iter().any(|item| matches!(item, Effect::Stop))
    }
    /// Nothing to do.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct KeyA(u8);

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
