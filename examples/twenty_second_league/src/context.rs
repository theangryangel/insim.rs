use std::sync::Arc;

use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct Context {
    pub insim: insim::builder::SpawnedHandle,
    pub ui: kitcar::ui::ManagerHandle<super::components::Root>,
    pub presence: kitcar::presence::PresenceHandle,
    pub game: kitcar::game::GameHandle,
    pub leaderboard: kitcar::leaderboard::LeaderboardHandle<String>,
    pub config: Arc<super::config::Config>,
    pub shutdown: CancellationToken,
    pub database: super::db::Repo,
}
