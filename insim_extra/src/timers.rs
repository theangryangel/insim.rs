//! Keyed timers and background work on Tokio, whose results come back as
//! messages.
//!
//! kitcar's services and ui's components both schedule work through their
//! effects: a message after a delay or at an instant, one every period, the
//! output of a future, or each item of a stream. Each runs as its own Tokio
//! task and is aborted when its owner goes away, so nothing outlives the
//! service or component that asked for it.
//!
//! A [`TimerKey`] names a piece of work. Scheduling under a key replaces
//! whatever is pending under it, and cancelling by key or key type removes it.
//! A message already on its way when its work is replaced or cancelled is
//! dropped, never delivered.
//!
//! Clocks are Tokio's, so `#[tokio::test(start_paused = true)]` controls every
//! timer, and the `now` every update and render sees.

use std::{
    any::{Any, TypeId},
    fmt,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use futures::{FutureExt, StreamExt, future::BoxFuture, stream::BoxStream};
use tokio::{task::AbortHandle, time::MissedTickBehavior};

/// Names a timer or task, so scheduling it again replaces it and it can be
/// cancelled. Implemented for any `Debug + PartialEq + Send + 'static` type. A
/// newtype per purpose (`struct Boom(PlayerId)`) keeps keys distinct.
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

/// The current time on Tokio's clock, which a paused test runtime controls.
pub(crate) fn now() -> Instant {
    tokio::time::Instant::now().into_std()
}

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

/// One piece of work an owner asked for.
pub(crate) enum Job<M> {
    Once {
        key: Option<Box<dyn TimerKey>>,
        when: When,
        msg: M,
    },
    Every {
        key: Box<dyn TimerKey>,
        period: Duration,
        make: Arc<dyn Fn() -> M + Send + Sync>,
    },
    Task {
        key: Option<Box<dyn TimerKey>>,
        future: BoxFuture<'static, M>,
    },
    Stream(BoxStream<'static, M>),
    Cancel(Cancel),
}

impl<M> fmt::Debug for Job<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Once { key, when, .. } => f.debug_tuple("Once").field(key).field(when).finish(),
            Self::Every { key, period, .. } => {
                f.debug_tuple("Every").field(key).field(period).finish()
            },
            Self::Task { key, .. } => f.debug_tuple("Task").field(key).finish(),
            Self::Stream(_) => f.write_str("Stream"),
            Self::Cancel(cancel) => f.debug_tuple("Cancel").field(cancel).finish(),
        }
    }
}

impl<M: Send + 'static> Job<M> {
    /// Convert the messages this job produces.
    pub(crate) fn map<N: Send + 'static>(self, f: Arc<dyn Fn(M) -> N + Send + Sync>) -> Job<N> {
        match self {
            Self::Once { key, when, msg } => Job::Once {
                key,
                when,
                msg: f(msg),
            },
            Self::Every { key, period, make } => Job::Every {
                key,
                period,
                make: Arc::new(move || f(make())),
            },
            Self::Task { key, future } => Job::Task {
                key,
                future: future.map(move |msg| f(msg)).boxed(),
            },
            Self::Stream(stream) => Job::Stream(stream.map(move |msg| f(msg)).boxed()),
            Self::Cancel(cancel) => Job::Cancel(cancel),
        }
    }
}

/// Tells a job's messages apart from those of work since replaced or
/// cancelled. Unique for the life of the process.
pub(crate) type JobId = u64;

fn next_id() -> JobId {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Where a job's messages go. Returns false once nobody is listening, which
/// ends a repeating job.
pub(crate) type Sink<M> = Arc<dyn Fn(JobId, M) -> bool + Send + Sync>;

struct Entry {
    id: JobId,
    key: Option<Box<dyn TimerKey>>,
    handle: AbortHandle,
    /// Delivers more than once: an interval or a stream.
    repeats: bool,
}

/// One owner's pending work, each piece a Tokio task feeding its [`Sink`].
///
/// The owner hands every message it receives to [`accept`](Self::accept)
/// before acting on it: a message from work since replaced or cancelled was
/// already in flight, and is refused. Dropping `Timers` aborts everything.
pub(crate) struct Timers<M> {
    sink: Sink<M>,
    entries: Vec<Entry>,
}

impl<M> fmt::Debug for Timers<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Timers")
            .field("pending", &self.entries.len())
            .finish_non_exhaustive()
    }
}

