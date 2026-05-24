use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::PlayerType,
};
use tokio_util::sync::CancellationToken;

use super::config::MetronomeConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum MetronomePhase {
    #[default]
    Waiting,
    SettingUp,
    Racing,
}

#[derive(Clone, Debug)]
pub(super) struct PlayerInfo {
    pub(super) ucid: ConnectionId,
    pub(super) uname: String,
    pub(super) pname: String,
    pub(super) ptype: PlayerType,
}

#[derive(Clone, Default, Debug)]
pub(super) struct MetronomeGlobal {
    pub(super) target: Duration,
    pub(super) leaderboard: Vec<(String, String, i64)>,
}

pub(super) struct MetronomeInner {
    pub(super) config: MetronomeConfig,
    pub(super) phase: MetronomePhase,
    setup_cancel: Option<CancellationToken>,
    pub(super) runtime_cancel: CancellationToken,
    /// plid → (ucid, uname, checkpoint1_time)
    pub(super) active_runs: HashMap<PlayerId, (ConnectionId, String, Duration)>,
    pub(super) players: HashMap<PlayerId, PlayerInfo>,
    /// (uname, pname, best_delta_ms) sorted by delta asc
    pub(super) leaderboard: Vec<(String, String, i64)>,
    pub(super) db: Option<(crate::db::Pool, i64)>,
}

#[derive(Clone)]
pub(super) struct Metronome {
    inner: Arc<RwLock<MetronomeInner>>,
}

impl Metronome {
    pub(super) fn new(
        config: MetronomeConfig,
        runtime_cancel: CancellationToken,
        db: Option<(crate::db::Pool, i64)>,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetronomeInner {
                config,
                phase: MetronomePhase::default(),
                setup_cancel: None,
                runtime_cancel,
                active_runs: HashMap::new(),
                players: HashMap::new(),
                leaderboard: Vec::new(),
                db,
            })),
        }
    }

    pub(super) fn read(&self) -> RwLockReadGuard<'_, MetronomeInner> {
        self.inner.read().expect("poison")
    }

    pub(super) fn write(&self) -> RwLockWriteGuard<'_, MetronomeInner> {
        self.inner.write().expect("poison")
    }
}

impl MetronomeInner {
    pub(super) fn make_setup_cancel(&mut self) -> CancellationToken {
        let token = self.runtime_cancel.child_token();
        self.setup_cancel = Some(token.clone());
        token
    }

    pub(super) fn cancel_setup(&mut self) {
        if let Some(c) = self.setup_cancel.take() {
            c.cancel();
        }
    }

    pub(super) fn clear_setup_cancel(&mut self) {
        self.setup_cancel = None;
    }

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
