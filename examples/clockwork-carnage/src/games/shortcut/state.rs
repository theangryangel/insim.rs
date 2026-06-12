use std::{collections::HashMap, sync::Arc, time::Duration};

use insim::identifiers::{ConnectionId, PlayerId};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio_util::sync::CancellationToken;

use super::config::ShortcutConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum ShortcutPhase {
    #[default]
    Waiting,
    SettingUp,
    Racing,
}

impl ShortcutPhase {
    pub(super) fn label(self) -> &'static str {
        match self {
            ShortcutPhase::Waiting => "waiting for players",
            ShortcutPhase::SettingUp => "setting up track",
            ShortcutPhase::Racing => "racing",
        }
    }
}

#[derive(Clone, Default, Debug)]
pub(super) struct ShortcutGlobal {
    pub(super) phase: String,
    pub(super) leaderboard: Vec<(String, String, i64)>,
}

pub(super) struct ShortcutInner {
    pub(super) config: ShortcutConfig,
    pub(super) phase: ShortcutPhase,
    setup_cancel: Option<CancellationToken>,
    pub(super) runtime_cancel: CancellationToken,
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
    pub(super) fn new(
        config: ShortcutConfig,
        runtime_cancel: CancellationToken,
        db: Option<(crate::db::Pool, i64)>,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ShortcutInner {
                config,
                phase: ShortcutPhase::default(),
                setup_cancel: None,
                runtime_cancel,
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
            phase: self.phase.label().to_string(),
            leaderboard: self.leaderboard.clone(),
        }
    }
}