impl<M> Drop for Timers<M> {
    fn drop(&mut self) {
        for entry in &self.entries {
            entry.handle.abort();
        }
    }
}

impl<M: Send + 'static> Timers<M> {
    pub(crate) fn new(sink: Sink<M>) -> Self {
        Self {
            sink,
            entries: Vec::new(),
        }
    }

    /// Start `job`. Needs a Tokio runtime. `now` resolves relative delays.
    pub(crate) fn run(&mut self, job: Job<M>, now: Instant) {
        let id = next_id();
        let sink = self.sink.clone();
        let (key, repeats, handle) = match job {
            Job::Cancel(cancel) => {
                self.cancel(&cancel);
                return;
            },
            Job::Once { key, when, msg } => {
                let at = tokio::time::Instant::from_std(when.resolve(now));
                let handle = tokio::spawn(async move {
                    tokio::time::sleep_until(at).await;
                    let _ = sink(id, msg);
                });
                (key, false, handle)
            },
            Job::Every { key, period, make } => {
                // A zero period would make Tokio panic; one millisecond is as
                // fast as anything here needs.
                let period = period.max(Duration::from_millis(1));
                let start = tokio::time::Instant::from_std(now) + period;
                let handle = tokio::spawn(async move {
                    let mut interval = tokio::time::interval_at(start, period);
                    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                    loop {
                        let _ = interval.tick().await;
                        if !sink(id, make()) {
                            break;
                        }
                    }
                });
                (Some(key), true, handle)
            },
            Job::Task { key, future } => {
                let handle = tokio::spawn(async move {
                    let _ = sink(id, future.await);
                });
                (key, false, handle)
            },
            Job::Stream(mut stream) => {
                let handle = tokio::spawn(async move {
                    while let Some(msg) = stream.next().await {
                        if !sink(id, msg) {
                            break;
                        }
                    }
                });
                (None, true, handle)
            },
        };
        // Replaces whatever was pending under the same key.
        if let Some(key) = key.as_deref() {
            self.entries.retain(|entry| {
                let same = entry.key.as_deref().is_some_and(|k| k.same(key));
                if same {
                    entry.handle.abort();
                }
                !same
            });
        }
        self.entries.push(Entry {
            id,
            key,
            handle: handle.abort_handle(),
            repeats,
        });
    }

    fn cancel(&mut self, cancel: &Cancel) {
        self.entries.retain(|entry| {
            let matched = entry.key.as_deref().is_some_and(|key| cancel.matches(key));
            if matched {
                entry.handle.abort();
            }
            !matched
        });
    }

    /// Whether a message from job `id` should be acted on: false if the job
    /// was replaced or cancelled after sending it. A one-shot is done once
    /// accepted.
    pub(crate) fn accept(&mut self, id: JobId) -> bool {
        let Some(index) = self.entries.iter().position(|entry| entry.id == id) else {
            return false;
        };
        if !self.entries[index].repeats {
            let _ = self.entries.swap_remove(index);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct KeyA(u8);
    #[derive(Debug, PartialEq)]
    struct KeyB(u8);

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn timers() -> (
        Timers<&'static str>,
        mpsc::UnboundedReceiver<(JobId, &'static str)>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let sink: Sink<&'static str> = Arc::new(move |id, msg| tx.send((id, msg)).is_ok());
        (Timers::new(sink), rx)
    }

    fn once(key: Option<KeyA>, delay: u64, msg: &'static str) -> Job<&'static str> {
        Job::Once {
            key: key.map(|k| Box::new(k) as Box<dyn TimerKey>),
            when: When::After(ms(delay)),
            msg,
        }
    }

    /// Nothing more arrives, however long we wait.
    async fn quiet(rx: &mut mpsc::UnboundedReceiver<(JobId, &'static str)>) -> bool {
        !matches!(
            tokio::time::timeout(Duration::from_secs(60), rx.recv()).await,
            Ok(Some(_))
        )
    }

    #[test]
    fn keys_compare_by_type_and_value() {
        assert!(KeyA(1).same(&KeyA(1)));
        assert!(!KeyA(1).same(&KeyA(2)));
        assert!(!KeyA(1).same(&KeyB(1)));
        assert!(Cancel::Type(TypeId::of::<KeyA>()).matches(&KeyA(9)));
        assert!(!Cancel::Type(TypeId::of::<KeyA>()).matches(&KeyB(9)));
    }

    #[tokio::test(start_paused = true)]
    async fn fire_in_time_order_and_once() {
        let (mut timers, mut rx) = timers();
        timers.run(once(Some(KeyA(1)), 50, "late"), now());
        timers.run(once(None, 20, "early"), now());
        let (id, msg) = rx.recv().await.unwrap();
        assert_eq!(msg, "early");
        assert!(timers.accept(id));
        // A one-shot is accepted once.
        assert!(!timers.accept(id));
        assert_eq!(rx.recv().await.unwrap().1, "late");
    }

    #[tokio::test(start_paused = true)]
    async fn replaced_or_cancelled_work_is_refused_even_in_flight() {
        let (mut timers, mut rx) = timers();
        timers.run(once(Some(KeyA(1)), 10, "old"), now());
        // It has fired and is waiting to be handled...
        let (old, _) = rx.recv().await.unwrap();
        // ...when the same key is scheduled again.
        timers.run(once(Some(KeyA(1)), 10, "new"), now());
        assert!(!timers.accept(old));
        let (new, msg) = rx.recv().await.unwrap();
        assert_eq!(msg, "new");
        assert!(timers.accept(new));

        timers.run(once(Some(KeyA(2)), 10, "cancelled"), now());
        let (cancelled, _) = rx.recv().await.unwrap();
        timers.run(Job::Cancel(Cancel::Type(TypeId::of::<KeyA>())), now());
        assert!(!timers.accept(cancelled));
    }

    #[tokio::test(start_paused = true)]
    async fn every_ticks_each_period_until_cancelled() {
        let (mut timers, mut rx) = timers();
        let start = tokio::time::Instant::now();
        timers.run(
            Job::Every {
                key: Box::new(KeyA(0)),
                period: ms(100),
                make: Arc::new(|| "tick"),
            },
            now(),
        );
        for n in 1..=3 {
            let (id, _) = rx.recv().await.unwrap();
            assert_eq!(start.elapsed(), ms(100 * n));
            // Repeating work stays accepted.
            assert!(timers.accept(id));
        }
        timers.run(Job::Cancel(Cancel::Key(Box::new(KeyA(0)))), now());
        assert!(quiet(&mut rx).await);
    }

    #[tokio::test(start_paused = true)]
    async fn tasks_report_and_dropping_aborts_everything() {
        let (mut timers, mut rx) = timers();
        timers.run(
            Job::Task {
                key: None,
                future: async { "loaded" }.boxed(),
            },
            now(),
        );
        let (id, msg) = rx.recv().await.unwrap();
        assert_eq!(msg, "loaded");
        assert!(timers.accept(id));

        timers.run(once(None, 10, "never"), now());
        timers.run(
            Job::Task {
                key: None,
                future: async {
                    tokio::time::sleep(ms(10)).await;
                    "never"
                }
                .boxed(),
            },
            now(),
        );
        drop(timers);
        assert!(quiet(&mut rx).await);
    }
}
