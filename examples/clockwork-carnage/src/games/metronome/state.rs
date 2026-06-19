use std::{collections::HashMap, sync::Arc, time::Duration};

use insim::identifiers::{ConnectionId, PlayerId};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::config::MetronomeConfig;

#[derive(Clone, Default, Debug)]
pub(super) struct MetronomeGlobal {
    pub(super) target: Duration,
    pub(super) leaderboard: Vec<(String, String, i64)>,
}

pub(super) struct MetronomeInner {
    pub(super) config: MetronomeConfig,
    /// plid -> (ucid, uname, checkpoint1_time)
    pub(super) active_runs: HashMap<PlayerId, (ConnectionId, String, Duration)>,
    /// (uname, pname, best_delta_ms) sorted by delta asc
    pub(super) leaderboard: Vec<(String, String, i64)>,
    pub(super) db: Option<(crate::db::Pool, i64)>,
}

#[derive(Clone)]
pub(super) struct Metronome {
    inner: Arc<RwLock<MetronomeInner>>,
}

impl Metronome {
    pub(super) fn new(config: MetronomeConfig, db: Option<(crate::db::Pool, i64)>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetronomeInner {
                config,
                active_runs: HashMap::new(),
                leaderboard: Vec::new(),
                db,
            })),
        }
    }

    pub(super) fn read(&self) -> RwLockReadGuard<'_, MetronomeInner> {
        self.inner.read()
    }

    pub(super) fn write(&self) -> RwLockWriteGuard<'_, MetronomeInner> {
        self.inner.write()
    }
}

impl MetronomeInner {
    pub(super) fn update_leaderboard(&mut self, uname: &str, pname: &str, delta_ms: i64) {
        if let Some(entry) = self.leaderboard.iter_mut().find(|e| e.0 == uname) {
            if delta_ms < entry.2 {
                entry.2 = delta_ms;
                entry.1 = pname.to_string();
            }
        } else {
            self.leaderboard
                .push((uname.to_string(), pname.to_string(), delta_ms));
        }
        self.leaderboard.sort_by_key(|e| e.2);
        self.leaderboard.truncate(10);
    }

    pub(super) fn snapshot(&self) -> MetronomeGlobal {
        MetronomeGlobal {
            target: self.config.target,
            leaderboard: self.leaderboard.clone(),
        }
    }
}
