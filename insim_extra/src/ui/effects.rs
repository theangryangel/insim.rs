//! [`Effects`]: what a [`Component`](super::Component) asks for beyond its own
//! state.

use std::{
    any::TypeId,
    fmt,
    future::Future,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{FutureExt, Stream, StreamExt, future::BoxFuture};

use crate::timers::{Cancel, Job, TimerKey, When};

/// Work for a component's screen to start, and what to report to its parent,
/// in order. Return it from [`Component::mounted`](super::Component::mounted)
/// and [`Component::update`](super::Component::update):
///
/// ```ignore
/// fn update(&mut self, msg: Msg) -> Effects<Msg, Picked> {
///     match msg {
///         Msg::Next => {
///             self.page += 1;
///             let (store, page) = (self.store.clone(), self.page);
///             Effects::new().keyed_task(Fetch, async move { Msg::Loaded(page, store.page(page).await) })
///         },
///         Msg::Loaded(page, rows) => { self.rows = rows; Effects::none() },
///         Msg::Pick(row) => Effects::new().emit(Picked(row)),
///     }
/// }
/// ```
///
/// Timers, tasks and streams belong to the component instance: their messages
/// come back to its `update`, and they are aborted when it unmounts. Anything
/// outside the UI, such as sending packets, is for the service: [`emit`](Self::emit)
/// and let the parent's [`Child::on`](super::Child::on) turn it into a message.
#[must_use = "effects do nothing unless returned to the screen"]
pub struct Effects<M, O = ()> {
    pub(crate) jobs: Vec<Job<M>>,
    pub(crate) emits: Vec<O>,
}

impl<M, O> Default for Effects<M, O> {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            emits: Vec::new(),
        }
    }
}

impl<M, O: fmt::Debug> fmt::Debug for Effects<M, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Effects")
            .field("jobs", &self.jobs)
            .field("emits", &self.emits)
            .finish()
    }
}

impl<M: Send + 'static, O> Effects<M, O> {
    /// Nothing to do.
    pub fn none() -> Self {
        Self::default()
    }
    /// Start a chain.
    pub fn new() -> Self {
        Self::default()
    }
    fn job(mut self, job: Job<M>) -> Self {
        self.jobs.push(job);
        self
    }

    /// Report `out` to the parent, which turns it into one of its own
    /// messages with [`Child::on`](super::Child::on). Dropped if the parent
    /// gave no `on`.
    pub fn emit(mut self, out: O) -> Self {
        self.emits.push(out);
        self
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
    /// key. Schedule on every keystroke to debounce a type-in.
    pub fn schedule_after(self, key: impl TimerKey, delay: Duration, msg: M) -> Self {
        self.job(Job::Once {
            key: Some(Box::new(key)),
            when: When::After(delay),
            msg,
        })
    }
    /// Deliver `msg` every `period` until cancelled or unmounted, replacing
    /// anything pending under the same key. Missed ticks are skipped.
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

    /// Run a future and deliver its output as a message: a database query, a
    /// web request. Aborted if the component unmounts first.
    pub fn task(self, future: impl Future<Output = M> + Send + 'static) -> Self {
        self.job(Job::Task {
            key: None,
            future: future.boxed(),
        })
    }
    /// Like [`task`](Self::task), replacing anything pending under the same
    /// key. A task it replaces is aborted, and its result never arrives: click
    /// "next page" five times and only the last page lands.
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
    /// Deliver each item of a stream as a message, until it ends or the
    /// component unmounts.
    pub fn stream(self, stream: impl Stream<Item = M> + Send + 'static) -> Self {
        self.job(Job::Stream(stream.boxed()))
    }

    /// Append `other`.
    pub fn and(mut self, other: Self) -> Self {
        self.jobs.extend(other.jobs);
        self.emits.extend(other.emits);
        self
    }

    /// What it reports to the parent, in order. For tests.
    pub fn emitted(&self) -> impl Iterator<Item = &O> {
        self.emits.iter()
    }
    /// One-shot timers to schedule, with their keys. For tests.
    pub fn timers(&self) -> impl Iterator<Item = (Option<&dyn TimerKey>, &M)> {
        self.jobs.iter().filter_map(|job| match job {
            Job::Once { key, msg, .. } => Some((key.as_deref(), msg)),
            _ => None,
        })
    }
    /// Take the tasks out to run by hand. For tests: await each and pass its
    /// message back to `update`.
    pub fn take_tasks(&mut self) -> Vec<BoxFuture<'static, M>> {
        let mut tasks = Vec::new();
        let mut kept = Vec::new();
        for job in self.jobs.drain(..) {
            match job {
                Job::Task { future, .. } => tasks.push(future),
                other => kept.push(other),
            }
        }
        self.jobs = kept;
        tasks
    }
    /// Nothing to do or report.
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty() && self.emits.is_empty()
    }
}
