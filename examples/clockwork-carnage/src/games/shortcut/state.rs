use std::{collections::HashMap, sync::Arc, time::Duration};

use insim::identifiers::{ConnectionId, PlayerId};
use kitcar::RoundPhase;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone, Default, Debug)]
pub(super) struct ShortcutGlobal {
    pub(super) phase: String,
    pub(super) leaderboard: Vec<(String, String, i64)>,
}

pub(super) struct ShortcutInner {
    pub(super) phase: RoundPhase,
    /// plid -> (ucid, uname, start_time)
    pub(super) active_runs: HashMap<PlayerId, (ConnectionId, String, Duration)>,
    /// (uname, pname, best_time_ms) sorted by time asc
    pub(super) leaderboard: Vec<(String, String, i64)>,
    pub(super) db: Option<(crate::db::Pool, i64)>,
}

#[derive(Clone)]
pub(super) struct Shortcut {
    inner: Arc<RwLock<ShortcutInner>>,
}

impl Shortcut {
    pub(super) fn new(db: Option<(crate::db::Pool, i64)>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ShortcutInner {
                phase: RoundPhase::default(),
                active_runs: HashMap::new(),
                leaderboard: Vec::new(),
                db,
            })),
        }
    }

    pub(super) fn read(&self) -> RwLockReadGuard<'_, ShortcutInner> {
        self.inner.read()
    }

    pub(super) fn write(&self) -> RwLockWriteGuard<'_, ShortcutInner> {
        self.inner.write()
    }
}

impl ShortcutInner {
    pub(super) fn update_leaderboard(&mut self, uname: &str, pname: &str, time_ms: i64) {
        if let Some(entry) = self.leaderboard.iter_mut().find(|e| e.0 == uname) {
            if time_ms < entry.2 {
                entry.2 = time_ms;
                entry.1 = pname.to_string();
            }
        } else {
            self.leaderboard
                .push((uname.to_string(), pname.to_string(), time_ms));
        }
        self.leaderboard.sort_by_key(|e| e.2);
        self.leaderboard.truncate(10);
    }

    pub(super) fn snapshot(&self) -> ShortcutGlobal {
        ShortcutGlobal {
            phase: self.phase.to_string(),
            leaderboard: self.leaderboard.clone(),
        }
    }
}
