//! A service's pending timers. Driven by supplied time; it never reads the clock.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::effects::{Cancel, TimerKey};

struct Entry<M> {
    key: Option<Box<dyn TimerKey>>,
    at: Instant,
    // A repeating timer re-arms itself; a one-shot holds its message.
    repeat: Option<(Duration, Arc<dyn Fn() -> M + Send + Sync>)>,
    msg: Option<M>,
}

pub(crate) struct Timers<M> {
    entries: Vec<Entry<M>>,
}

impl<M> Default for Timers<M> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<M> Timers<M> {
    fn remove_key(&mut self, key: &dyn TimerKey) {
        self.entries
            .retain(|entry| !entry.key.as_deref().is_some_and(|k| k.same(key)));
    }
    pub(crate) fn once(&mut self, key: Option<Box<dyn TimerKey>>, at: Instant, msg: M) {
        if let Some(key) = &key {
            self.remove_key(key.as_ref());
        }
        self.entries.push(Entry {
            key,
            at,
            repeat: None,
            msg: Some(msg),
        });
    }
    pub(crate) fn every(
        &mut self,
        key: Box<dyn TimerKey>,
        period: Duration,
        make: Arc<dyn Fn() -> M + Send + Sync>,
        now: Instant,
    ) {
        self.remove_key(key.as_ref());
        self.entries.push(Entry {
            key: Some(key),
            at: now + period,
            repeat: Some((period, make)),
            msg: None,
        });
    }
    pub(crate) fn cancel(&mut self, cancel: &Cancel) {
        self.entries
            .retain(|entry| !entry.key.as_deref().is_some_and(|k| cancel.matches(k)));
    }
    /// The earliest time a timer fires.
    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.entries.iter().map(|entry| entry.at).min()
    }
    /// Remove due timers and return their messages, earliest first. Repeating
    /// timers are re-armed for their next period after `now`.
    pub(crate) fn expire(&mut self, now: Instant) -> Vec<M> {
        let mut due: Vec<(Instant, M)> = Vec::new();
        self.entries.retain_mut(|entry| {
            if entry.at > now {
                return true;
            }
            match &entry.repeat {
                Some((period, make)) => {
                    due.push((entry.at, make()));
                    while entry.at <= now {
                        entry.at += *period;
                    }
                    true
                },
                None => {
                    if let Some(msg) = entry.msg.take() {
                        due.push((entry.at, msg));
                    }
                    false
                },
            }
        });
        due.sort_by_key(|(at, _)| *at);
        due.into_iter().map(|(_, msg)| msg).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct Key(u8);

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn keyed_timers_replace_and_fire_in_time_order() {
        let now = Instant::now();
        let mut timers = Timers::default();
        timers.once(Some(Box::new(Key(1))), now + ms(50), "first");
        timers.once(None, now + ms(20), "unkeyed");
        // Same key: replaces the 50ms timer.
        timers.once(Some(Box::new(Key(1))), now + ms(30), "moved");
        assert_eq!(timers.deadline(), Some(now + ms(20)));
        assert!(timers.expire(now + ms(10)).is_empty());
        assert_eq!(timers.expire(now + ms(60)), ["unkeyed", "moved"]);
        assert_eq!(timers.deadline(), None);
    }

    #[test]
    fn cancel_by_key_or_type() {
        let now = Instant::now();
        let mut timers = Timers::default();
        for n in 0..3 {
            timers.once(Some(Box::new(Key(n))), now, n);
        }
        timers.cancel(&Cancel::Key(Box::new(Key(1))));
        assert_eq!(timers.expire(now), [0, 2]);
        for n in 0..3 {
            timers.once(Some(Box::new(Key(n))), now, n);
        }
        timers.cancel(&Cancel::Type(TypeId::of::<Key>()));
        assert!(timers.expire(now).is_empty());
    }

    #[test]
    fn repeating_timers_skip_missed_ticks() {
        let now = Instant::now();
        let mut timers = Timers::default();
        timers.every(Box::new(Key(0)), ms(100), Arc::new(|| "tick"), now);
        assert_eq!(timers.expire(now + ms(100)), ["tick"]);
        // Far behind: one tick, not three, and re-armed after `now`.
        assert_eq!(timers.expire(now + ms(450)), ["tick"]);
        assert_eq!(timers.deadline(), Some(now + ms(500)));
    }
}
