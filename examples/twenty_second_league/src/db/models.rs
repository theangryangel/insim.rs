use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Player {
    pub id: i64,
    pub uname: String,
    pub pname: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Event {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub name: String,
    pub track: String,
    pub layout: Option<String>,
    pub target_time: String,
    pub restart_after: String,
    pub rounds: i64,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct LeaderboardEntry {
    pub pname: String,
    pub total_points: i64,
    pub position: i64,
}
